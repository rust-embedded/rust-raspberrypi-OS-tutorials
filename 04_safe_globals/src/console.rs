// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2022 Andre Richter <andre.o.richter@gmail.com>

//! System console.

use crate::bsp;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Console interfaces.
pub mod interface {
    use core::fmt;

    /// Console write functions.
    pub trait Write {
        /// Write a Rust format string.
        fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result;
    }

    /// Console statistics.
    pub trait Statistics {
        /// Return the number of characters written.
        fn chars_written(&self) -> usize {
            0
        }
    }

    /// Trait alias for a full-fledged console.
    pub trait All: Write + Statistics {}
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Return a reference to the console.
///
/// This is the global console used by all printing macros.
pub fn console() -> &'static dyn interface::All {
    bsp::console::console()
}
