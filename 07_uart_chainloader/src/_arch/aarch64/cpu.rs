// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! Architectural processor code.

use crate::{bsp, cpu};
use cortex_a::{asm, regs::*};

//--------------------------------------------------------------------------------------------------
// Boot Code
//--------------------------------------------------------------------------------------------------

/// The entry of the `kernel` binary.
///
/// The function must be named `_start`, because the linker is looking for this exact name.
///
/// # Safety
///
/// - Linker script must ensure to place this function where it is expected by the target machine.
/// - We have to hope that the compiler omits any stack pointer usage before the stack pointer is
///   actually set (`SP.set()`).
#[no_mangle]
pub unsafe fn _start() -> ! {
    use crate::relocate;

    // Expect the boot core to start in EL2.
    if bsp::cpu::BOOT_CORE_ID == cpu::smp::core_id() {
        SP.set(bsp::memory::boot_core_stack_end() as u64);
        relocate::relocate_self()
    } else {
        // If not core0, infinitely wait for events.
        wait_forever()
    }
}

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
