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

/// We are outsmarting the compiler here by using a trait as a layer of indirection. Because we are
/// generating PIC code, a static dispatch to `init()` would generate a relative jump from the
/// callee to `init()`. However, when calling `init()`, code just finished copying the binary to the
/// actual link-time address, and hence is still running at whatever location the previous loader
/// has put it. So we do not want a relative jump, because it would not jump to the relocated code.
///
/// By indirecting through a trait object, we can make use of the property that vtables store
/// absolute addresses. So calling `init()` this way will kick execution to the relocated binary.
pub trait RunTimeInit {
    /// Equivalent to `crt0` or `c0` code in C/C++ world. Clears the `bss` section, then jumps to
    /// kernel init code.
    ///
    /// # Safety
    ///
    /// - Only a single core must be active and running this function.
    unsafe fn runtime_init(&self) -> ! {
        zero_bss();

        crate::kernel_init()
    }
}

struct Traitor;
impl RunTimeInit for Traitor {}

/// Give the callee a `RunTimeInit` trait object.
pub fn get() -> &'static dyn RunTimeInit {
    &Traitor {}
}
