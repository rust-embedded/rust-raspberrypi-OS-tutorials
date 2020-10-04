// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2019-2020 Andre Richter <andre.o.richter@gmail.com>

//! Console sanity tests - RX, TX and statistics.

#![feature(format_args_nl)]
#![no_main]
#![no_std]

mod panic_exit_failure;

use libkernel::{bsp, console, print};

#[no_mangle]
unsafe fn kernel_init() -> ! {
    use bsp::console::{console, qemu_bring_up_console};
    use console::interface::*;

    qemu_bring_up_console();

    // Handshake
    assert_eq!(console().read_char(), 'A');
    assert_eq!(console().read_char(), 'B');
    assert_eq!(console().read_char(), 'C');
    print!("OK1234");

    // 6
    print!("{}", console().chars_written());

    // 3
    print!("{}", console().chars_read());

    // The QEMU process running this test will be closed by the I/O test harness.
    loop {}
}
