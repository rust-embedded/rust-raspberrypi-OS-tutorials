// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! Memory Management.

pub mod mmu;

use core::ops::RangeInclusive;

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Zero out an inclusive memory range.
///
/// # Safety
///
/// - `range.start` and `range.end` must be valid.
/// - `range.start` and `range.end` must be `T` aligned.
pub unsafe fn zero_volatile<T>(range: RangeInclusive<*mut T>)
where
    T: From<u8>,
{
    let mut ptr = *range.start();
    let end_inclusive = *range.end();

    while ptr <= end_inclusive {
        core::ptr::write_volatile(ptr, T::from(0));
        ptr = ptr.offset(1);
    }
}

//--------------------------------------------------------------------------------------------------
// Testing
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use test_macros::kernel_test;

    /// Check `zero_volatile()`.
    #[kernel_test]
    fn zero_volatile_works() {
        let mut x: [usize; 3] = [10, 11, 12];
        let x_range = x.as_mut_ptr_range();
        let x_range_inclusive =
            RangeInclusive::new(x_range.start, unsafe { x_range.end.offset(-1) });

        unsafe { zero_volatile(x_range_inclusive) };

        assert_eq!(x, [0, 0, 0]);
    }

    /// Check `bss` section layout.
    #[kernel_test]
    fn bss_section_is_sane() {
        use crate::bsp::memory::bss_range_inclusive;
        use core::mem;

        let start = *bss_range_inclusive().start() as usize;
        let end = *bss_range_inclusive().end() as usize;

        assert_eq!(start % mem::size_of::<usize>(), 0);
        assert_eq!(end % mem::size_of::<usize>(), 0);
        assert!(end >= start);
    }
}
