/*
 * MIT License
 *
 * Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

use crate::memory::{get_virt_addr_properties, AttributeFields};
use cortex_a::{barrier, regs::*};
use register::register_bitfields;

register_bitfields! {u64,
    // AArch64 Reference Manual page 2150
    STAGE1_DESCRIPTOR [
        /// Privileged execute-never
        PXN      OFFSET(53) NUMBITS(1) [
            False = 0,
            True = 1
        ],

        /// Various address fields, depending on use case
        LVL2_OUTPUT_ADDR_4KiB    OFFSET(21) NUMBITS(27) [], // [47:21]
        NEXT_LVL_TABLE_ADDR_4KiB OFFSET(12) NUMBITS(36) [], // [47:12]

        /// Access flag
        AF       OFFSET(10) NUMBITS(1) [
            False = 0,
            True = 1
        ],

        /// Shareability field
        SH       OFFSET(8) NUMBITS(2) [
            OuterShareable = 0b10,
            InnerShareable = 0b11
        ],

        /// Access Permissions
        AP       OFFSET(6) NUMBITS(2) [
            RW_EL1 = 0b00,
            RW_EL1_EL0 = 0b01,
            RO_EL1 = 0b10,
            RO_EL1_EL0 = 0b11
        ],

        /// Memory attributes index into the MAIR_EL1 register
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

const FOUR_KIB: usize = 4 * 1024;
const FOUR_KIB_SHIFT: usize = 12; // log2(4 * 1024)

const TWO_MIB: usize = 2 * 1024 * 1024;
const TWO_MIB_SHIFT: usize = 21; // log2(2 * 1024 * 1024)

/// A descriptor pointing to the next page table.
struct TableDescriptor(register::FieldValue<u64, STAGE1_DESCRIPTOR::Register>);

impl TableDescriptor {
    fn new(next_lvl_table_addr: usize) -> Result<TableDescriptor, &'static str> {
        if next_lvl_table_addr % FOUR_KIB != 0 {
            return Err("TableDescriptor: Address is not 4 KiB aligned.");
        }

        let shifted = next_lvl_table_addr >> FOUR_KIB_SHIFT;

        Ok(TableDescriptor(
            STAGE1_DESCRIPTOR::VALID::True
                + STAGE1_DESCRIPTOR::TYPE::Table
                + STAGE1_DESCRIPTOR::NEXT_LVL_TABLE_ADDR_4KiB.val(shifted as u64),
        ))
    }

    fn value(&self) -> u64 {
        self.0.value
    }
}

/// A function that maps the generic memory range attributes to HW-specific
/// attributes of the MMU.
fn into_mmu_attributes(
    attribute_fields: AttributeFields,
) -> register::FieldValue<u64, STAGE1_DESCRIPTOR::Register> {
    use crate::memory::{AccessPermissions, MemAttributes};

    // Memory attributes
    let mut desc = match attribute_fields.mem_attributes {
        MemAttributes::CacheableDRAM => {
            STAGE1_DESCRIPTOR::SH::InnerShareable + STAGE1_DESCRIPTOR::AttrIndx.val(mair::NORMAL)
        }
        MemAttributes::NonCacheableDRAM => {
            STAGE1_DESCRIPTOR::SH::InnerShareable
                + STAGE1_DESCRIPTOR::AttrIndx.val(mair::NORMAL_NON_CACHEABLE)
        }
        MemAttributes::Device => {
            STAGE1_DESCRIPTOR::SH::OuterShareable + STAGE1_DESCRIPTOR::AttrIndx.val(mair::DEVICE)
        }
    };

    // Access Permissions
    desc += match attribute_fields.acc_perms {
        AccessPermissions::ReadOnly => STAGE1_DESCRIPTOR::AP::RO_EL1,
        AccessPermissions::ReadWrite => STAGE1_DESCRIPTOR::AP::RW_EL1,
    };

    // Execute Never
    desc += if attribute_fields.execute_never {
        STAGE1_DESCRIPTOR::PXN::True
    } else {
        STAGE1_DESCRIPTOR::PXN::False
    };

    desc
}

/// A Level2 block descriptor with 2 MiB aperture.
///
/// The output points to physical memory.
struct Lvl2BlockDescriptor(register::FieldValue<u64, STAGE1_DESCRIPTOR::Register>);

impl Lvl2BlockDescriptor {
    fn new(
        output_addr: usize,
        attribute_fields: AttributeFields,
    ) -> Result<Lvl2BlockDescriptor, &'static str> {
        if output_addr % TWO_MIB != 0 {
            return Err("BlockDescriptor: Address is not 2 MiB aligned.");
        }

        let shifted = output_addr >> TWO_MIB_SHIFT;

        Ok(Lvl2BlockDescriptor(
            STAGE1_DESCRIPTOR::VALID::True
                + STAGE1_DESCRIPTOR::AF::True
                + into_mmu_attributes(attribute_fields)
                + STAGE1_DESCRIPTOR::TYPE::Block
                + STAGE1_DESCRIPTOR::LVL2_OUTPUT_ADDR_4KiB.val(shifted as u64),
        ))
    }

    fn value(&self) -> u64 {
        self.0.value
    }
}

/// A page descriptor with 4 KiB aperture.
///
/// The output points to physical memory.
struct PageDescriptor(register::FieldValue<u64, STAGE1_DESCRIPTOR::Register>);

impl PageDescriptor {
    fn new(
        output_addr: usize,
        attribute_fields: AttributeFields,
    ) -> Result<PageDescriptor, &'static str> {
        if output_addr % FOUR_KIB != 0 {
            return Err("PageDescriptor: Address is not 4 KiB aligned.");
        }

        let shifted = output_addr >> FOUR_KIB_SHIFT;

        Ok(PageDescriptor(
            STAGE1_DESCRIPTOR::VALID::True
                + STAGE1_DESCRIPTOR::AF::True
                + into_mmu_attributes(attribute_fields)
                + STAGE1_DESCRIPTOR::TYPE::Table
                + STAGE1_DESCRIPTOR::NEXT_LVL_TABLE_ADDR_4KiB.val(shifted as u64),
        ))
    }

    fn value(&self) -> u64 {
        self.0.value
    }
}

/// Constants for indexing the MAIR_EL1.
#[allow(dead_code)]
mod mair {
    pub const DEVICE: u64 = 0;
    pub const NORMAL: u64 = 1;
    pub const NORMAL_NON_CACHEABLE: u64 = 2;
}

/// Setup function for the MAIR_EL1 register.
fn set_up_mair() {
    // Define the three memory types that we will map. Cacheable and
    // non-cacheable normal DRAM, and device.
    MAIR_EL1.write(
        // Attribute 2
        MAIR_EL1::Attr2_HIGH::Memory_OuterNonCacheable
            + MAIR_EL1::Attr2_LOW_MEMORY::InnerNonCacheable

        // Attribute 1
            + MAIR_EL1::Attr1_HIGH::Memory_OuterWriteBack_NonTransient_ReadAlloc_WriteAlloc
            + MAIR_EL1::Attr1_LOW_MEMORY::InnerWriteBack_NonTransient_ReadAlloc_WriteAlloc

            // Attribute 0
            + MAIR_EL1::Attr0_HIGH::Device
            + MAIR_EL1::Attr0_LOW_DEVICE::Device_nGnRE,
    );
}

trait BaseAddr {
    fn base_addr_u64(&self) -> u64;
    fn base_addr_usize(&self) -> usize;
}

impl BaseAddr for [u64; 512] {
    fn base_addr_u64(&self) -> u64 {
        self as *const u64 as u64
    }

    fn base_addr_usize(&self) -> usize {
        self as *const u64 as usize
    }
}

const NUM_ENTRIES_4KIB: usize = 512;

// A wrapper struct is needed here so that the align attribute can be used.
#[repr(C)]
#[repr(align(4096))]
struct PageTable {
    entries: [u64; NUM_ENTRIES_4KIB],
}

/// The LVL2 page table containng the 2 MiB entries.
static mut LVL2_TABLE: PageTable = PageTable {
    entries: [0; NUM_ENTRIES_4KIB],
};

/// The LVL3 page table containing the 4 KiB entries.
///
/// The first entry of the LVL2_TABLE will forward to this table.
static mut LVL3_TABLE: PageTable = PageTable {
    entries: [0; NUM_ENTRIES_4KIB],
};

/// Set up identity mapped page tables for the first 1 GiB of address space.
///
/// The first 2 MiB are 4 KiB granule, the rest 2 MiB.
pub unsafe fn init() -> Result<(), &'static str> {
    // Prepare the memory attribute indirection register.
    set_up_mair();

    // Point the first 2 MiB of virtual addresses to the follow-up LVL3
    // page-table.
    LVL2_TABLE.entries[0] = match TableDescriptor::new(LVL3_TABLE.entries.base_addr_usize()) {
        Err(s) => return Err(s),
        Ok(d) => d.value(),
    };

    // Fill the rest of the LVL2 (2 MiB) entries as block descriptors.
    //
    // Notice the skip(1) which makes the iteration start at the second 2 MiB
    // block (0x20_0000).
    for (block_descriptor_nr, entry) in LVL2_TABLE.entries.iter_mut().enumerate().skip(1) {
        let virt_addr = block_descriptor_nr << TWO_MIB_SHIFT;

        let (output_addr, attribute_fields) = match get_virt_addr_properties(virt_addr) {
            Err(s) => return Err(s),
            Ok((a, b)) => (a, b),
        };

        let block_desc = match Lvl2BlockDescriptor::new(output_addr, attribute_fields) {
            Err(s) => return Err(s),
            Ok(desc) => desc,
        };

        *entry = block_desc.value();
    }

    // Finally, fill the single LVL3 table (4 KiB granule).
    for (page_descriptor_nr, entry) in LVL3_TABLE.entries.iter_mut().enumerate() {
        let virt_addr = page_descriptor_nr << FOUR_KIB_SHIFT;

        let (output_addr, attribute_fields) = match get_virt_addr_properties(virt_addr) {
            Err(s) => return Err(s),
            Ok((a, b)) => (a, b),
        };

        let page_desc = match PageDescriptor::new(output_addr, attribute_fields) {
            Err(s) => return Err(s),
            Ok(desc) => desc,
        };

        *entry = page_desc.value();
    }

    // Point to the LVL2 table base address in TTBR0.
    TTBR0_EL1.set_baddr(LVL2_TABLE.entries.base_addr_u64());

    // Configure various settings of stage 1 of the EL1 translation regime.
    let ips = ID_AA64MMFR0_EL1.read(ID_AA64MMFR0_EL1::PARange);
    TCR_EL1.write(
        TCR_EL1::TBI0::Ignored
            + TCR_EL1::IPS.val(ips)
            + TCR_EL1::TG0::KiB_4 // 4 KiB granule
            + TCR_EL1::SH0::Inner
            + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
            + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
            + TCR_EL1::EPD0::EnableTTBR0Walks
            + TCR_EL1::T0SZ.val(34), // Start walks at level 2
    );

    // Switch the MMU on.
    //
    // First, force all previous changes to be seen before the MMU is enabled.
    barrier::isb(barrier::SY);

    // Enable the MMU and turn on data and instruction caching.
    SCTLR_EL1.modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable);

    // Force MMU init to complete before next instruction
    barrier::isb(barrier::SY);

    Ok(())
}
