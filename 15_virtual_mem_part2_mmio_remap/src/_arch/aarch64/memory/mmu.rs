// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! Memory Management Unit Driver.
//!
//! Only 64 KiB granule is supported.

use crate::{
    bsp,
    memory::{
        mmu,
        mmu::{
            AccessPermissions, Address, AddressType, AttributeFields, MemAttributes, Page,
            PageSliceDescriptor, Physical, Virtual,
        },
    },
    synchronization::InitStateLock,
};
use core::convert;
use cortex_a::{barrier, regs::*};
use register::{register_bitfields, InMemoryRegister};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------
use mmu::interface::TranslationGranule;

// A table descriptor, as per ARMv8-A Architecture Reference Manual Figure D5-15.
register_bitfields! {u64,
    STAGE1_TABLE_DESCRIPTOR [
        /// Physical address of the next descriptor.
        NEXT_LEVEL_TABLE_ADDR_64KiB OFFSET(16) NUMBITS(32) [], // [47:16]

        TYPE  OFFSET(1) NUMBITS(1) [
            Block = 0,
            Table = 1
        ],

        VALID OFFSET(0) NUMBITS(1) [
            False = 0,
            True = 1
        ]
    ]
}

// A level 3 page descriptor, as per ARMv8-A Architecture Reference Manual Figure D5-17.
register_bitfields! {u64,
    STAGE1_PAGE_DESCRIPTOR [
        /// Privileged execute-never.
        PXN      OFFSET(53) NUMBITS(1) [
            False = 0,
            True = 1
        ],

        /// Physical address of the next table descriptor (lvl2) or the page descriptor (lvl3).
        OUTPUT_ADDR_64KiB OFFSET(16) NUMBITS(32) [], // [47:16]

        /// Access flag.
        AF       OFFSET(10) NUMBITS(1) [
            False = 0,
            True = 1
        ],

        /// Shareability field.
        SH       OFFSET(8) NUMBITS(2) [
            OuterShareable = 0b10,
            InnerShareable = 0b11
        ],

        /// Access Permissions.
        AP       OFFSET(6) NUMBITS(2) [
            RW_EL1 = 0b00,
            RW_EL1_EL0 = 0b01,
            RO_EL1 = 0b10,
            RO_EL1_EL0 = 0b11
        ],

        /// Memory attributes index into the MAIR_EL1 register.
        AttrIndx OFFSET(2) NUMBITS(3) [],

        TYPE     OFFSET(1) NUMBITS(1) [
            Block = 0,
            Table = 1
        ],

        VALID    OFFSET(0) NUMBITS(1) [
            False = 0,
            True = 1
        ]
    ]
}

/// A table descriptor for 64 KiB aperture.
///
/// The output points to the next table.
#[derive(Copy, Clone)]
#[repr(transparent)]
struct TableDescriptor(u64);

/// A page descriptor with 64 KiB aperture.
///
/// The output points to physical memory.
#[derive(Copy, Clone)]
#[repr(transparent)]
struct PageDescriptor(u64);

#[derive(Copy, Clone)]
enum Granule512MiB {}

trait BaseAddr {
    fn phys_base_addr(&self) -> Address<Physical>;
}

/// Constants for indexing the MAIR_EL1.
#[allow(dead_code)]
mod mair {
    pub const DEVICE: u64 = 0;
    pub const NORMAL: u64 = 1;
}

/// Memory Management Unit type.
struct MemoryManagementUnit;

/// This constant is the power-of-two exponent that defines the virtual address space size.
///
/// Values tested and known to be working:
///   - 30 (1 GiB)
///   - 31 (2 GiB)
///   - 32 (4 GiB)
///   - 33 (8 GiB)
const ADDR_SPACE_SIZE_EXPONENT: usize = 33;

const NUM_LVL2_TABLES: usize = (1 << ADDR_SPACE_SIZE_EXPONENT) >> Granule512MiB::SHIFT;
const T0SZ: u64 = (64 - ADDR_SPACE_SIZE_EXPONENT) as u64;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Big monolithic struct for storing the translation tables. Individual levels must be 64 KiB
/// aligned, hence the "reverse" order of appearance.
#[repr(C)]
#[repr(align(65536))]
pub(in crate::memory::mmu) struct FixedSizeTranslationTable<const NUM_TABLES: usize> {
    /// Page descriptors, covering 64 KiB windows per entry.
    lvl3: [[PageDescriptor; 8192]; NUM_TABLES],

