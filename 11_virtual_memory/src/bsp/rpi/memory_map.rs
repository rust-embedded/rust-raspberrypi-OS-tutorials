// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! The board's memory map.

#[cfg(feature = "bsp_rpi3")]
#[rustfmt::skip]
pub const END_INCLUSIVE:       usize =        0x3FFF_FFFF;

#[cfg(feature = "bsp_rpi4")]
#[rustfmt::skip]
pub const END_INCLUSIVE:       usize =        0xFFFF_FFFF;

/// Physical devices.
#[rustfmt::skip]
pub mod mmio {
    #[cfg(feature = "bsp_rpi3")]
    pub const BASE:            usize =        0x3F00_0000;

    #[cfg(feature = "bsp_rpi4")]
    pub const BASE:            usize =        0xFE00_0000;

    pub const GPIO_BASE:       usize = BASE + 0x0020_0000;
    pub const PL011_UART_BASE: usize = BASE + 0x0020_1000;
    pub const END_INCLUSIVE:   usize = super::END_INCLUSIVE;
}
