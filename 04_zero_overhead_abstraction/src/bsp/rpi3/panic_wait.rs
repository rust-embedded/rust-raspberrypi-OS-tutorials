// SPDX-License-Identifier: MIT
//
// Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>

//! A panic handler that infinitely waits.

use crate::println;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(args) = info.message() {
        println!("Kernel panic: {}", args);
    } else {
        println!("Kernel panic!");
    }

    super::wait_forever()
}