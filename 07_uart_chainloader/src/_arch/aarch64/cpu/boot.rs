// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2021 Andre Richter <andre.o.richter@gmail.com>

//! Architectural boot code.
//!
//! # Orientation
//!
//! Since arch modules are imported into generic modules using the path attribute, the path of this
//! file is:
//!
//! crate::cpu::boot::arch_boot

use crate::{bsp, cpu};
use cortex_a::regs::*;

//--------------------------------------------------------------------------------------------------
// Public Code
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

    if bsp::cpu::BOOT_CORE_ID == cpu::smp::core_id() {
        SP.set(bsp::memory::boot_core_stack_end() as u64);
        relocate::relocate_self()
    } else {
        // If not core0, infinitely wait for events.
        cpu::wait_forever()
    }
}
