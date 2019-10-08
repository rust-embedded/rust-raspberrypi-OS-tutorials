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
#![feature(trait_alias)]
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
    use interface::console::Statistics;

    // Initialize the BSP's device drivers.
    for i in bsp::device_drivers().iter() {
        if let Err(()) = i.init() {
            // This message will only be readable if, at the time of failure,
            // the return value of `bsp::console()` is already in functioning
            // state.
            panic!("Error loading driver: {}", i.compatible())
        }
    }

    // If all drivers are loaded, UART is functional now and `println!()` calls
    // are transmitted on the physical wires.
    println!("[0] Hello from pure Rust!");

    println!("[1] Drivers probed:");
    for (i, driver) in bsp::device_drivers().iter().enumerate() {
        println!("    {}. {}", i + 1, driver.compatible());
    }

    println!("[2] Chars written: {}", bsp::console().chars_written());

    println!("[3] Stopping here.");
    bsp::wait_forever()
}
