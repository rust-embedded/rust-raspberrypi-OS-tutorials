// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2021 Andre Richter <andre.o.richter@gmail.com>

//! Architectural translation table.
//!
//! Only 64 KiB granule is supported.
//!
//! # Orientation
//!
//! Since arch modules are imported into generic modules using the path attribute, the path of this
//! file is:
//!
//! crate::memory::mmu::translation_table::arch_translation_table

use crate::{
    bsp, memory,
    memory::{
        mmu::{
            arch_mmu::{Granule512MiB, Granule64KiB},
            AccessPermissions, AttributeFields, MemAttributes, Page, PageSliceDescriptor,
        },
        Address, Physical, Virtual,
    },
};
use core::convert;
use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields,
    registers::InMemoryRegister,
};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

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
        /// Unprivileged execute-never.
        UXN      OFFSET(54) NUMBITS(1) [
            False = 0,
            True = 1
        ],

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
            Reserved_Invalid = 0,
            Page = 1
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
#[repr(C)]
struct TableDescriptor {
    value: u64,
}

/// A page descriptor with 64 KiB aperture.
///
/// The output points to physical memory.
#[derive(Copy, Clone)]
#[repr(C)]
struct PageDescriptor {
    value: u64,
}

trait StartAddr {
    fn virt_start_addr(&self) -> Address<Virtual>;
}

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Big monolithic struct for storing the translation tables. Individual levels must be 64 KiB
/// aligned, so the lvl3 is put first.
#[repr(C)]
#[repr(align(65536))]
pub struct FixedSizeTranslationTable<const NUM_TABLES: usize> {
    /// Page descriptors, covering 64 KiB windows per entry.
    lvl3: [[PageDescriptor; 8192]; NUM_TABLES],

    /// Table descriptors, covering 512 MiB windows.
    lvl2: [TableDescriptor; NUM_TABLES],

    /// Index of the next free MMIO page.
    cur_l3_mmio_index: usize,

    /// Have the tables been initialized?
    initialized: bool,
}

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

impl<T, const N: usize> StartAddr for [T; N] {
    fn virt_start_addr(&self) -> Address<Virtual> {
        Address::new(self as *const _ as usize)
    }
}

impl TableDescriptor {
    /// Create an instance.
    ///
    /// Descriptor is invalid by default.
    pub const fn new_zeroed() -> Self {
        Self { value: 0 }
    }

    /// Create an instance pointing to the supplied address.
    pub fn from_next_lvl_table_addr(phys_next_lvl_table_addr: Address<Physical>) -> Self {
        let val = InMemoryRegister::<u64, STAGE1_TABLE_DESCRIPTOR::Register>::new(0);

        let shifted = phys_next_lvl_table_addr.into_usize() >> Granule64KiB::SHIFT;
        val.write(
            STAGE1_TABLE_DESCRIPTOR::NEXT_LEVEL_TABLE_ADDR_64KiB.val(shifted as u64)
                + STAGE1_TABLE_DESCRIPTOR::TYPE::Table
                + STAGE1_TABLE_DESCRIPTOR::VALID::True,
        );

        TableDescriptor { value: val.get() }
    }
}

