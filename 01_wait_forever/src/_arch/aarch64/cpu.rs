// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! Architectural processor code.

// Assembly counterpart to this file.
global_asm!(include_str!("cpu.S"));

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Pause execution on the core.
#[inline(always)]
pub fn wait_forever() -> ! {
    unsafe {
        loop {
            // nomem means that the asm code does not read or write to memory. By default the
            // compiler will assume that inline assembly can read or write any memory address that
            // is accessible to it (e.g. through a pointer passed as an operand, or a global).

            // nostack means that the asm code does not push any data onto the stack. This allows
            // the compiler to use optimizations such as the stack red zone on x86-64 to avoid stack
            // pointer adjustments.
            #[rustfmt::skip]
            asm!(
                "wfe",
                options(nomem, nostack, preserves_flags)
            );
        }
    }
}
