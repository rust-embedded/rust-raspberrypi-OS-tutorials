// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2022 Andre Richter <andre.o.richter@gmail.com>

//! BSP console facilities.

use crate::console;

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Return a reference to the console.
pub fn console() -> &'static dyn console::interface::All {
    &super::driver::PL011_UART
}
