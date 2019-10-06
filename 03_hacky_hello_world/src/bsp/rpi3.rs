// SPDX-License-Identifier: MIT
//
// Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>

//! Board Support Package for the Raspberry Pi 3.

mod panic_wait;

use crate::interface;
use core::fmt;

global_asm!(include_str!("rpi3/start.S"));

/// A mystical, magical device for generating QEMU output out of the void.
struct QEMUOutput;

/// Implementing `console::Write` enables usage of the `format_args!` macros,
/// which in turn are used to implement the `kernel`'s `print!` and `println!`
/// macros.
///
/// See [`src/print.rs`].
///
/// [`src/print.rs`]: ../../print/index.html
impl interface::console::Write for QEMUOutput {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            unsafe {
                core::ptr::write_volatile(0x3F21_5040 as *mut u8, c as u8);
            }
        }

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
// Implementation of the kernel's BSP calls
////////////////////////////////////////////////////////////////////////////////

/// Returns a ready-to-use `console::Write` implementation.
pub fn console() -> impl interface::console::Write {
    QEMUOutput {}
}
