// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

// Rust embedded logo for `make doc`.
#![doc(html_logo_url = "https://git.io/JeGIp")]

//! The `kernel` library.
//!
//! Used by `main.rs` to compose the final kernel binary.
//!
//! # TL;DR - Overview of important Kernel entities
//!
//! - [`bsp::console::console()`] - Returns a reference to the kernel's [console interface].
//! - [`bsp::driver::driver_manager()`] - Returns a reference to the kernel's [driver interface].
//! - [`memory::mmu::mmu()`] - Returns a reference to the kernel's [MMU interface].
//! - [`time::time_manager()`] - Returns a reference to the kernel's [timer interface].
//!
//! [console interface]: ../libkernel/console/interface/index.html
//! [driver interface]: ../libkernel/driver/interface/trait.DriverManager.html
//! [MMU interface]: ../libkernel/memory/mmu/interface/trait.MMU.html
//! [timer interface]: ../libkernel/time/interface/trait.TimeManager.html
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

#![allow(incomplete_features)]
#![feature(const_generics)]
#![feature(custom_inner_attributes)]
#![feature(format_args_nl)]
#![feature(global_asm)]
#![feature(linkage)]
#![feature(naked_functions)]
#![feature(panic_info_message)]
#![feature(slice_ptr_range)]
#![feature(trait_alias)]
#![no_std]
// Testing
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(crate::test_runner)]

// `mod cpu` provides the `_start()` function, the first function to run. `_start()` then calls
// `runtime_init()`, which jumps to `kernel_init()` (defined in `main.rs`).

mod panic_wait;
mod runtime_init;
mod synchronization;

pub mod bsp;
pub mod console;
pub mod cpu;
pub mod driver;
pub mod exception;
pub mod memory;
pub mod print;
pub mod time;

//--------------------------------------------------------------------------------------------------
// Testing
//--------------------------------------------------------------------------------------------------

/// The default runner for unit tests.
pub fn test_runner(tests: &[&test_types::UnitTest]) {
    println!("Running {} tests", tests.len());
    println!("-------------------------------------------------------------------\n");
    for (i, test) in tests.iter().enumerate() {
        print!("{:>3}. {:.<58}", i + 1, test.name);

        // Run the actual test.
        (test.test_func)();

        // Failed tests call panic!(). Execution reaches here only if the test has passed.
        println!("[ok]")
    }
}

/// The `kernel_init()` for unit tests. Called from `runtime_init()`.
#[cfg(test)]
#[no_mangle]
unsafe fn kernel_init() -> ! {
    bsp::console::qemu_bring_up_console();

    test_main();

    cpu::qemu_exit_success()
}
