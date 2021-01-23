// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! Architectural processor code.
//!
//! # Orientation
//!
//! Since arch modules are imported into generic modules using the path attribute, the path of this
//! file is:
//!
//! crate::cpu::arch_cpu

use cortex_a::asm;

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

pub use asm::nop;

/// Pause execution on the core.
#[inline(always)]
pub fn wait_forever() -> ! {
    loop {
        asm::wfe()
    }
}

//--------------------------------------------------------------------------------------------------
// Testing
//--------------------------------------------------------------------------------------------------
use qemu_exit::QEMUExit;

const QEMU_EXIT_HANDLE: qemu_exit::AArch64 = qemu_exit::AArch64::new();

/// Make the host QEMU binary execute `exit(1)`.
pub fn qemu_exit_failure() -> ! {
    QEMU_EXIT_HANDLE.exit_failure()
}

/// Make the host QEMU binary execute `exit(0)`.
pub fn qemu_exit_success() -> ! {
    QEMU_EXIT_HANDLE.exit_success()
}
