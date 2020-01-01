// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! Relocation code.

/// Relocates the own binary from `bsp::BOARD_DEFAULT_LOAD_ADDRESS` to the `__binary_start` address
/// from the linker script.
///
/// # Safety
///
/// - Only a single core must be active and running this function.
/// - Function must not use the `bss` section.
pub unsafe fn relocate_self<T>() -> ! {
    extern "C" {
        static __binary_start: usize;
        static __binary_end: usize;
    }

    let binary_start_addr: usize = &__binary_start as *const _ as _;
    let binary_end_addr: usize = &__binary_end as *const _ as _;
    let binary_size_in_byte: usize = binary_end_addr - binary_start_addr;

    // Get the relocation destination address from the linker symbol.
    let mut reloc_dst_addr: *mut T = binary_start_addr as *mut T;

    // The address of where the previous firmware loaded us.
    let mut src_addr: *const T = crate::bsp::BOARD_DEFAULT_LOAD_ADDRESS as *const _;

    // Copy the whole binary.
    //
    // This is essentially a `memcpy()` optimized for throughput by transferring in chunks of T.
    let n = binary_size_in_byte / core::mem::size_of::<T>();
    for _ in 0..n {
        use core::ptr;

        ptr::write_volatile::<T>(reloc_dst_addr, ptr::read_volatile::<T>(src_addr));
        reloc_dst_addr = reloc_dst_addr.offset(1);
        src_addr = src_addr.offset(1);
    }

    // Call `init()` through a trait object, causing the jump to use an absolute address to reach
    // the relocated binary. An elaborate explanation can be found in the runtime_init.rs source
    // comments.
    crate::runtime_init::get().runtime_init()
}
