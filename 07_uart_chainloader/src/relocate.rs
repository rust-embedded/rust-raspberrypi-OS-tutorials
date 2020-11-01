// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! Relocation code.

use crate::{bsp, cpu};

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Relocates the own binary from `bsp::memory::board_default_load_addr()` to the `__binary_start`
/// address from the linker script.
///
/// # Safety
///
/// - Only a single core must be active and running this function.
/// - Function must not use the `bss` section.
#[inline(never)]
pub unsafe fn relocate_self() -> ! {
    let range = bsp::memory::relocated_binary_range_inclusive();
    let mut relocated_binary_start_addr = *range.start();
    let relocated_binary_end_addr_inclusive = *range.end();

    // The address of where the previous firmware loaded us.
    let mut current_binary_start_addr = bsp::memory::board_default_load_addr();

    // Copy the whole binary.
    while relocated_binary_start_addr <= relocated_binary_end_addr_inclusive {
        core::ptr::write_volatile(
            relocated_binary_start_addr,
            core::ptr::read_volatile(current_binary_start_addr),
        );
        relocated_binary_start_addr = relocated_binary_start_addr.offset(1);
        current_binary_start_addr = current_binary_start_addr.offset(1);
    }

    // The following function calls form a hack to achieve an "absolute jump" to
    // `runtime_init::runtime_init()` by forcing an indirection through the global offset table
    // (GOT), so that execution continues from the relocated binary.
    //
    // Without this, the address of `runtime_init()` would be calculated as a relative offset from
    // the current program counter, since we are compiling as `position independent code`. This
    // would cause us to keep executing from the address to which the firmware loaded us, instead of
    // the relocated position.
    //
    // There likely is a more elegant way to do this.
    let relocated_runtime_init_addr = bsp::memory::relocated_runtime_init_addr() as usize;
    cpu::branch_to_raw_addr(relocated_runtime_init_addr)
}
