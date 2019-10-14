// SPDX-License-Identifier: MIT
//
// Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>

//! Rust runtime initialization code.

/// Equivalent to `crt0` or `c0` code in C/C++ world. Clears the `bss` section,
/// then calls the kernel entry.
///
/// Called from `BSP` code.
///
/// # Safety
///
/// - Only a single core must be active and running this function.
pub unsafe fn init() -> ! {
    extern "C" {
        // Boundaries of the .bss section, provided by the linker script
        static mut __bss_start: u64;
        static mut __bss_end: u64;
    }

    // Zero out the .bss section
    r0::zero_bss(&mut __bss_start, &mut __bss_end);

    crate::kernel_entry()
}
