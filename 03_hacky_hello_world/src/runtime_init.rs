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
#[no_mangle]
pub unsafe extern "C" fn runtime_init() -> ! {
    zero_bss();

    crate::kernel_init()
}
