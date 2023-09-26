// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020-2023 Andre Richter <andre.o.richter@gmail.com>

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

/// Align up.
#[inline(always)]
pub const fn align_up(value: usize, alignment: usize) -> usize {
    assert!(alignment.is_power_of_two());

    (value + alignment - 1) & !(alignment - 1)
}

/// Convert a size into human readable format.
pub const fn size_human_readable_ceil(size: usize) -> (usize, &'static str) {
    const KIB: usize = 1024;
    const MIB: usize = 1024 * 1024;
    const GIB: usize = 1024 * 1024 * 1024;

    if (size / GIB) > 0 {
        (size.div_ceil(GIB), "GiB")
    } else if (size / MIB) > 0 {
        (size.div_ceil(MIB), "MiB")
    } else if (size / KIB) > 0 {
        (size.div_ceil(KIB), "KiB")
    } else {
        (size, "Byte")
    }
}
