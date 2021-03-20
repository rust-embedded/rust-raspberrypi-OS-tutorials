// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020-2021 Andre Richter <andre.o.richter@gmail.com>

//! General purpose code.

/// Check if a value is aligned to a given size.
#[inline(always)]
pub const fn is_aligned(value: usize, alignment: usize) -> bool {
    assert!(alignment.is_power_of_two());

    (value & (alignment - 1)) == 0
}

/// Align down.
#[inline(always)]
pub const fn align_down(value: usize, alignment: usize) -> usize {
    assert!(alignment.is_power_of_two());

    value & !(alignment - 1)
}