    /// Table descriptors, covering 512 MiB windows.
    lvl2: [TableDescriptor; NUM_TABLES],

    /// Index of the next free MMIO page.
    cur_l3_mmio_index: usize,

    /// Have the tables been initialized?
    initialized: bool,
}

pub(in crate::memory::mmu) type ArchTranslationTable = FixedSizeTranslationTable<NUM_LVL2_TABLES>;

// Supported translation granules are exported below, so that BSP code can pick between the options.
// This driver only supports 64 KiB at the moment.

#[derive(Copy, Clone)]
/// 64 KiB translation granule.
pub enum Granule64KiB {}

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

/// The translation tables.
///
/// # Safety
///
/// - Supposed to land in `.bss`. Therefore, ensure that all initial member values boil down to "0".
static KERNEL_TABLES: InitStateLock<ArchTranslationTable> =
    InitStateLock::new(ArchTranslationTable::new());

static MMU: MemoryManagementUnit = MemoryManagementUnit;

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

impl mmu::interface::TranslationGranule for Granule512MiB {
    const SIZE: usize = 512 * 1024 * 1024;
    const SHIFT: usize = 29; // log2(SIZE)
}

impl<T, const N: usize> BaseAddr for [T; N] {
    fn phys_base_addr(&self) -> Address<Physical> {
        // The binary is still identity mapped, so we don't need to convert here.
        Address::new(self as *const _ as usize)
    }
}

impl convert::From<usize> for TableDescriptor {
    fn from(next_lvl_table_addr: usize) -> Self {
        let val = InMemoryRegister::<u64, STAGE1_TABLE_DESCRIPTOR::Register>::new(0);

        let shifted = next_lvl_table_addr >> Granule64KiB::SHIFT;
        val.write(
            STAGE1_TABLE_DESCRIPTOR::VALID::True
                + STAGE1_TABLE_DESCRIPTOR::TYPE::Table
                + STAGE1_TABLE_DESCRIPTOR::NEXT_LEVEL_TABLE_ADDR_64KiB.val(shifted as u64),
        );

        TableDescriptor(val.get())
    }
}

/// Convert the kernel's generic memory attributes to HW-specific attributes of the MMU.
impl convert::From<AttributeFields>
    for register::FieldValue<u64, STAGE1_PAGE_DESCRIPTOR::Register>
{
    fn from(attribute_fields: AttributeFields) -> Self {
        // Memory attributes.
        let mut desc = match attribute_fields.mem_attributes {
            MemAttributes::CacheableDRAM => {
                STAGE1_PAGE_DESCRIPTOR::SH::InnerShareable
                    + STAGE1_PAGE_DESCRIPTOR::AttrIndx.val(mair::NORMAL)
            }
            MemAttributes::Device => {
                STAGE1_PAGE_DESCRIPTOR::SH::OuterShareable
                    + STAGE1_PAGE_DESCRIPTOR::AttrIndx.val(mair::DEVICE)
            }
        };

        // Access Permissions.
        desc += match attribute_fields.acc_perms {
            AccessPermissions::ReadOnly => STAGE1_PAGE_DESCRIPTOR::AP::RO_EL1,
            AccessPermissions::ReadWrite => STAGE1_PAGE_DESCRIPTOR::AP::RW_EL1,
        };

        // Execute Never.
        desc += if attribute_fields.execute_never {
            STAGE1_PAGE_DESCRIPTOR::PXN::True
        } else {
            STAGE1_PAGE_DESCRIPTOR::PXN::False
        };

        desc
    }
}

impl PageDescriptor {
    /// Create an instance.
    fn new(output_addr: *const Page<Physical>, attribute_fields: &AttributeFields) -> Self {
        let val = InMemoryRegister::<u64, STAGE1_PAGE_DESCRIPTOR::Register>::new(0);

        let shifted = output_addr as u64 >> Granule64KiB::SHIFT;
        val.write(
            STAGE1_PAGE_DESCRIPTOR::VALID::True
                + STAGE1_PAGE_DESCRIPTOR::AF::True
                + attribute_fields.clone().into()
                + STAGE1_PAGE_DESCRIPTOR::TYPE::Table
                + STAGE1_PAGE_DESCRIPTOR::OUTPUT_ADDR_64KiB.val(shifted),
        );

        Self(val.get())
    }

