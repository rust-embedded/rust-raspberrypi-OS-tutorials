// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! Memory Management Unit.
//!
//! Static page tables, compiled on boot; Everything 64 KiB granule.

use crate::{
    bsp, interface,
    memory::{AccessPermissions, AttributeFields, MemAttributes},
};
use core::convert;
use cortex_a::{barrier, regs::*};
use register::register_bitfields;

// A table descriptor, as per AArch64 Reference Manual Figure D4-15.
register_bitfields! {u64,
    STAGE1_TABLE_DESCRIPTOR [
        /// Physical address of the next page table.
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

// A level 3 page descriptor, as per AArch64 Reference Manual Figure D4-17.
register_bitfields! {u64,
    STAGE1_PAGE_DESCRIPTOR [
        /// Privileged execute-never.
        PXN      OFFSET(53) NUMBITS(1) [
            False = 0,
            True = 1
        ],

        /// Physical address of the next page table (lvl2) or the page descriptor (lvl3).
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

const SIXTYFOUR_KIB_SHIFT: usize = 16; //  log2(64 * 1024)
const FIVETWELVE_MIB_SHIFT: usize = 29; // log2(512 * 1024 * 1024)

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

/// Big monolithic struct for storing the page tables. Individual levels must be 64 KiB aligned,
/// hence the "reverse" order of appearance.
#[repr(C)]
#[repr(align(65536))]
struct PageTables<const N: usize> {
    // Page descriptors, covering 64 KiB windows per entry.
    lvl3: [[PageDescriptor; 8192]; N],
    // Table descriptors, covering 512 MiB windows.
    lvl2: [TableDescriptor; N],
}

/// Usually evaluates to 1 GiB for RPi3 and 4 GiB for RPi 4.
const ENTRIES_512_MIB: usize = bsp::addr_space_size() >> FIVETWELVE_MIB_SHIFT;

/// The page tables.
///
/// Supposed to land in `.bss`. Therefore, ensure that they boil down to all "0" entries.
static mut TABLES: PageTables<{ ENTRIES_512_MIB }> = PageTables {
    lvl3: [[PageDescriptor(0); 8192]; ENTRIES_512_MIB],
    lvl2: [TableDescriptor(0); ENTRIES_512_MIB],
};

trait BaseAddr {
    fn base_addr_u64(&self) -> u64;
    fn base_addr_usize(&self) -> usize;
}

impl<T, const N: usize> BaseAddr for [T; N] {
    fn base_addr_u64(&self) -> u64 {
        self as *const T as u64
    }

    fn base_addr_usize(&self) -> usize {
        self as *const T as usize
    }
}

impl convert::From<usize> for TableDescriptor {
    fn from(next_lvl_table_addr: usize) -> Self {
        let shifted = next_lvl_table_addr >> SIXTYFOUR_KIB_SHIFT;
        let val = (STAGE1_TABLE_DESCRIPTOR::VALID::True
            + STAGE1_TABLE_DESCRIPTOR::TYPE::Table
            + STAGE1_TABLE_DESCRIPTOR::NEXT_LEVEL_TABLE_ADDR_64KiB.val(shifted as u64))
        .value;

        TableDescriptor(val)
    }
}

/// Convert the kernel's generic memory range attributes to HW-specific attributes of the MMU.
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
    fn new(output_addr: usize, attribute_fields: AttributeFields) -> PageDescriptor {
        let shifted = output_addr >> SIXTYFOUR_KIB_SHIFT;
        let val = (STAGE1_PAGE_DESCRIPTOR::VALID::True
            + STAGE1_PAGE_DESCRIPTOR::AF::True
            + attribute_fields.into()
            + STAGE1_PAGE_DESCRIPTOR::TYPE::Table
            + STAGE1_PAGE_DESCRIPTOR::OUTPUT_ADDR_64KiB.val(shifted as u64))
        .value;

        PageDescriptor(val)
    }
}

/// Constants for indexing the MAIR_EL1.
#[allow(dead_code)]
mod mair {
    pub const DEVICE: u64 = 0;
    pub const NORMAL: u64 = 1;
}

/// Setup function for the MAIR_EL1 register.
fn set_up_mair() {
    // Define the memory types being mapped.
    MAIR_EL1.write(
        // Attribute 1 - Cacheable normal DRAM.
        MAIR_EL1::Attr1_HIGH::Memory_OuterWriteBack_NonTransient_ReadAlloc_WriteAlloc
            + MAIR_EL1::Attr1_LOW_MEMORY::InnerWriteBack_NonTransient_ReadAlloc_WriteAlloc

            // Attribute 0 - Device.
            + MAIR_EL1::Attr0_HIGH::Device
            + MAIR_EL1::Attr0_LOW_DEVICE::Device_nGnRE,
    );
}

/// Iterates over all static page table entries and fills them at once.
///
/// # Safety
///
/// - Modifies a `static mut`. Ensure it only happens from here.
unsafe fn populate_pt_entries() -> Result<(), &'static str> {
    for (l2_nr, l2_entry) in TABLES.lvl2.iter_mut().enumerate() {
        *l2_entry = TABLES.lvl3[l2_nr].base_addr_usize().into();

        for (l3_nr, l3_entry) in TABLES.lvl3[l2_nr].iter_mut().enumerate() {
            let virt_addr = (l2_nr << FIVETWELVE_MIB_SHIFT) + (l3_nr << SIXTYFOUR_KIB_SHIFT);

            let (output_addr, attribute_fields) =
                bsp::virt_mem_layout().get_virt_addr_properties(virt_addr)?;

            *l3_entry = PageDescriptor::new(output_addr, attribute_fields);
        }
    }

    Ok(())
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
            + TCR_EL1::T0SZ.val(32), // TTBR0 spans 4 GiB total.
    );
}

//--------------------------------------------------------------------------------------------------
// Arch-public
//--------------------------------------------------------------------------------------------------

pub struct MMU;

//--------------------------------------------------------------------------------------------------
// OS interface implementations
//--------------------------------------------------------------------------------------------------

impl interface::mm::MMU for MMU {
    unsafe fn init(&self) -> Result<(), &'static str> {
        // Fail early if translation granule is not supported. Both RPis support it, though.
        if !ID_AA64MMFR0_EL1.matches_all(ID_AA64MMFR0_EL1::TGran64::Supported) {
            return Err("64 KiB translation granule not supported");
        }

        // Prepare the memory attribute indirection register.
        set_up_mair();

        // Populate page tables.
        populate_pt_entries()?;

        // Set the "Translation Table Base Register".
        TTBR0_EL1.set_baddr(TABLES.lvl2.base_addr_u64());

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
