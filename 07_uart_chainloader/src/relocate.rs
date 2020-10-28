// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! Relocation code.

use crate::{bsp, runtime_init};

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Relocates the own binary from `bsp::cpu::BOARD_DEFAULT_LOAD_ADDRESS` to the `__binary_start`
/// address from the linker script.
///
/// # Safety
///
/// - Only a single core must be active and running this function.
/// - Function must not use the `bss` section.
pub unsafe fn relocate_self() -> ! {
    let range = bsp::memory::binary_range_inclusive();
    let mut reloc_destination_addr = *range.start();
    let reloc_end_addr_inclusive = *range.end();

    // The address of where the previous firmware loaded us.
    let mut src_addr = bsp::memory::board_default_load_addr();

    // TODO Make it work for the case src_addr > reloc_addr as well.
    let diff = reloc_destination_addr as usize - src_addr as usize;

    // Copy the whole binary.
    //
    // This is essentially a `memcpy()` optimized for throughput by transferring in chunks of T.
    loop {
        core::ptr::write_volatile(reloc_destination_addr, core::ptr::read_volatile(src_addr));
        reloc_destination_addr = reloc_destination_addr.offset(1);
        src_addr = src_addr.offset(1);

        if reloc_destination_addr > reloc_end_addr_inclusive {
            break;
        }
    }

    let relocated_runtime_init_addr = runtime_init::runtime_init as *const () as usize + diff;
    let relocated_runtime_init: fn() -> ! =
        core::mem::transmute(relocated_runtime_init_addr as *const ());

    relocated_runtime_init()
}
