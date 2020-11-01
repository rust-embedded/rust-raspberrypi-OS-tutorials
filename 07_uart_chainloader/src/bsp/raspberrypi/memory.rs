// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! BSP Memory Management.

use core::{cell::UnsafeCell, ops::RangeInclusive};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

// Symbols from the linker script.
extern "Rust" {
    static __binary_start: UnsafeCell<u64>;
    static __binary_end_inclusive: UnsafeCell<u64>;
    static __runtime_init_reloc: UnsafeCell<u64>;
    static __bss_start: UnsafeCell<u64>;
    static __bss_end_inclusive: UnsafeCell<u64>;
}

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// The board's memory map.
#[rustfmt::skip]
pub(super) mod map {
    pub const BOOT_CORE_STACK_END:        usize =        0x8_0000;

    pub const BOARD_DEFAULT_LOAD_ADDRESS: usize =        0x8_0000;

    pub const GPIO_OFFSET:                usize =        0x0020_0000;
    pub const UART_OFFSET:                usize =        0x0020_1000;

    /// Physical devices.
    #[cfg(feature = "bsp_rpi3")]
    pub mod mmio {
        use super::*;

        pub const START:            usize =         0x3F00_0000;
        pub const GPIO_START:       usize = START + GPIO_OFFSET;
        pub const PL011_UART_START: usize = START + UART_OFFSET;
    }

    /// Physical devices.
    #[cfg(feature = "bsp_rpi4")]
    pub mod mmio {
        use super::*;

        pub const START:            usize =         0xFE00_0000;
        pub const GPIO_START:       usize = START + GPIO_OFFSET;
        pub const PL011_UART_START: usize = START + UART_OFFSET;
    }
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Exclusive end address of the boot core's stack.
#[inline(always)]
pub fn boot_core_stack_end() -> usize {
    map::BOOT_CORE_STACK_END
}

/// The address on which the Raspberry firmware loads every binary by default.
#[inline(always)]
pub fn board_default_load_addr() -> *const u64 {
    map::BOARD_DEFAULT_LOAD_ADDRESS as _
}

/// Return the inclusive range spanning the relocated kernel binary.
///
/// # Safety
///
/// - Values are provided by the linker script and must be trusted as-is.
/// - The linker-provided addresses must be u64 aligned.
pub fn relocated_binary_range_inclusive() -> RangeInclusive<*mut u64> {
    unsafe { RangeInclusive::new(__binary_start.get(), __binary_end_inclusive.get()) }
}

/// The relocated address of function `runtime_init()`.
#[inline(always)]
pub fn relocated_runtime_init_addr() -> *const u64 {
    unsafe { __runtime_init_reloc.get() as _ }
}

/// Return the inclusive range spanning the relocated .bss section.
///
/// # Safety
///
/// - Values are provided by the linker script and must be trusted as-is.
/// - The linker-provided addresses must be u64 aligned.
pub fn relocated_bss_range_inclusive() -> RangeInclusive<*mut u64> {
    let range;
    unsafe {
        range = RangeInclusive::new(__bss_start.get(), __bss_end_inclusive.get());
    }
    assert!(!range.is_empty());

    range
}
