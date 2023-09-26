// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2022-2023 Andre Richter <andre.o.richter@gmail.com>

//! Test if backtracing code detects an invalid link.

#![feature(format_args_nl)]
#![no_main]
#![no_std]

/// Console tests should time out on the I/O harness in case of panic.
mod panic_wait_forever;

use libkernel::{backtrace, bsp, cpu, exception, memory};

#[inline(never)]
fn nested_2() -> &'static str {
    unsafe { backtrace::corrupt_link() };
    libkernel::println!("{}", libkernel::backtrace::Backtrace);
    "foo"
}

#[inline(never)]
fn nested_1() {
    libkernel::println!("{}", nested_2())
}

#[no_mangle]
unsafe fn kernel_init() -> ! {
    exception::handling_init();
    memory::init();
    bsp::driver::qemu_bring_up_console();

    nested_1();

    // The QEMU process running this test will be closed by the I/O test harness.
    cpu::wait_forever()
}
