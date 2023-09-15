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
    memory::{mmu::TranslationGranule, Address, Physical},
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
    #[inline(always)]
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
    #[inline(always)]
    fn configure_translation_control(&self) {
        let t1sz = (64 - bsp::memory::mmu::KernelVirtAddrSpace::SIZE_SHIFT) as u64;

        TCR_EL1.write(
            TCR_EL1::TBI1::Used
                + TCR_EL1::IPS::Bits_40
                + TCR_EL1::TG1::KiB_64
                + TCR_EL1::SH1::Inner
                + TCR_EL1::ORGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
                + TCR_EL1::IRGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
                + TCR_EL1::EPD1::EnableTTBR1Walks
                + TCR_EL1::A1::TTBR1
                + TCR_EL1::T1SZ.val(t1sz)
                + TCR_EL1::EPD0::DisableTTBR0Walks,
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
    unsafe fn enable_mmu_and_caching(
        &self,
        phys_tables_base_addr: Address<Physical>,
    ) -> Result<(), MMUEnableError> {
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

        // Set the "Translation Table Base Register".
        TTBR1_EL1.set_baddr(phys_tables_base_addr.as_usize() as u64);

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