/// Convert the kernel's generic memory attributes to HW-specific attributes of the MMU.
impl convert::From<AttributeFields>
    for tock_registers::fields::FieldValue<u64, STAGE1_PAGE_DESCRIPTOR::Register>
{
    fn from(attribute_fields: AttributeFields) -> Self {
        // Memory attributes.
        let mut desc = match attribute_fields.mem_attributes {
            MemAttributes::CacheableDRAM => {
                STAGE1_PAGE_DESCRIPTOR::SH::InnerShareable
                    + STAGE1_PAGE_DESCRIPTOR::AttrIndx.val(memory::mmu::arch_mmu::mair::NORMAL)
            }
            MemAttributes::Device => {
                STAGE1_PAGE_DESCRIPTOR::SH::OuterShareable
                    + STAGE1_PAGE_DESCRIPTOR::AttrIndx.val(memory::mmu::arch_mmu::mair::DEVICE)
            }
        };

        // Access Permissions.
        desc += match attribute_fields.acc_perms {
            AccessPermissions::ReadOnly => STAGE1_PAGE_DESCRIPTOR::AP::RO_EL1,
            AccessPermissions::ReadWrite => STAGE1_PAGE_DESCRIPTOR::AP::RW_EL1,
        };

        // The execute-never attribute is mapped to PXN in AArch64.
        desc += if attribute_fields.execute_never {
            STAGE1_PAGE_DESCRIPTOR::PXN::True
        } else {
            STAGE1_PAGE_DESCRIPTOR::PXN::False
        };

        // Always set unprivileged exectue-never as long as userspace is not implemented yet.
        desc += STAGE1_PAGE_DESCRIPTOR::UXN::True;

        desc
    }
}

/// Convert the HW-specific attributes of the MMU to kernel's generic memory attributes.
impl convert::TryFrom<InMemoryRegister<u64, STAGE1_PAGE_DESCRIPTOR::Register>> for AttributeFields {
    type Error = &'static str;

    fn try_from(
        desc: InMemoryRegister<u64, STAGE1_PAGE_DESCRIPTOR::Register>,
    ) -> Result<AttributeFields, Self::Error> {
        let mem_attributes = match desc.read(STAGE1_PAGE_DESCRIPTOR::AttrIndx) {
            memory::mmu::arch_mmu::mair::NORMAL => MemAttributes::CacheableDRAM,
            memory::mmu::arch_mmu::mair::DEVICE => MemAttributes::Device,
            _ => return Err("Unexpected memory attribute"),
        };

        let acc_perms = match desc.read_as_enum(STAGE1_PAGE_DESCRIPTOR::AP) {
            Some(STAGE1_PAGE_DESCRIPTOR::AP::Value::RO_EL1) => AccessPermissions::ReadOnly,
            Some(STAGE1_PAGE_DESCRIPTOR::AP::Value::RW_EL1) => AccessPermissions::ReadWrite,
            _ => return Err("Unexpected access permission"),
        };

        let execute_never = desc.read(STAGE1_PAGE_DESCRIPTOR::PXN) > 0;

        Ok(AttributeFields {
            mem_attributes,
            acc_perms,
            execute_never,
        })
    }
}

impl PageDescriptor {
    /// Create an instance.
    ///
    /// Descriptor is invalid by default.
    pub const fn new_zeroed() -> Self {
        Self { value: 0 }
    }

    /// Create an instance.
    pub fn from_output_page_ptr(
        phys_output_page_ptr: *const Page<Physical>,
        attribute_fields: &AttributeFields,
    ) -> Self {
        let val = InMemoryRegister::<u64, STAGE1_PAGE_DESCRIPTOR::Register>::new(0);

        let shifted = phys_output_page_ptr as u64 >> Granule64KiB::SHIFT;
        val.write(
            STAGE1_PAGE_DESCRIPTOR::OUTPUT_ADDR_64KiB.val(shifted)
                + STAGE1_PAGE_DESCRIPTOR::AF::True
                + STAGE1_PAGE_DESCRIPTOR::TYPE::Page
                + STAGE1_PAGE_DESCRIPTOR::VALID::True
                + (*attribute_fields).into(),
        );

        Self { value: val.get() }
    }

    /// Returns the valid bit.
    fn is_valid(&self) -> bool {
        InMemoryRegister::<u64, STAGE1_PAGE_DESCRIPTOR::Register>::new(self.value)
            .is_set(STAGE1_PAGE_DESCRIPTOR::VALID)
    }

    /// Returns the output page.
    fn output_page_ptr(&self) -> *const Page<Physical> {
        let shifted = InMemoryRegister::<u64, STAGE1_PAGE_DESCRIPTOR::Register>::new(self.value)
            .read(STAGE1_PAGE_DESCRIPTOR::OUTPUT_ADDR_64KiB);
        let addr = shifted << Granule64KiB::SHIFT;

        addr as *const Page<Physical>
    }

