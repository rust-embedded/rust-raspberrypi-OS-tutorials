// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! AArch64.

pub mod sync;
mod time;

use crate::{bsp, interface};
use cortex_a::{asm, regs::*};

/// The entry of the `kernel` binary.
///
/// The function must be named `_start`, because the linker is looking for this exact name.
///
/// # Safety
///
/// - Linker script must ensure to place this function at `0x80_000`.
#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    const CORE_MASK: u64 = 0x3;

    if bsp::BOOT_CORE_ID == MPIDR_EL1.get() & CORE_MASK {
        SP.set(bsp::BOOT_CORE_STACK_START);
        crate::runtime_init::runtime_init()
    } else {
        // If not core0, infinitely wait for events.
        wait_forever()
    }
}

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

static TIMER: time::Timer = time::Timer;

//--------------------------------------------------------------------------------------------------
// Implementation of the kernel's architecture abstraction code
//--------------------------------------------------------------------------------------------------

pub use asm::nop;

/// Spin for `n` cycles.
pub fn spin_for_cycles(n: usize) {
    for _ in 0..n {
        asm::nop();
    }
}

/// Return a reference to a `interface::time::TimeKeeper` implementation.
pub fn timer() -> &'static impl interface::time::Timer {
    &TIMER
}

/// Pause execution on the calling CPU core.
#[inline(always)]
pub fn wait_forever() -> ! {
    loop {
        asm::wfe()
    }
}
