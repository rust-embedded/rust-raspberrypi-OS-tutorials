// SPDX-License-Identifier: MIT
//
// Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>

//! The board's memory map.

/// Physical devices.
#[rustfmt::skip]
pub mod mmio {
    pub const BASE:           usize =        0x3F00_0000;
    pub const GPIO_BASE:      usize = BASE + 0x0020_0000;
    pub const MINI_UART_BASE: usize = BASE + 0x0021_5000;
}