    /// Returns the attributes.
    fn try_attributes(&self) -> Result<AttributeFields, &'static str> {
        InMemoryRegister::<u64, STAGE1_PAGE_DESCRIPTOR::Register>::new(self.value).try_into()
    }
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

impl<const AS_SIZE: usize> memory::mmu::AssociatedTranslationTable
    for memory::mmu::AddressSpace<AS_SIZE>
where
    [u8; Self::SIZE >> Granule512MiB::SHIFT]: Sized,
{
    type TableStartFromBottom = FixedSizeTranslationTable<{ Self::SIZE >> Granule512MiB::SHIFT }>;
}

impl<const NUM_TABLES: usize> FixedSizeTranslationTable<NUM_TABLES> {
    // Reserve the last 256 MiB of the address space for MMIO mappings.
    const L2_MMIO_START_INDEX: usize = NUM_TABLES - 1;
    const L3_MMIO_START_INDEX: usize = 8192 / 2;

    /// Create an instance.
    #[allow(clippy::assertions_on_constants)]
    const fn _new(for_precompute: bool) -> Self {
        assert!(bsp::memory::mmu::KernelGranule::SIZE == Granule64KiB::SIZE);

        // Can't have a zero-sized address space.
        assert!(NUM_TABLES > 0);

        Self {
            lvl3: [[PageDescriptor::new_zeroed(); 8192]; NUM_TABLES],
            lvl2: [TableDescriptor::new_zeroed(); NUM_TABLES],
            cur_l3_mmio_index: Self::L3_MMIO_START_INDEX,
            initialized: for_precompute,
        }
    }

    pub const fn new_for_precompute() -> Self {
        Self::_new(true)
    }

    #[cfg(test)]
    pub fn new_for_runtime() -> Self {
        Self::_new(false)
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
    fn lvl2_lvl3_index_from_page_ptr(
        &self,
        virt_page_ptr: *const Page<Virtual>,
    ) -> Result<(usize, usize), &'static str> {
        let addr = virt_page_ptr as usize;
        let lvl2_index = addr >> Granule512MiB::SHIFT;
        let lvl3_index = (addr & Granule512MiB::MASK) >> Granule64KiB::SHIFT;

        if lvl2_index > (NUM_TABLES - 1) {
            return Err("Virtual page is out of bounds of translation table");
        }

        Ok((lvl2_index, lvl3_index))
    }

    /// Returns the PageDescriptor corresponding to the supplied page address.
    #[inline(always)]
    fn page_descriptor_from_page_ptr(
        &self,
        virt_page_ptr: *const Page<Virtual>,
    ) -> Result<&PageDescriptor, &'static str> {
        let (lvl2_index, lvl3_index) = self.lvl2_lvl3_index_from_page_ptr(virt_page_ptr)?;
        let desc = &self.lvl3[lvl2_index][lvl3_index];

        Ok(desc)
    }

    /// Sets the PageDescriptor corresponding to the supplied page address.
    ///
    /// Doesn't allow overriding an already valid page.
    #[inline(always)]
    fn set_page_descriptor_from_page_ptr(
        &mut self,
        virt_page_ptr: *const Page<Virtual>,
        new_desc: &PageDescriptor,
    ) -> Result<(), &'static str> {
        let (lvl2_index, lvl3_index) = self.lvl2_lvl3_index_from_page_ptr(virt_page_ptr)?;
        let desc = &mut self.lvl3[lvl2_index][lvl3_index];

        if desc.is_valid() {
            return Err("Virtual page is already mapped");
        }

        *desc = *new_desc;
        Ok(())
    }
}

//------------------------------------------------------------------------------
// OS Interface Code
//------------------------------------------------------------------------------