    /// Returns the valid bit.
    fn is_valid(&self) -> bool {
        InMemoryRegister::<u64, STAGE1_PAGE_DESCRIPTOR::Register>::new(self.0)
            .is_set(STAGE1_PAGE_DESCRIPTOR::VALID)
    }
}

impl<const NUM_TABLES: usize> FixedSizeTranslationTable<{ NUM_TABLES }> {
    // Reserve the last 256 MiB of the address space for MMIO mappings.
    const L2_MMIO_START_INDEX: usize = NUM_TABLES - 1;
    const L3_MMIO_START_INDEX: usize = 8192 / 2;

    /// Create an instance.
    pub const fn new() -> Self {
        assert!(NUM_TABLES > 0);

        Self {
            lvl3: [[PageDescriptor(0); 8192]; NUM_TABLES],
            lvl2: [TableDescriptor(0); NUM_TABLES],
            cur_l3_mmio_index: 0,
            initialized: false,
        }
    }

    /// The start address of the table's MMIO range.
    #[inline(always)]
    fn mmio_start_addr(&self) -> Address<Virtual> {
        Address::new(
            (Self::L2_MMIO_START_INDEX << Granule512MiB::SHIFT)
                | (Self::L3_MMIO_START_INDEX << Granule64KiB::SHIFT),
        )
    }

    /// The inclusive end address of the table's MMIO range.
    #[inline(always)]
    fn mmio_end_addr_inclusive(&self) -> Address<Virtual> {
        Address::new(
            (Self::L2_MMIO_START_INDEX << Granule512MiB::SHIFT)
                | (8191 << Granule64KiB::SHIFT)
                | (Granule64KiB::SIZE - 1),
        )
    }

    /// Helper to calculate the lvl2 and lvl3 indices from an address.
    #[inline(always)]
    fn lvl2_lvl3_index_from<ATYPE: AddressType>(
        &self,
        addr: *const Page<ATYPE>,
    ) -> Result<(usize, usize), &'static str> {
        let lvl2_index = addr as usize >> Granule512MiB::SHIFT;
        let lvl3_index = (addr as usize & Granule512MiB::MASK) >> Granule64KiB::SHIFT;

        if lvl2_index > (NUM_TABLES - 1) {
            return Err("Virtual page is out of bounds of translation table");
        }

        Ok((lvl2_index, lvl3_index))
    }

    /// Returns the PageDescriptor corresponding to the supplied Page.
    #[inline(always)]
    fn page_descriptor_from(
        &mut self,
        addr: *const Page<Virtual>,
    ) -> Result<&mut PageDescriptor, &'static str> {
        let (lvl2_index, lvl3_index) = self.lvl2_lvl3_index_from(addr)?;

        Ok(&mut self.lvl3[lvl2_index][lvl3_index])
    }
}

/// Setup function for the MAIR_EL1 register.
fn set_up_mair() {
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
fn configure_translation_control() {
    let ips = ID_AA64MMFR0_EL1.read(ID_AA64MMFR0_EL1::PARange);

    TCR_EL1.write(
        TCR_EL1::TBI0::Ignored
            + TCR_EL1::IPS.val(ips)
            + TCR_EL1::TG0::KiB_64
            + TCR_EL1::SH0::Inner
            + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
            + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
            + TCR_EL1::EPD0::EnableTTBR0Walks
            + TCR_EL1::T0SZ.val(T0SZ),
    );
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Return a guarded reference to the kernel's translation tables.
pub(in crate::memory::mmu) fn kernel_translation_tables(
) -> &'static InitStateLock<ArchTranslationTable> {
    &KERNEL_TABLES
}

/// Return a reference to the MMU instance.
pub(in crate::memory::mmu) fn mmu() -> &'static impl mmu::interface::MMU {
    &MMU
}

//------------------------------------------------------------------------------
// OS Interface Code
//------------------------------------------------------------------------------
impl mmu::interface::TranslationGranule for Granule64KiB {
    const SIZE: usize = 64 * 1024;
    const SHIFT: usize = 16; // log2(SIZE)
}

