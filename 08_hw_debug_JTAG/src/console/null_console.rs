// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2022-2023 Andre Richter <andre.o.richter@gmail.com>

//! Null console.

use super::interface;
use core::fmt;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

pub struct NullConsole;

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

pub static NULL_CONSOLE: NullConsole = NullConsole {};

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

impl interface::Write for NullConsole {
    fn write_char(&self, _c: char) {}

    fn write_fmt(&self, _args: fmt::Arguments) -> fmt::Result {
        fmt::Result::Ok(())
    }

    fn flush(&self) {}
}

impl interface::Read for NullConsole {
    fn clear_rx(&self) {}
}

impl interface::Statistics for NullConsole {}
impl interface::All for NullConsole {}
