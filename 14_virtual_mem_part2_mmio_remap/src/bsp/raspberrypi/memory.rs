// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! BSP Memory Management.
//!
//! The physical memory layout after the kernel has been loaded by the Raspberry's firmware, which
//! copies the binary to 0x8_0000:
//!
//! +---------------------------------------------+
//! |                                             |
//! | Unmapped                                    |
//! |                                             |
//! +---------------------------------------------+
//! |                                             | rx_start @ 0x8_0000
//! | .text                                       |
//! | .rodata                                     |
//! | .got                                        |
//! |                                             | rx_end_inclusive
//! +---------------------------------------------+
//! |                                             | rw_start == rx_end
//! | .data                                       |
//! | .bss                                        |
//! |                                             | rw_end_inclusive
//! +---------------------------------------------+
//! |                                             | rw_end
//! | Unmapped Boot-core Stack Guard Page         |
//! |                                             |
//! +---------------------------------------------+
//! |                                             | boot_core_stack_start          ^
//! |                                             |                                | stack
//! | Boot-core Stack                             |                                | growth
//! |                                             |                                | direction
//! |                                             | boot_core_stack_end_inclusive  |
//! +---------------------------------------------+

pub mod mmu;

use crate::memory::{Address, Physical, Virtual};
use core::cell::UnsafeCell;

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

// Symbols from the linker script.
extern "Rust" {
    static __rx_start: UnsafeCell<()>;
    static __rx_end_exclusive: UnsafeCell<()>;

    static __rw_start: UnsafeCell<()>;
    static __rw_end_exclusive: UnsafeCell<()>;

    static __boot_core_stack_start: UnsafeCell<()>;
    static __boot_core_stack_end_exclusive: UnsafeCell<()>;

    static __boot_core_stack_guard_page_start: UnsafeCell<()>;
    static __boot_core_stack_guard_page_end_exclusive: UnsafeCell<()>;
}

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// The board's physical memory map.
#[rustfmt::skip]
pub(super) mod map {
    use super::*;

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

/// Start address of the Read+Execute (RX) range.
///
/// # Safety
///
/// - Value is provided by the linker script and must be trusted as-is.
#[inline(always)]
fn virt_rx_start() -> Address<Virtual> {
    Address::new(unsafe { __rx_start.get() as usize })
}

/// Size of the Read+Execute (RX) range.
///
/// # Safety
///
/// - Value is provided by the linker script and must be trusted as-is.
#[inline(always)]
fn rx_size() -> usize {
    unsafe { (__rx_end_exclusive.get() as usize) - (__rx_start.get() as usize) }
}

/// Start address of the Read+Write (RW) range.
#[inline(always)]
fn virt_rw_start() -> Address<Virtual> {
    Address::new(unsafe { __rw_start.get() as usize })
}

/// Size of the Read+Write (RW) range.
///
/// # Safety
///
/// - Value is provided by the linker script and must be trusted as-is.
#[inline(always)]
fn rw_size() -> usize {
    unsafe { (__rw_end_exclusive.get() as usize) - (__rw_start.get() as usize) }
}

/// Start address of the boot core's stack.
#[inline(always)]
fn virt_boot_core_stack_start() -> Address<Virtual> {
    Address::new(unsafe { __boot_core_stack_start.get() as usize })
}

/// Size of the boot core's stack.
#[inline(always)]
fn boot_core_stack_size() -> usize {
    unsafe {
        (__boot_core_stack_end_exclusive.get() as usize) - (__boot_core_stack_start.get() as usize)
    }
}

/// Exclusive end address of the physical address space.
#[inline(always)]
fn phys_addr_space_end() -> Address<Physical> {
    map::END
}
