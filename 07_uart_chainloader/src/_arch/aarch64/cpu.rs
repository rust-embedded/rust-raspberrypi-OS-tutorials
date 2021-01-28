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

/// Spin for `n` cycles.
#[cfg(feature = "bsp_rpi3")]
#[inline(always)]
pub fn spin_for_cycles(n: usize) {
    for _ in 0..n {
        asm::nop();
    }
}

/// Pause execution on the core.
#[inline(always)]
pub fn wait_forever() -> ! {
    loop {
        asm::wfe()
    }
}

/// Branch to a raw integer value.
///
/// # Safety
///
/// - This is highly unsafe. Use with care.
#[inline(always)]
pub unsafe fn branch_to_raw_addr(addr: usize) -> ! {
    asm!(
        "blr {destination:x}",
        destination = in(reg) addr,
        options(nomem, nostack)
    );

    core::intrinsics::unreachable()
}
