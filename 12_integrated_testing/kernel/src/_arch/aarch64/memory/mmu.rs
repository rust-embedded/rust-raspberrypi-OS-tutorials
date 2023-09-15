// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>

//! Memory Management Unit Driver.
//!
//! Only 64 KiB granule is supported.
//!
//! # Orientation
//!
//! Since arch modules are imported into generic modules using the path attribute, the path of this
//! file is:
//!
//! crate::memory::mmu::arch_mmu

use crate::{
    bsp, memory,
    memory::mmu::{translation_table::KernelTranslationTable, TranslationGranule},
};
use aarch64_cpu::{asm::barrier, registers::*};
use core::intrinsics::unlikely;
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

/// Memory Management Unit type.
struct MemoryManagementUnit;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

pub type Granule512MiB = TranslationGranule<{ 512 * 1024 * 1024 }>;
pub type Granule64KiB = TranslationGranule<{ 64 * 1024 }>;

/// Constants for indexing the MAIR_EL1.
#[allow(dead_code)]
pub mod mair {
    pub const DEVICE: u64 = 0;
    pub const NORMAL: u64 = 1;
}

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

/// The kernel translation tables.
///
/// # Safety
///
/// - Supposed to land in `.bss`. Therefore, ensure that all initial member values boil down to "0".
static mut KERNEL_TABLES: KernelTranslationTable = KernelTranslationTable::new();

static MMU: MemoryManagementUnit = MemoryManagementUnit;

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

impl<const AS_SIZE: usize> memory::mmu::AddressSpace<AS_SIZE> {
    /// Checks for architectural restrictions.
    pub const fn arch_address_space_size_sanity_checks() {
        // Size must be at least one full 512 MiB table.
        assert!((AS_SIZE % Granule512MiB::SIZE) == 0);

        // Check for 48 bit virtual address size as maximum, which is supported by any ARMv8
        // version.
        assert!(AS_SIZE <= (1 << 48));
    }
}

impl MemoryManagementUnit {
    /// Setup function for the MAIR_EL1 register.
    fn set_up_mair(&self) {
        // Define the memory types being mapped.
        MAIR_EL1.write(
            // Attribute 1 - Cacheable normal DRAM.
            MAIR_EL1::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc +
        MAIR_EL1::Attr1_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc +

        // Attribute 0 - Device.
        MAIR_EL1::Attr0_Device::nonGathering_nonReordering_EarlyWriteAck,
        );
    }

    /// Configure various settings of stage 1 of the EL1 translation regime.
    fn configure_translation_control(&self) {
        let t0sz = (64 - bsp::memory::mmu::KernelAddrSpace::SIZE_SHIFT) as u64;

        TCR_EL1.write(
            TCR_EL1::TBI0::Used
                + TCR_EL1::IPS::Bits_40
                + TCR_EL1::TG0::KiB_64
                + TCR_EL1::SH0::Inner
                + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
                + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
                + TCR_EL1::EPD0::EnableTTBR0Walks
                + TCR_EL1::A1::TTBR0
                + TCR_EL1::T0SZ.val(t0sz)
                + TCR_EL1::EPD1::DisableTTBR1Walks,
        );
    }
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Return a reference to the MMU instance.
pub fn mmu() -> &'static impl memory::mmu::interface::MMU {
    &MMU
}

//------------------------------------------------------------------------------
// OS Interface Code
//------------------------------------------------------------------------------
use memory::mmu::MMUEnableError;

impl memory::mmu::interface::MMU for MemoryManagementUnit {
    unsafe fn enable_mmu_and_caching(&self) -> Result<(), MMUEnableError> {
        if unlikely(self.is_enabled()) {
            return Err(MMUEnableError::AlreadyEnabled);
        }

        // Fail early if translation granule is not supported.
        if unlikely(!ID_AA64MMFR0_EL1.matches_all(ID_AA64MMFR0_EL1::TGran64::Supported)) {
            return Err(MMUEnableError::Other(
                "Translation granule not supported in HW",
            ));
        }

        // Prepare the memory attribute indirection register.
        self.set_up_mair();

        // Populate translation tables.
        KERNEL_TABLES
            .populate_tt_entries()
            .map_err(MMUEnableError::Other)?;

        // Set the "Translation Table Base Register".
        TTBR0_EL1.set_baddr(KERNEL_TABLES.phys_base_address());

        self.configure_translation_control();

        // Switch the MMU on.
        //
        // First, force all previous changes to be seen before the MMU is enabled.
        barrier::isb(barrier::SY);

        // Enable the MMU and turn on data and instruction caching.
        SCTLR_EL1.modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable);

        // Force MMU init to complete before next instruction.
        barrier::isb(barrier::SY);

        Ok(())
    }

    #[inline(always)]
    fn is_enabled(&self) -> bool {
        SCTLR_EL1.matches_all(SCTLR_EL1::M::Enable)
    }
}

//--------------------------------------------------------------------------------------------------
// Testing
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use core::{cell::UnsafeCell, ops::Range};
    use test_macros::kernel_test;

    /// Check if KERNEL_TABLES is in .bss.
    #[kernel_test]
    fn kernel_tables_in_bss() {
        extern "Rust" {
            static __bss_start: UnsafeCell<u64>;
            static __bss_end_exclusive: UnsafeCell<u64>;
        }

        let bss_range = unsafe {
            Range {
                start: __bss_start.get(),
                end: __bss_end_exclusive.get(),
            }
        };
        let kernel_tables_addr = unsafe { &KERNEL_TABLES as *const _ as usize as *mut u64 };

        assert!(bss_range.contains(&kernel_tables_addr));
    }
}
