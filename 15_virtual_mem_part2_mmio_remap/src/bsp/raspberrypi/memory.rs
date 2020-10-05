// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! BSP Memory Management.
//!
//! The physical memory layout after the kernel has been loaded by the Raspberry's firmware, which
//! copies the binary to 0x8_0000:
//!
//! +---------------------------------------------+
//! |                                             | 0x0
//! | Unmapped                                    |
//! |                                             | 0x6_FFFF
//! +---------------------------------------------+
//! | BOOT_CORE_STACK_START                       | 0x7_0000
//! |                                             |            ^
//! | ...                                         |            | Stack growth direction
//! |                                             |            |
//! | BOOT_CORE_STACK_END_INCLUSIVE               | 0x7_FFFF
//! +---------------------------------------------+
//! | RO_START == BOOT_CORE_STACK_END             | 0x8_0000
//! |                                             |
//! |                                             |
//! | .text                                       |
//! | .exception_vectors                          |
//! | .rodata                                     |
//! |                                             |
//! | RO_END_INCLUSIVE                            | 0x8_0000 + __ro_size - 1
//! +---------------------------------------------+
//! | RO_END == DATA_START                        | 0x8_0000 + __ro_size
//! |                                             |
//! | .data                                       |
//! | .bss                                        |
//! |                                             |
//! | DATA_END_INCLUSIVE                          | 0x8_0000 + __ro_size + __data_size - 1
//! +---------------------------------------------+

pub mod mmu;

use crate::memory::mmu::{Address, Physical, Virtual};
use core::{cell::UnsafeCell, ops::RangeInclusive};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

// Symbols from the linker script.
extern "Rust" {
    static __bss_start: UnsafeCell<u64>;
    static __bss_end_inclusive: UnsafeCell<u64>;
    static __ro_start: UnsafeCell<()>;
    static __ro_size: UnsafeCell<()>;
    static __data_size: UnsafeCell<()>;
}

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// The board's physical memory map.
#[rustfmt::skip]
pub(super) mod map {
    use super::*;

    pub const BOOT_CORE_STACK_SIZE:                  usize = 0x1_0000;

    /// Physical devices.
    #[cfg(feature = "bsp_rpi3")]
    pub mod mmio {
        use super::*;

        pub const PERIPHERAL_IC_START: Address<Physical> = Address::new(0x3F00_B200);
        pub const PERIPHERAL_IC_SIZE:  usize             =              0x24;

        pub const GPIO_START:          Address<Physical> = Address::new(0x3F20_0000);
        pub const GPIO_SIZE:           usize             =              0xA0;

        pub const PL011_UART_START:    Address<Physical> = Address::new(0x3F20_1000);
        pub const PL011_UART_SIZE:     usize             =              0x48;

        pub const LOCAL_IC_START:      Address<Physical> = Address::new(0x4000_0000);
        pub const LOCAL_IC_SIZE:       usize             =              0x100;

        pub const END:                 Address<Physical> = Address::new(0x4001_0000);
    }

    /// Physical devices.
    #[cfg(feature = "bsp_rpi4")]
    pub mod mmio {
        use super::*;

        pub const GPIO_START:       Address<Physical> = Address::new(0xFE20_0000);
        pub const GPIO_SIZE:        usize             =              0xA0;

        pub const PL011_UART_START: Address<Physical> = Address::new(0xFE20_1000);
        pub const PL011_UART_SIZE:  usize             =              0x48;

        pub const GICD_START:       Address<Physical> = Address::new(0xFF84_1000);
        pub const GICD_SIZE:        usize             =              0x824;

        pub const GICC_START:       Address<Physical> = Address::new(0xFF84_2000);
        pub const GICC_SIZE:        usize             =              0x14;

        pub const END:              Address<Physical> = Address::new(0xFF85_0000);
    }

    pub const END: Address<Physical> = mmio::END;
}

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

/// Start address of the Read-Only (RO) range.
///
/// # Safety
///
/// - Value is provided by the linker script and must be trusted as-is.
#[inline(always)]
fn virt_ro_start() -> Address<Virtual> {
    Address::new(unsafe { __ro_start.get() as usize })
}

/// Size of the Read-Only (RO) range of the kernel binary.
///
/// # Safety
///
/// - Value is provided by the linker script and must be trusted as-is.
#[inline(always)]
fn ro_size() -> usize {
    unsafe { __ro_size.get() as usize }
}

/// Start address of the data range.
#[inline(always)]
fn virt_data_start() -> Address<Virtual> {
    virt_ro_start() + ro_size()
}

/// Size of the data range.
///
/// # Safety
///
/// - Value is provided by the linker script and must be trusted as-is.
#[inline(always)]
fn data_size() -> usize {
    unsafe { __data_size.get() as usize }
}

/// Start address of the boot core's stack.
#[inline(always)]
fn virt_boot_core_stack_start() -> Address<Virtual> {
    virt_ro_start() - map::BOOT_CORE_STACK_SIZE
}

/// Size of the boot core's stack.
#[inline(always)]
fn boot_core_stack_size() -> usize {
    map::BOOT_CORE_STACK_SIZE
}

/// Exclusive end address of the physical address space.
#[inline(always)]
fn phys_addr_space_end() -> Address<Physical> {
    map::END
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Exclusive end address of the boot core's stack.
#[inline(always)]
pub fn phys_boot_core_stack_end() -> Address<Physical> {
    // The binary is still identity mapped, so we don't need to convert here.
    let end = virt_boot_core_stack_start().into_usize() + boot_core_stack_size();
    Address::new(end)
}

/// Return the inclusive range spanning the .bss section.
///
/// # Safety
///
/// - Values are provided by the linker script and must be trusted as-is.
/// - The linker-provided addresses must be u64 aligned.
pub fn bss_range_inclusive() -> RangeInclusive<*mut u64> {
    unsafe { RangeInclusive::new(__bss_start.get(), __bss_end_inclusive.get()) }
}
