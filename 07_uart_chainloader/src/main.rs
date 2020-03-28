// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

// Rust embedded logo for `make doc`.
#![doc(html_logo_url = "https://git.io/JeGIp")]

//! The `kernel` binary.
//!
//! # TL;DR - Overview of important Kernel entities
//!
//! - [`bsp::console::console()`] - Returns a reference to the kernel's [console interface].
//! - [`bsp::driver::driver_manager()`] - Returns a reference to the kernel's [driver interface].
//!
//! [console interface]: ../libkernel/console/interface/index.html
//! [driver interface]: ../libkernel/driver/interface/trait.DriverManager.html
//!
//! # Code organization and architecture
//!
//! The code is divided into different *modules*, each representing a typical **subsystem** of the
//! `kernel`. Top-level module files of subsystems reside directly in the `src` folder. For example,
//! `src/memory.rs` contains code that is concerned with all things memory management.
//!
//! ## Visibility of processor architecture code
//!
//! Some of the `kernel`'s subsystems depend on low-level code that is specific to the target
//! processor architecture. For each supported processor architecture, there exists a subfolder in
//! `src/_arch`, for example, `src/_arch/aarch64`.
//!
//! The architecture folders mirror the subsystem modules laid out in `src`. For example,
//! architectural code that belongs to the `kernel`'s memory subsystem (`src/memory.rs`) would go
//! into `src/_arch/aarch64/memory.rs`. The latter file is directly included and re-exported in
//! `src/memory.rs`, so that the architectural code parts are transparent with respect to the code's
//! module organization. That means a public function `foo()` defined in
//! `src/_arch/aarch64/memory.rs` would be reachable as `crate::memory::foo()` only.
//!
//! The `_` in `_arch` denotes that this folder is not part of the standard module hierarchy.
//! Rather, it's contents are conditionally pulled into respective files using the `#[path =
//! "_arch/xxx/yyy.rs"]` attribute.
//!
//! ## BSP code
//!
//! `BSP` stands for Board Support Package. `BSP` code is organized under `src/bsp.rs` and contains
//! target board specific definitions and functions. These are things such as the board's memory map
//! or instances of drivers for devices that are featured on the respective board.
//!
//! Just like processor architecture code, the `BSP` code's module structure tries to mirror the
//! `kernel`'s subsystem modules, but there is no transparent re-exporting this time. That means
//! whatever is provided must be called starting from the `bsp` namespace, e.g.
//! `bsp::driver::driver_manager()`.
//!
//! ## Kernel interfaces
//!
//! Both `arch` and `bsp` contain code that is conditionally compiled depending on the actual target
//! and board for which the kernel is compiled. For example, the `interrupt controller` hardware of
//! the `Raspberry Pi 3` and the `Raspberry Pi 4` is different, but we want the rest of the `kernel`
//! code to play nicely with any of the two without much hassle.
//!
//! In order to provide a clean abstraction between `arch`, `bsp` and `generic kernel code`,
//! `interface` traits are provided *whenever possible* and *where it makes sense*. They are defined
//! in the respective subsystem module and help to enforce the idiom of *program to an interface,
//! not an implementation*. For example, there will be a common IRQ handling interface which the two
//! different interrupt controller `drivers` of both Raspberrys will implement, and only export the
//! interface to the rest of the `kernel`.
//!
//! ```
//!         +-------------------+
//!         | Interface (Trait) |
//!         |                   |
//!         +--+-------------+--+
//!            ^             ^
//!            |             |
//!            |             |
//! +----------+--+       +--+----------+
//! | kernel code |       |  bsp code   |
//! |             |       |  arch code  |
//! +-------------+       +-------------+
//! ```
//!
//! # Summary
//!
//! For a logical `kernel` subsystem, corresponding code can be distributed over several physical
//! locations. Here is an example for the **memory** subsystem:
//!
//! - `src/memory.rs` and `src/memory/**/*`
//!   - Common code that is agnostic of target processor architecture and `BSP` characteristics.
//!     - Example: A function to zero a chunk of memory.
//!   - Interfaces for the memory subsystem that are implemented by `arch` or `BSP` code.
//!     - Example: An `MMU` interface that defines `MMU` function prototypes.
//! - `src/bsp/__board_name__/memory.rs` and `src/bsp/__board_name__/memory/**/*`
//!   - `BSP` specific code.
//!   - Example: The board's memory map (physical addresses of DRAM and MMIO devices).
//! - `src/_arch/__arch_name__/memory.rs` and `src/_arch/__arch_name__/memory/**/*`
//!   - Processor architecture specific code.
//!   - Example: Implementation of the `MMU` interface for the `__arch_name__` processor
//!     architecture.
//!
//! From a namespace perspective, **memory** subsystem code lives in:
//!
//! - `crate::memory::*`
//! - `crate::bsp::memory::*`

#![feature(format_args_nl)]
#![feature(naked_functions)]
#![feature(panic_info_message)]
#![feature(trait_alias)]
#![no_main]
#![no_std]

// `mod cpu` provides the `_start()` function, the first function to run. `_start()` then calls
// `relocate::relocate_self()`. `relocate::relocate_self()` calls `runtime_init()`, which jumps to
// `kernel_init()`.

mod bsp;
mod console;
mod cpu;
mod driver;
mod memory;
mod panic_wait;
mod print;
mod relocate;
mod runtime_init;
mod synchronization;

/// Early init code.
///
/// # Safety
///
/// - Only a single core must be active and running this function.
/// - The init calls in this function must appear in the correct order.
unsafe fn kernel_init() -> ! {
    use driver::interface::DriverManager;

    for i in bsp::driver::driver_manager().all_device_drivers().iter() {
        if i.init().is_err() {
            panic!("Error loading driver: {}", i.compatible())
        }
    }
    bsp::driver::driver_manager().post_device_driver_init();
    // println! is usable from here on.

    // Transition from unsafe to safe.
    kernel_main()
}

/// The main function running after the early init.
fn kernel_main() -> ! {
    use bsp::console::console;
    use console::interface::All;

    println!(" __  __ _      _ _                 _ ");
    println!("|  \\/  (_)_ _ (_) |   ___  __ _ __| |");
    println!("| |\\/| | | ' \\| | |__/ _ \\/ _` / _` |");
    println!("|_|  |_|_|_||_|_|____\\___/\\__,_\\__,_|");
    println!();
    println!("{:^37}", bsp::board_name());
    println!();
    println!("[ML] Requesting binary");
    console().flush();

    // Clear the RX FIFOs, if any, of spurious received characters before starting with the loader
    // protocol.
    console().clear();

    // Notify `Minipush` to send the binary.
    for _ in 0..3 {
        console().write_char(3 as char);
    }

    // Read the binary's size.
    let mut size: u32 = u32::from(console().read_char() as u8);
    size |= u32::from(console().read_char() as u8) << 8;
    size |= u32::from(console().read_char() as u8) << 16;
    size |= u32::from(console().read_char() as u8) << 24;

    // Trust it's not too big.
    console().write_char('O');
    console().write_char('K');

    let kernel_addr: *mut u8 = bsp::cpu::BOARD_DEFAULT_LOAD_ADDRESS as *mut u8;
    unsafe {
        // Read the kernel byte by byte.
        for i in 0..size {
            *kernel_addr.offset(i as isize) = console().read_char() as u8;
        }
    }

    println!("[ML] Loaded! Executing the payload now\n");
    console().flush();

    // Use black magic to get a function pointer.
    let kernel: extern "C" fn() -> ! = unsafe { core::mem::transmute(kernel_addr as *const ()) };

    // Jump to loaded kernel!
    kernel()
}
