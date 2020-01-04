// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! Rust runtime initialization code.

use crate::memory;
use core::ops::Range;

/// Return the range spanning the .bss section.
///
/// # Safety
///
/// - The symbol-provided addresses must be valid.
/// - The symbol-provided addresses must be usize aligned.
unsafe fn bss_range() -> Range<*mut usize> {
    extern "C" {
        // Boundaries of the .bss section, provided by linker script symbols.
        static mut __bss_start: usize;
        static mut __bss_end: usize;
    }

    Range {
        start: &mut __bss_start,
        end: &mut __bss_end,
    }
}

/// Zero out the .bss section.
///
/// # Safety
///
/// - Must only be called pre `kernel_init()`.
#[inline(always)]
unsafe fn zero_bss() {
    memory::zero_volatile(bss_range());
}

/// Equivalent to `crt0` or `c0` code in C/C++ world. Clears the `bss` section, then jumps to kernel
/// init code.
///
/// # Safety
///
/// - Only a single core must be active and running this function.
pub unsafe fn runtime_init() -> ! {
    extern "Rust" {
        fn kernel_init() -> !;
    }

    zero_bss();

    kernel_init()
}

//--------------------------------------------------------------------------------------------------
// Testing
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use test_macros::kernel_test;

    /// Check `bss` section layout.
    #[kernel_test]
    fn bss_section_is_sane() {
        use core::mem;

        let start = unsafe { bss_range().start } as *const _ as usize;
        let end = unsafe { bss_range().end } as *const _ as usize;

        assert_eq!(start % mem::size_of::<usize>(), 0);
        assert_eq!(end % mem::size_of::<usize>(), 0);
        assert!(end >= start);
    }
}
