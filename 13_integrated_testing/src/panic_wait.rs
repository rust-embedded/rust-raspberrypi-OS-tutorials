// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! A panic handler that infinitely waits.

use crate::{arch, bsp};
use core::{fmt, panic::PanicInfo};

fn _panic_print(args: fmt::Arguments) {
    use fmt::Write;

    unsafe { bsp::panic_console_out().write_fmt(args).unwrap() };
}

/// Prints with a newline - only use from the panic handler.
///
/// Carbon copy from https://doc.rust-lang.org/src/std/macros.rs.html
#[macro_export]
macro_rules! panic_println {
    ($($arg:tt)*) => ({
        _panic_print(format_args_nl!($($arg)*));
    })
}

/// The point of exit for the "standard" (non-testing) `libkernel`.
///
/// This code will be used by the release kernel binary and the `integration tests`. It is linked
/// weakly, so that the integration tests can overload it to exit `QEMU` instead of spinning
/// forever.
///
/// This is one possible approach to solve the problem that `cargo` can not know who the consumer of
/// the library will be:
/// - The release kernel binary that should safely park the paniced core,
/// - or an `integration test` that is executed in QEMU, which should just exit QEMU.
#[cfg(not(test))]
#[linkage = "weak"]
#[no_mangle]
fn _panic_exit() -> ! {
    arch::wait_forever()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(args) = info.message() {
        panic_println!("\nKernel panic: {}", args);
    } else {
        panic_println!("\nKernel panic!");
    }

    _panic_exit()
}

//--------------------------------------------------------------------------------------------------
// Testing
//--------------------------------------------------------------------------------------------------

/// The point of exit when the library is compiled for testing.
#[cfg(test)]
#[no_mangle]
fn _panic_exit() -> ! {
    arch::qemu_exit_failure()
}
