// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2022 Andre Richter <andre.o.richter@gmail.com>

//! A panic handler that infinitely waits.

use crate::{bsp, cpu};
use core::{fmt, panic::PanicInfo};

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

fn _panic_print(args: fmt::Arguments) {
    use fmt::Write;

    unsafe { bsp::console::panic_console_out().write_fmt(args).unwrap() };
}

/// The point of exit for `libkernel`.
///
/// It is linked weakly, so that the integration tests can overload its standard behavior.
#[linkage = "weak"]
#[no_mangle]
fn _panic_exit() -> ! {
    #[cfg(not(feature = "test_build"))]
    {
        cpu::wait_forever()
    }

    #[cfg(feature = "test_build")]
    {
        cpu::qemu_exit_failure()
    }
}

/// Prints with a newline - only use from the panic handler.
///
/// Carbon copy from <https://doc.rust-lang.org/src/std/macros.rs.html>
#[macro_export]
macro_rules! panic_println {
    ($($arg:tt)*) => ({
        _panic_print(format_args_nl!($($arg)*));
    })
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use crate::time::interface::TimeManager;

    let timestamp = crate::time::time_manager().uptime();

    if let Some(args) = info.message() {
        panic_println!(
            "[  {:>3}.{:06}] Kernel panic: {}",
            timestamp.as_secs(),
            timestamp.subsec_micros(),
            args,
        );
    } else {
        panic_println!(
            "[  {:>3}.{:06}] Kernel panic!",
            timestamp.as_secs(),
            timestamp.subsec_micros(),
        );
    }

    _panic_exit()
}
