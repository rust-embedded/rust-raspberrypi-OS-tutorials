// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

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

// `_start()` then calls `runtime_init()`, which on completion, jumps to `kernel_init()`.
mod runtime_init;

// Conditionally includes the selected `BSP` code.
mod bsp;

mod memory;
mod panic_wait;

/// Early init code.
///
/// # Safety
///
/// - Only a single core must be active and running this function.
unsafe fn kernel_init() -> ! {
    panic!()
}
