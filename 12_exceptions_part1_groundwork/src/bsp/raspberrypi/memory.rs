// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! BSP Memory Management.

pub mod mmu;

use core::ops::Range;

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

// Symbols from the linker script.
extern "C" {
    static __ro_start: usize;
    static __ro_end: usize;
    static __bss_start: usize;
    static __bss_end: usize;
}

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// The board's memory map.
#[rustfmt::skip]
pub(super) mod map {
    pub const END_INCLUSIVE:       usize =        0xFFFF_FFFF;

    pub const BOOT_CORE_STACK_END: usize =        0x8_0000;

    pub const GPIO_OFFSET:         usize =        0x0020_0000;
    pub const UART_OFFSET:         usize =        0x0020_1000;

    /// Physical devices.
    #[cfg(feature = "bsp_rpi3")]
    pub mod mmio {
        use super::*;

        pub const BASE:            usize =        0x3F00_0000;
        pub const GPIO_BASE:       usize = BASE + GPIO_OFFSET;
        pub const PL011_UART_BASE: usize = BASE + UART_OFFSET;
        pub const END_INCLUSIVE:   usize =        0x4000_FFFF;
    }

    /// Physical devices.
    #[cfg(feature = "bsp_rpi4")]
    pub mod mmio {
        use super::*;

        pub const BASE:            usize =        0xFE00_0000;
        pub const GPIO_BASE:       usize = BASE + GPIO_OFFSET;
        pub const PL011_UART_BASE: usize = BASE + UART_OFFSET;
        pub const END_INCLUSIVE:   usize =        0xFF84_FFFF;
    }
}

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

/// Start address of the Read-Only (RO) range.
///
/// # Safety
///
/// - Value is provided by the linker script and must be trusted as-is.
#[inline(always)]
fn ro_start() -> usize {
    unsafe { &__ro_start as *const _ as usize }
}

/// Size of the Read-Only (RO) range of the kernel binary.
///
/// # Safety
///
/// - Value is provided by the linker script and must be trusted as-is.
#[inline(always)]
fn ro_end() -> usize {
    unsafe { &__ro_end as *const _ as usize }
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Exclusive end address of the boot core's stack.
#[inline(always)]
pub fn boot_core_stack_end() -> usize {
    map::BOOT_CORE_STACK_END
}

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
