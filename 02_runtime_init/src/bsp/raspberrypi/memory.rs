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
    static __bss_start: UnsafeCell<u64>;
    static __bss_end_inclusive: UnsafeCell<u64>;
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Return the inclusive range spanning the .bss section.
///
/// # Safety
///
/// - Values are provided by the linker script and must be trusted as-is.
/// - The linker-provided addresses must be u64 aligned.
pub fn bss_range_inclusive() -> RangeInclusive<*mut u64> {
    unsafe { RangeInclusive::new(__bss_start.get(), __bss_end_inclusive.get()) }
}
