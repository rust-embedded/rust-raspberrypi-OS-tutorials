// SPDX-License-Identifier: MIT
//
// Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>

// Rust embedded logo for `make doc`.
#![doc(html_logo_url = "https://git.io/JeGIp")]

//! The `kernel`
//!
//! The `kernel` is composed by glueing together hardware-specific Board Support
//! Package (`BSP`) code and hardware-agnostic `kernel` code through the
//! [`kernel::interface`] traits.
//!
//! [`kernel::interface`]: interface/index.html

#![feature(format_args_nl)]
#![feature(panic_info_message)]
#![no_main]
#![no_std]

// This module conditionally includes the correct `BSP` which provides the
// `_start()` function, the first function to run.
mod bsp;

// Afterwards, `BSP`'s early init code calls `runtime_init::init()` of this
// module, which on completion, jumps to `kernel_entry()`.
mod runtime_init;

mod interface;
mod print;

/// Entrypoint of the `kernel`.
fn kernel_entry() -> ! {
    println!("[0] Hello from pure Rust!");

    println!("[1] Stopping here.");
    bsp::wait_forever()
}
