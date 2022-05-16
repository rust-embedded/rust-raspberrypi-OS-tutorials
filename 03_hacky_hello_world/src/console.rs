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
    /// Console write functions.
    ///
    /// `core::fmt::Write` is exactly what we need for now. Re-export it here because
    /// implementing `console::Write` gives a better hint to the reader about the
    /// intention.
    pub use core::fmt::Write;
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Return a reference to the console.
///
/// This is the global console used by all printing macros.
pub fn console() -> impl interface::Write {
    bsp::console::console()
}
