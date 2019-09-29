// SPDX-License-Identifier: MIT
//
// Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>

// Rust embedded logo for `make doc`.
#![doc(html_logo_url = "https://git.io/JeGIp")]

//! The `kernel`

#![feature(asm)]
#![feature(global_asm)]
#![feature(naked_functions)]
#![no_main]
#![no_std]

// This module conditionally includes the correct `BSP` which provides the
// `_start()` function, the first function to run.
mod bsp;

// Afterwards, `BSP`'s early init code calls `runtime_init::init()` of this
// module, which on completion, jumps to `kernel_entry()`.
mod runtime_init;

/// Entrypoint of the `kernel`.
fn kernel_entry() -> ! {
    panic!()
}
