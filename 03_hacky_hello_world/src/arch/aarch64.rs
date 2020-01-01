// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! AArch64.

global_asm!(include_str!("aarch64/start.S"));

//--------------------------------------------------------------------------------------------------
// Implementation of the kernel's architecture abstraction code
//--------------------------------------------------------------------------------------------------

/// Pause execution on the calling CPU core.
#[inline(always)]
pub fn wait_forever() -> ! {
    unsafe {
        loop {
            asm!("wfe" :::: "volatile")
        }
    }
}
