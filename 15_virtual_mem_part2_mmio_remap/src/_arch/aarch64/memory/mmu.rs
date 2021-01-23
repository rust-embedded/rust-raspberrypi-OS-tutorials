// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

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
    memory::{
        mmu::{translation_table::KernelTranslationTable, TranslationGranule},
        Address, Physical,
    },
    synchronization::InitStateLock,
};
use cortex_a::{barrier, regs::*};

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

/// The min supported address space size.
pub const MIN_ADDR_SPACE_SIZE: usize = 1024 * 1024 * 1024; // 1 GiB

/// The max supported address space size.
pub const MAX_ADDR_SPACE_SIZE: usize = 8 * 1024 * 1024 * 1024; // 8 GiB

/// The supported address space size granule.
pub type AddrSpaceSizeGranule = Granule512MiB;

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
static KERNEL_TABLES: InitStateLock<KernelTranslationTable> =
    InitStateLock::new(KernelTranslationTable::new());

static MMU: MemoryManagementUnit = MemoryManagementUnit;

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

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
        let ips = ID_AA64MMFR0_EL1.read(ID_AA64MMFR0_EL1::PARange);
        let t0sz = (64 - bsp::memory::mmu::KernelVirtAddrSpaceSize::SHIFT) as u64;

        TCR_EL1.write(
            TCR_EL1::TBI0::Ignored
                + TCR_EL1::IPS.val(ips)
                + TCR_EL1::EPD1::DisableTTBR1Walks
                + TCR_EL1::TG0::KiB_64
                + TCR_EL1::SH0::Inner
                + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
                + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
                + TCR_EL1::EPD0::EnableTTBR0Walks
                + TCR_EL1::T0SZ.val(t0sz),
        );
    }
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Return a guarded reference to the kernel's translation tables.
pub fn kernel_translation_tables() -> &'static InitStateLock<KernelTranslationTable> {
    &KERNEL_TABLES
}

/// Return a reference to the MMU instance.
pub fn mmu() -> &'static impl memory::mmu::interface::MMU {
    &MMU
}

//------------------------------------------------------------------------------
// OS Interface Code
//------------------------------------------------------------------------------

impl memory::mmu::interface::MMU for MemoryManagementUnit {
    unsafe fn enable(
        &self,
        kernel_table_phys_base_addr: Address<Physical>,
    ) -> Result<(), &'static str> {
        // Fail early if translation granule is not supported. Both RPis support it, though.
        if !ID_AA64MMFR0_EL1.matches_all(ID_AA64MMFR0_EL1::TGran64::Supported) {
            return Err("Translation granule not supported in HW");
        }

        // Prepare the memory attribute indirection register.
        self.set_up_mair();

        // Set the "Translation Table Base Register".
        TTBR0_EL1.set_baddr(kernel_table_phys_base_addr.into_usize() as u64);

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
}

//--------------------------------------------------------------------------------------------------
// Testing
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use test_macros::kernel_test;

    /// Check if KERNEL_TABLES is in .bss.
    #[kernel_test]
    fn kernel_tables_in_bss() {
        let bss_range = bsp::memory::bss_range_inclusive();
        let kernel_tables_addr = &KERNEL_TABLES as *const _ as usize as *mut u64;

        assert!(bss_range.contains(&kernel_tables_addr));
    }
}
