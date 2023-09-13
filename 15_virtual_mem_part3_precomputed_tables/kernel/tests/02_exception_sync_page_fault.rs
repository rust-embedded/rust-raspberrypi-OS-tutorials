// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2019-2023 Andre Richter <andre.o.richter@gmail.com>

//! Page faults must result in synchronous exceptions.

#![feature(format_args_nl)]
#![no_main]
#![no_std]

/// Overwrites libkernel's `panic_wait::_panic_exit()` so that it returns a "success" code.
///
/// In this test, reaching the panic is a success, because it is called from the synchronous
/// exception handler, which is what this test wants to achieve.
///
/// It also means that this integration test can not use any other code that calls panic!() directly
/// or indirectly.
mod panic_exit_success;

use libkernel::{bsp, cpu, exception, info, memory, println};

#[no_mangle]
unsafe fn kernel_init() -> ! {
    exception::handling_init();
    memory::init();
    bsp::driver::qemu_bring_up_console();

    // This line will be printed as the test header.
    println!("Testing synchronous exception handling by causing a page fault");

    info!("Writing beyond mapped area to address 9 GiB...");
    let big_addr: u64 = 9 * 1024 * 1024 * 1024;
    core::ptr::read_volatile(big_addr as *mut u64);

    // If execution reaches here, the memory access above did not cause a page fault exception.
    cpu::qemu_exit_failure()
}
