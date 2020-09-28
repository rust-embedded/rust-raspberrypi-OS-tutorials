// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! BSP Memory Management.

use core::ops::Range;

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

// Symbols from the linker script.
extern "C" {
    static __bss_start: usize;
    static __bss_end: usize;
}

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// The early boot core's stack address.
pub const BOOT_CORE_STACK_START: usize = 0x80_000;

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

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Return the range spanning the .bss section.
///
/// # Safety
///
/// - Values are provided by the linker script and must be trusted as-is.
/// - The linker-provided addresses must be u64 aligned.
pub fn bss_range() -> Range<*mut u64> {
    unsafe {
        Range {
            start: &__bss_start as *const _ as *mut u64,
            end: &__bss_end as *const _ as *mut u64,
        }
    }
}
