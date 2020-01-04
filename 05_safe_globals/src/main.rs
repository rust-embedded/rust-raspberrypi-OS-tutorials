// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

// Rust embedded logo for `make doc`.
#![doc(html_logo_url = "https://git.io/JeGIp")]

//! The `kernel`
//!
//! The `kernel` is composed by glueing together code from
//!
//!   - [Hardware-specific Board Support Packages] (`BSPs`).
//!   - [Architecture-specific code].
//!   - HW- and architecture-agnostic `kernel` code.
//!
//! using the [`kernel::interface`] traits.
//!
//! [Hardware-specific Board Support Packages]: bsp/index.html
//! [Architecture-specific code]: arch/index.html
//! [`kernel::interface`]: interface/index.html

#![feature(format_args_nl)]
#![feature(panic_info_message)]
#![feature(trait_alias)]
#![no_main]
#![no_std]

// Conditionally includes the selected `architecture` code, which provides the `_start()` function,
// the first function to run.
mod arch;

// `_start()` then calls `runtime_init()`, which on completion, jumps to `kernel_init()`.
mod runtime_init;

// Conditionally includes the selected `BSP` code.
mod bsp;

mod interface;
mod memory;
mod panic_wait;
mod print;

/// Early init code.
///
/// # Safety
///
/// - Only a single core must be active and running this function.
unsafe fn kernel_init() -> ! {
    use interface::console::Statistics;

    println!("[0] Hello from pure Rust!");

    println!("[1] Chars written: {}", bsp::console().chars_written());

    println!("[2] Stopping here.");
    arch::wait_forever()
}
