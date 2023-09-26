// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>

//! System console.

mod buffer_console;

use crate::synchronization;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Console interfaces.
pub mod interface {
    use core::fmt;

    /// Console write functions.
    pub trait Write {
        /// Write a single character.
        fn write_char(&self, c: char);

        /// Write a slice of characters.
        fn write_array(&self, a: &[char]);

        /// Write a Rust format string.
        fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result;

        /// Block until the last buffered character has been physically put on the TX wire.
        fn flush(&self);
    }

    /// Console read functions.
    pub trait Read {
        /// Read a single character.
        fn read_char(&self) -> char {
            ' '
        }

        /// Clear RX buffers, if any.
        fn clear_rx(&self);
    }

    /// Console statistics.
    pub trait Statistics {
        /// Return the number of characters written.
        fn chars_written(&self) -> usize {
            0
        }

        /// Return the number of characters read.
        fn chars_read(&self) -> usize {
            0
        }
    }

    /// Trait alias for a full-fledged console.
    pub trait All: Write + Read + Statistics {}
}

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

static CUR_CONSOLE: InitStateLock<&'static (dyn interface::All + Sync)> =
    InitStateLock::new(&buffer_console::BUFFER_CONSOLE);

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------
use synchronization::{interface::ReadWriteEx, InitStateLock};

/// Register a new console.
pub fn register_console(new_console: &'static (dyn interface::All + Sync)) {
    CUR_CONSOLE.write(|con| *con = new_console);

    static FIRST_SWITCH: InitStateLock<bool> = InitStateLock::new(true);
    FIRST_SWITCH.write(|first| {
        if *first {
            *first = false;

            buffer_console::BUFFER_CONSOLE.dump();
        }
    });
}

/// Return a reference to the currently registered console.
///
/// This is the global console used by all printing macros.
pub fn console() -> &'static dyn interface::All {
    CUR_CONSOLE.read(|con| *con)
}
