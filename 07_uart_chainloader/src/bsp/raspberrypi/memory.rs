// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! BSP Memory Management.

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// The early boot core's stack address.
pub const BOOT_CORE_STACK_START: usize = 0x80_000;

/// The address on which the Raspberry firmware loads every binary by default.
pub const BOARD_DEFAULT_LOAD_ADDRESS: usize = 0x80_000;

/// The board's memory map.
#[rustfmt::skip]
pub(super) mod map {
    pub const GPIO_OFFSET:         usize =        0x0020_0000;
    pub const UART_OFFSET:         usize =        0x0020_1000;

    /// Physical devices.
    #[cfg(feature = "bsp_rpi3")]
    pub mod mmio {
        use super::*;

        pub const BASE:            usize =        0x3F00_0000;
        pub const GPIO_BASE:       usize = BASE + GPIO_OFFSET;
        pub const PL011_UART_BASE: usize = BASE + UART_OFFSET;
    }

    /// Physical devices.
    #[cfg(feature = "bsp_rpi4")]
    pub mod mmio {
        use super::*;

        pub const BASE:            usize =        0xFE00_0000;
        pub const GPIO_BASE:       usize = BASE + GPIO_OFFSET;
        pub const PL011_UART_BASE: usize = BASE + UART_OFFSET;
    }
}
