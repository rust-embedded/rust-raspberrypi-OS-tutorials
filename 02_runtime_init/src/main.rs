// SPDX-License-Identifier: MIT
//
// Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>

// Rust embedded logo for `make doc`.
#![doc(html_logo_url = "https://git.io/JeGIp")]

//! The `kernel`

#![feature(asm)]
#![feature(global_asm)]
#![no_main]
#![no_std]

// Conditionally includes the selected `architecture` code, which provides the `_start()` function,
// the first function to run.
mod arch;

// `_start()` then calls `runtime_init::init()`, which on completion, jumps to `kernel_entry()`.
mod runtime_init;

// Conditionally includes the selected `BSP` code.
mod bsp;

mod panic_wait;

/// Entrypoint of the `kernel`.
fn kernel_entry() -> ! {
    panic!()
}
