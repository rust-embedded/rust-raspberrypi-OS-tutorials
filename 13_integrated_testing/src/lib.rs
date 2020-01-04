// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

// Rust embedded logo for `make doc`.
#![doc(html_logo_url = "https://git.io/JeGIp")]

//! The `kernel` library.
//!
//! Used by `main.rs` to compose the final kernel binary.

#![allow(incomplete_features)]
#![feature(const_generics)]
#![feature(format_args_nl)]
#![feature(global_asm)]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(slice_ptr_range)]
#![feature(trait_alias)]
#![no_std]
// Testing
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(crate::test_runner)]

// Conditionally includes the selected `architecture` code, which provides the `_start()` function,
// the first function to run.
pub mod arch;

// `_start()` then calls `runtime_init()`, which on completion, jumps to `kernel_init()`.
mod runtime_init;

// Conditionally includes the selected `BSP` code.
pub mod bsp;

pub mod interface;
mod memory;
mod panic_wait;
pub mod print;

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
    bsp::qemu_bring_up_console();

    test_main();

    arch::qemu_exit_success()
}