impl<const NUM_TABLES: usize> memory::mmu::translation_table::interface::TranslationTable
    for FixedSizeTranslationTable<NUM_TABLES>
{
    fn init(&mut self) -> Result<(), &'static str> {
        if self.initialized {
            return Ok(());
        }

        // Populate the l2 entries.
        for (lvl2_nr, lvl2_entry) in self.lvl2.iter_mut().enumerate() {
            let virt_table_addr = self.lvl3[lvl2_nr].virt_start_addr();
            let phys_table_addr = memory::mmu::try_kernel_virt_addr_to_phys_addr(virt_table_addr)?;

            let new_desc = TableDescriptor::from_next_lvl_table_addr(phys_table_addr);
            *lvl2_entry = new_desc;
        }

        self.cur_l3_mmio_index = Self::L3_MMIO_START_INDEX;
        self.initialized = true;

        Ok(())
    }

    unsafe fn map_pages_at(
        &mut self,
        virt_pages: &PageSliceDescriptor<Virtual>,
        phys_pages: &PageSliceDescriptor<Physical>,
        attr: &AttributeFields,
    ) -> Result<(), &'static str> {
        assert!(self.initialized, "Translation tables not initialized");

        let v = virt_pages.as_slice();
        let p = phys_pages.as_slice();

        // No work to do for empty slices.
        if v.is_empty() {
            return Ok(());
        }

        if v.len() != p.len() {
            return Err("Tried to map page slices with unequal sizes");
        }

        if p.last().unwrap().as_ptr() >= bsp::memory::mmu::phys_addr_space_end_page_ptr() {
            return Err("Tried to map outside of physical address space");
        }

        let iter = p.iter().zip(v.iter());
        for (phys_page, virt_page) in iter {
            let new_desc = PageDescriptor::from_output_page_ptr(phys_page.as_ptr(), attr);
            let virt_page = virt_page.as_ptr();

            self.set_page_descriptor_from_page_ptr(virt_page, &new_desc)?;
        }

        Ok(())
    }

    fn next_mmio_virt_page_slice(
        &mut self,
        num_pages: usize,
    ) -> Result<PageSliceDescriptor<Virtual>, &'static str> {
        assert!(self.initialized, "Translation tables not initialized");

        if num_pages == 0 {
            return Err("num_pages == 0");
        }

        if (self.cur_l3_mmio_index + num_pages) > 8191 {
            return Err("Not enough MMIO space left");
        }

        let addr = Address::new(
            (Self::L2_MMIO_START_INDEX << Granule512MiB::SHIFT)
                | (self.cur_l3_mmio_index << Granule64KiB::SHIFT),
        );
        self.cur_l3_mmio_index += num_pages;

        Ok(PageSliceDescriptor::from_addr(addr, num_pages))
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

    fn try_virt_page_ptr_to_phys_page_ptr(
        &self,
        virt_page_ptr: *const Page<Virtual>,
    ) -> Result<*const Page<Physical>, &'static str> {
        let page_desc = self.page_descriptor_from_page_ptr(virt_page_ptr)?;

        if !page_desc.is_valid() {
            return Err("Page marked invalid");
        }

        Ok(page_desc.output_page_ptr())
    }

    fn try_page_attributes(
        &self,
        virt_page_ptr: *const Page<Virtual>,
    ) -> Result<AttributeFields, &'static str> {
        let page_desc = self.page_descriptor_from_page_ptr(virt_page_ptr)?;

        if !page_desc.is_valid() {
            return Err("Page marked invalid");
        }

        page_desc.try_attributes()
    }

    /// Try to translate a virtual address to a physical address.
    ///
    /// Will only succeed if there exists a valid mapping for the input address.
    fn try_virt_addr_to_phys_addr(
        &self,
        virt_addr: Address<Virtual>,
    ) -> Result<Address<Physical>, &'static str> {
        let page = self.try_virt_page_ptr_to_phys_page_ptr(virt_addr.as_page_ptr())?;

        Ok(Address::new(page as usize + virt_addr.offset_into_page()))
    }
}

//--------------------------------------------------------------------------------------------------
// Testing
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
pub type MinSizeTranslationTable = FixedSizeTranslationTable<1>;

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
}
