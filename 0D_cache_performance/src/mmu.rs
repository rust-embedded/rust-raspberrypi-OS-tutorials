/*
 * MIT License
 *
 * Copyright (c) 2018 Andre Richter <andre.o.richter@gmail.com>
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

use cortex_a::{barrier, regs::*};
use register::register_bitfields;

register_bitfields! {u64,
    // AArch64 Reference Manual page 2150
    STAGE1_DESCRIPTOR [
        /// Execute-never
        XN       OFFSET(54) NUMBITS(1) [
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

trait BaseAddr {
    fn base_addr(&self) -> u64;
}

impl BaseAddr for [u64; 512] {
    fn base_addr(&self) -> u64 {
        self as *const u64 as u64
    }
}

const NUM_ENTRIES_4KIB: usize = 512;

// We need a wrapper struct here so that we can make use of the align attribute.
#[repr(C)]
#[repr(align(4096))]
struct PageTable {
    entries: [u64; NUM_ENTRIES_4KIB],
}

static mut LVL2_TABLE: PageTable = PageTable {
    entries: [0; NUM_ENTRIES_4KIB],
};
static mut SINGLE_LVL3_TABLE: PageTable = PageTable {
    entries: [0; NUM_ENTRIES_4KIB],
};

/// Set up identity mapped page tables for the first 1 gigabyte of address
/// space.
pub unsafe fn init() {
    // First, define the three memory types that we will map. Cacheable and
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

    // Descriptive consts for indexing into the correct MAIR_EL1 attributes.
    mod mair {
        pub const DEVICE: u64 = 0;
        pub const NORMAL: u64 = 1;
        pub const NORMAL_NON_CACHEABLE: u64 = 2;
    }

    // Set up the first LVL2 entry, pointing to a 4KiB table base address.
    let lvl3_base: u64 = SINGLE_LVL3_TABLE.entries.base_addr() >> 12;
    LVL2_TABLE.entries[0] = (STAGE1_DESCRIPTOR::VALID::True
        + STAGE1_DESCRIPTOR::TYPE::Table
        + STAGE1_DESCRIPTOR::NEXT_LVL_TABLE_ADDR_4KiB.val(lvl3_base))
    .value;

    // The second 2 MiB block.
    LVL2_TABLE.entries[1] = (STAGE1_DESCRIPTOR::VALID::True
        + STAGE1_DESCRIPTOR::TYPE::Block
        + STAGE1_DESCRIPTOR::AttrIndx.val(mair::NORMAL_NON_CACHEABLE)
        + STAGE1_DESCRIPTOR::AP::RW_EL1
        + STAGE1_DESCRIPTOR::SH::OuterShareable
        + STAGE1_DESCRIPTOR::AF::True
        // This translation is accessed for virtual 0x200000. Point to physical
        // 0x400000, aka the third phyiscal 2 MiB DRAM block (third block == 2,
        // because we start counting at 0).
        //
        // Here, we configure it non-cacheable.
        + STAGE1_DESCRIPTOR::LVL2_OUTPUT_ADDR_4KiB.val(2)
        + STAGE1_DESCRIPTOR::XN::True)
        .value;

    // Fill the rest of the LVL2 (2MiB) entries as block
    // descriptors. Differentiate between normal and device mem.
    let mmio_base: u64 = (super::MMIO_BASE >> 21).into();
    let common = STAGE1_DESCRIPTOR::VALID::True
        + STAGE1_DESCRIPTOR::TYPE::Block
        + STAGE1_DESCRIPTOR::AP::RW_EL1
        + STAGE1_DESCRIPTOR::AF::True
        + STAGE1_DESCRIPTOR::XN::True;

    // Notice the skip(2). Start at the third 2 MiB DRAM block, which will point
    // virtual 0x400000 to physical 0x400000, configured as cacheable memory.
    for (i, entry) in LVL2_TABLE.entries.iter_mut().enumerate().skip(2) {
        let j: u64 = i as u64;

        let mem_attr = if j >= mmio_base {
            STAGE1_DESCRIPTOR::SH::OuterShareable + STAGE1_DESCRIPTOR::AttrIndx.val(mair::DEVICE)
        } else {
            STAGE1_DESCRIPTOR::SH::InnerShareable + STAGE1_DESCRIPTOR::AttrIndx.val(mair::NORMAL)
        };

        *entry = (common + mem_attr + STAGE1_DESCRIPTOR::LVL2_OUTPUT_ADDR_4KiB.val(j)).value;
    }

    // Finally, fill the single LVL3 table (4 KiB granule). Differentiate
    // between code+RO and RW pages.
    //
    // Using the linker script, we ensure that the RO area is consecutive and 4
    // KiB aligned, and we export the boundaries via symbols.
    extern "C" {
        // The inclusive start of the read-only area, aka the address of the
        // first byte of the area.
        static mut __ro_start: u64;

        // The non-inclusive end of the read-only area, aka the address of the
        // first byte _after_ the RO area.
        static mut __ro_end: u64;
    }

    const PAGESIZE: u64 = 4096;
    let ro_first_page_index: u64 = &__ro_start as *const _ as u64 / PAGESIZE;

    // Notice the subtraction to calculate the last page index of the RO area
    // and not the first page index after the RO area.
    let ro_last_page_index: u64 = (&__ro_end as *const _ as u64 / PAGESIZE) - 1;

    let common = STAGE1_DESCRIPTOR::VALID::True
        + STAGE1_DESCRIPTOR::TYPE::Table
        + STAGE1_DESCRIPTOR::AttrIndx.val(mair::NORMAL)
        + STAGE1_DESCRIPTOR::SH::InnerShareable
        + STAGE1_DESCRIPTOR::AF::True;

    for (i, entry) in SINGLE_LVL3_TABLE.entries.iter_mut().enumerate() {
        let j: u64 = i as u64;

        let mem_attr = if j < ro_first_page_index || j > ro_last_page_index {
            STAGE1_DESCRIPTOR::AP::RW_EL1 + STAGE1_DESCRIPTOR::XN::True
        } else {
            STAGE1_DESCRIPTOR::AP::RO_EL1 + STAGE1_DESCRIPTOR::XN::False
        };

        *entry = (common + mem_attr + STAGE1_DESCRIPTOR::NEXT_LVL_TABLE_ADDR_4KiB.val(j)).value;
    }

    // Point to the LVL2 table base address in TTBR0.
    TTBR0_EL1.set_baddr(LVL2_TABLE.entries.base_addr());

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
}
