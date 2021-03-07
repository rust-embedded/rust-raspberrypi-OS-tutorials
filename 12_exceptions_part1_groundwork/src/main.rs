// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

// Rust embedded logo for `make doc`.
#![doc(html_logo_url = "https://git.io/JeGIp")]

//! The `kernel` binary.
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
//! architectural code that belongs to the `kernel`'s MMU subsystem (`src/memory/mmu.rs`) would go
//! into `src/_arch/aarch64/memory/mmu.rs`. The latter file is loaded as a module in
//! `src/memory/mmu.rs` using the `path attribute`. Usually, the chosen module name is the generic
//! module's name prefixed with `arch_`.
//!
//! For example, this is the top of `src/memory/mmu.rs`:
//!
//! ```
//! #[cfg(target_arch = "aarch64")]
//! #[path = "../_arch/aarch64/memory/mmu.rs"]
//! mod arch_mmu;
//! ```
//!
//! Often times, items from the `arch_ module` will be publicly reexported by the parent module.
//! This way, each architecture specific module can provide its implementation of an item, while the
//! caller must not be concerned which architecture has been conditionally compiled.
//!
//! ## BSP code
//!
//! `BSP` stands for Board Support Package. `BSP` code is organized under `src/bsp.rs` and contains
//! target board specific definitions and functions. These are things such as the board's memory map
//! or instances of drivers for devices that are featured on the respective board.
//!
//! Just like processor architecture code, the `BSP` code's module structure tries to mirror the
//! `kernel`'s subsystem modules, but there is no reexporting this time. That means whatever is
//! provided must be called starting from the `bsp` namespace, e.g. `bsp::driver::driver_manager()`.
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
//!
//! # Boot flow
//!
//! 1. The kernel's entry point is the function [`cpu::boot::arch_boot::_start()`].
//!     - It is implemented in `src/_arch/__arch_name__/cpu/boot.rs`.
//! 2. Once finished with architectural setup, the arch code calls [`runtime_init::runtime_init()`].
//!
//! [`cpu::boot::arch_boot::_start()`]: cpu/boot/arch_boot/fn._start.html
//! [`runtime_init::runtime_init()`]: runtime_init/fn.runtime_init.html

#![allow(clippy::clippy::upper_case_acronyms)]
#![allow(incomplete_features)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(const_generics)]
#![feature(const_panic)]
#![feature(format_args_nl)]
#![feature(global_asm)]
#![feature(panic_info_message)]
#![feature(trait_alias)]
#![no_main]
#![no_std]

mod bsp;
mod console;
mod cpu;
mod driver;
mod exception;
mod memory;
mod panic_wait;
mod print;
mod runtime_init;
mod synchronization;
mod time;

/// Early init code.
///
/// # Safety
///
/// - Only a single core must be active and running this function.
/// - The init calls in this function must appear in the correct order:
///     - Virtual memory must be activated before the device drivers.
///       - Without it, any atomic operations, e.g. the yet-to-be-introduced spinlocks in the device
///         drivers (which currently employ NullLocks instead of spinlocks), will fail to work on
///         the RPi SoCs.
unsafe fn kernel_init() -> ! {
    use driver::interface::DriverManager;
    use memory::mmu::interface::MMU;

    exception::handling_init();

    if let Err(string) = memory::mmu::mmu().init() {
        panic!("MMU: {}", string);
    }

    for i in bsp::driver::driver_manager().all_device_drivers().iter() {
        if let Err(x) = i.init() {
            panic!("Error loading driver: {}: {}", i.compatible(), x);
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
    use core::time::Duration;
    use driver::interface::DriverManager;
    use time::interface::TimeManager;

    info!("Booting on: {}", bsp::board_name());

    info!("MMU online. Special regions:");
    bsp::memory::mmu::virt_mem_layout().print_layout();

    let (_, privilege_level) = exception::current_privilege_level();
    info!("Current privilege level: {}", privilege_level);

    info!("Exception handling state:");
    exception::asynchronous::print_state();

    info!(
        "Architectural timer resolution: {} ns",
        time::time_manager().resolution().as_nanos()
    );

    info!("Drivers loaded:");
    for (i, driver) in bsp::driver::driver_manager()
        .all_device_drivers()
        .iter()
        .enumerate()
    {
        info!("      {}. {}", i + 1, driver.compatible());
    }

    info!("Timer test, spinning for 1 second");
    time::time_manager().spin_for(Duration::from_secs(1));

    // Cause an exception by accessing a virtual address for which no translation was set up. This
    // code accesses the address 8 GiB, which is outside the mapped address space.
    //
    // For demo purposes, the exception handler will catch the faulting 8 GiB address and allow
    // execution to continue.
    info!("");
    info!("Trying to read from address 8 GiB...");
    let mut big_addr: u64 = 8 * 1024 * 1024 * 1024;
    unsafe { core::ptr::read_volatile(big_addr as *mut u64) };

    info!("************************************************");
    info!("Whoa! We recovered from a synchronous exception!");
    info!("************************************************");
    info!("");
    info!("Let's try again");

    // Now use address 9 GiB. The exception handler won't forgive us this time.
    info!("Trying to read from address 9 GiB...");
    big_addr = 9 * 1024 * 1024 * 1024;
    unsafe { core::ptr::read_volatile(big_addr as *mut u64) };

    // Will never reach here in this tutorial.
    info!("Echoing input now");

    // Discard any spurious received characters before going into echo mode.
    console().clear_rx();
    loop {
        let c = bsp::console::console().read_char();
        bsp::console::console().write_char(c);
    }
}
