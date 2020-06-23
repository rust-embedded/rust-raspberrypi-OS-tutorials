// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020 Ken Kawamoto <kentaro.kawamoto@gmail.com>

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
            llvm_asm!("wfi"
                    :             // outputs
                    :             // inputs
                    :             // clobbers
                    : "volatile") // options
        }
    }
}
