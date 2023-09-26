// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2022-2023 Andre Richter <andre.o.richter@gmail.com>

//! A simple sanity test to see if exception restore code works.

#![feature(format_args_nl)]
#![no_main]
#![no_std]

/// Console tests should time out on the I/O harness in case of panic.
mod panic_wait_forever;

use core::arch::asm;
use libkernel::{bsp, cpu, exception, info, memory, println};

#[inline(never)]
fn nested_system_call() {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        asm!("svc #0x1337", options(nomem, nostack, preserves_flags));
    }

    #[cfg(not(target_arch = "aarch64"))]
    {
        info!("Not supported yet");
        cpu::wait_forever();
    }
}

#[no_mangle]
unsafe fn kernel_init() -> ! {
    exception::handling_init();
    memory::init();
    bsp::driver::qemu_bring_up_console();

    // This line will be printed as the test header.
    println!("Testing exception restore");

    info!("Making a dummy system call");

    // Calling this inside a function indirectly tests if the link register is restored properly.
    nested_system_call();

    info!("Back from system call!");

    // The QEMU process running this test will be closed by the I/O test harness.
    cpu::wait_forever();
}