impl<const NUM_TABLES: usize> mmu::interface::TranslationTable
    for FixedSizeTranslationTable<{ NUM_TABLES }>
{
    unsafe fn init(&mut self) {
        if self.initialized {
            return;
        }

        // Populate the l2 entries.
        for (lvl2_nr, lvl2_entry) in self.lvl2.iter_mut().enumerate() {
            *lvl2_entry = self.lvl3[lvl2_nr].phys_base_addr().into_usize().into();
        }

        self.cur_l3_mmio_index = Self::L3_MMIO_START_INDEX;
        self.initialized = true;
    }

    fn phys_base_address(&self) -> Address<Physical> {
        self.lvl2.phys_base_addr()
    }

    unsafe fn map_pages_at(
        &mut self,
        phys_pages: &PageSliceDescriptor<Physical>,
        virt_pages: &PageSliceDescriptor<Virtual>,
        attr: &AttributeFields,
    ) -> Result<(), &'static str> {
        assert_eq!(self.initialized, true, "Translation tables not initialized");

        let p = phys_pages.as_slice();
        let v = virt_pages.as_slice();

        if p.len() != v.len() {
            return Err("Tried to map page slices with unequal sizes");
        }

        // No work to do for empty slices.
        if p.is_empty() {
            return Ok(());
        }

        if p.last().unwrap().as_ptr() >= bsp::memory::mmu::phys_addr_space_end_page() {
            return Err("Tried to map outside of physical address space");
        }

        let iter = p.iter().zip(v.iter());
        for (phys_page, virt_page) in iter {
            let page_descriptor = self.page_descriptor_from(virt_page.as_ptr())?;
            if page_descriptor.is_valid() {
                return Err("Virtual page is already mapped");
            }

            *page_descriptor = PageDescriptor::new(phys_page.as_ptr(), &attr);
        }

        Ok(())
    }

    fn next_mmio_virt_page_slice(
        &mut self,
        num_pages: usize,
    ) -> Result<PageSliceDescriptor<Virtual>, &'static str> {
        assert_eq!(self.initialized, true, "Translation tables not initialized");

        if num_pages == 0 {
            return Err("num_pages == 0");
        }

        if (self.cur_l3_mmio_index + num_pages) > 8191 {
            return Err("Not enough MMIO space left");
        }

        let addr = (Self::L2_MMIO_START_INDEX << Granule512MiB::SHIFT)
            | (self.cur_l3_mmio_index << Granule64KiB::SHIFT);
        self.cur_l3_mmio_index += num_pages;

        Ok(PageSliceDescriptor::from_addr(
            Address::new(addr),
            num_pages,
        ))
    }

    fn is_virt_page_slice_mmio(&self, virt_pages: &PageSliceDescriptor<Virtual>) -> bool {
        let start_addr = virt_pages.start_addr();
        let end_addr_inclusive = virt_pages.end_addr_inclusive();

        for i in [start_addr, end_addr_inclusive].iter() {
            if (*i >= self.mmio_start_addr()) && (*i <= self.mmio_end_addr_inclusive()) {
                return true;
            }
        }

        false
    }
}

impl mmu::interface::MMU for MemoryManagementUnit {
    unsafe fn enable(
        &self,
        phys_kernel_table_base_addr: Address<Physical>,
    ) -> Result<(), &'static str> {
        // Fail early if translation granule is not supported. Both RPis support it, though.
        if !ID_AA64MMFR0_EL1.matches_all(ID_AA64MMFR0_EL1::TGran64::Supported) {
            return Err("Translation granule not supported in HW");
        }

        // Prepare the memory attribute indirection register.
        set_up_mair();

        // Set the "Translation Table Base Register".
        TTBR0_EL1.set_baddr(phys_kernel_table_base_addr.into_usize() as u64);

        configure_translation_control();

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
pub(in crate::memory::mmu) type MinSizeArchTranslationTable = FixedSizeTranslationTable<1>;

#[cfg(test)]
mod tests {
    use super::*;
    use test_macros::kernel_test;

    /// Check if the size of `struct TableDescriptor` is as expected.
    #[kernel_test]
    fn size_of_tabledescriptor_equals_64_bit() {
        assert_eq!(
            core::mem::size_of::<TableDescriptor>(),
            core::mem::size_of::<u64>()
        );
    }

    /// Check if the size of `struct PageDescriptor` is as expected.
    #[kernel_test]
    fn size_of_pagedescriptor_equals_64_bit() {
        assert_eq!(
            core::mem::size_of::<PageDescriptor>(),
            core::mem::size_of::<u64>()
        );
    }

    /// Check if KERNEL_TABLES is in .bss.
    #[kernel_test]
    fn kernel_tables_in_bss() {
        let bss_range = bsp::memory::bss_range_inclusive();
        let kernel_tables_addr = &KERNEL_TABLES as *const _ as usize as *mut u64;

        assert!(bss_range.contains(&kernel_tables_addr));
    }
}
