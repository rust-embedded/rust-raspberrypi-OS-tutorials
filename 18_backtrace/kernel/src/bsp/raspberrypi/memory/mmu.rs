// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>

//! BSP Memory Management Unit.

use crate::{
    memory::{
        mmu::{
            self as generic_mmu, AddressSpace, AssociatedTranslationTable, AttributeFields,
            MemoryRegion, PageAddress, TranslationGranule,
        },
        Physical, Virtual,
    },
    synchronization::InitStateLock,
};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

type KernelTranslationTable =
    <KernelVirtAddrSpace as AssociatedTranslationTable>::TableStartFromTop;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// The translation granule chosen by this BSP. This will be used everywhere else in the kernel to
/// derive respective data structures and their sizes. For example, the `crate::memory::mmu::Page`.
pub type KernelGranule = TranslationGranule<{ 64 * 1024 }>;

/// The kernel's virtual address space defined by this BSP.
pub type KernelVirtAddrSpace = AddressSpace<{ kernel_virt_addr_space_size() }>;

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

/// The kernel translation tables.
///
/// It is mandatory that InitStateLock is transparent.
///
/// That is, `size_of(InitStateLock<KernelTranslationTable>) == size_of(KernelTranslationTable)`.
/// There is a unit tests that checks this porperty.
#[link_section = ".data"]
#[no_mangle]
static KERNEL_TABLES: InitStateLock<KernelTranslationTable> =
    InitStateLock::new(KernelTranslationTable::new_for_precompute());

/// This value is needed during early boot for MMU setup.
///
/// This will be patched to the correct value by the "translation table tool" after linking. This
/// given value here is just a dummy.
#[link_section = ".text._start_arguments"]
#[no_mangle]
static PHYS_KERNEL_TABLES_BASE_ADDR: u64 = 0xCCCCAAAAFFFFEEEE;

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

/// This is a hack for retrieving the value for the kernel's virtual address space size as a
/// constant from a common place, since it is needed as a compile-time/link-time constant in both,
/// the linker script and the Rust sources.
#[allow(clippy::needless_late_init)]
const fn kernel_virt_addr_space_size() -> usize {
    let __kernel_virt_addr_space_size;

    include!("../kernel_virt_addr_space_size.ld");

    __kernel_virt_addr_space_size
}

/// Helper function for calculating the number of pages the given parameter spans.
const fn size_to_num_pages(size: usize) -> usize {
    assert!(size > 0);
    assert!(size % KernelGranule::SIZE == 0);

    size >> KernelGranule::SHIFT
}

/// The data pages of the kernel binary.
fn virt_data_region() -> MemoryRegion<Virtual> {
    let num_pages = size_to_num_pages(super::data_size());

    let start_page_addr = super::virt_data_start();
    let end_exclusive_page_addr = start_page_addr.checked_offset(num_pages as isize).unwrap();

    MemoryRegion::new(start_page_addr, end_exclusive_page_addr)
}

// There is no reason to expect the following conversions to fail, since they were generated offline
// by the `translation table tool`. If it doesn't work, a panic due to the unwraps is justified.
fn kernel_virt_to_phys_region(virt_region: MemoryRegion<Virtual>) -> MemoryRegion<Physical> {
    let phys_start_page_addr =
        generic_mmu::try_kernel_virt_page_addr_to_phys_page_addr(virt_region.start_page_addr())
            .unwrap();

    let phys_end_exclusive_page_addr = phys_start_page_addr
        .checked_offset(virt_region.num_pages() as isize)
        .unwrap();

    MemoryRegion::new(phys_start_page_addr, phys_end_exclusive_page_addr)
}

fn kernel_page_attributes(virt_page_addr: PageAddress<Virtual>) -> AttributeFields {
    generic_mmu::try_kernel_page_attributes(virt_page_addr).unwrap()
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// The code pages of the kernel binary.
pub fn virt_code_region() -> MemoryRegion<Virtual> {
    let num_pages = size_to_num_pages(super::code_size());

    let start_page_addr = super::virt_code_start();
    let end_exclusive_page_addr = start_page_addr.checked_offset(num_pages as isize).unwrap();

    MemoryRegion::new(start_page_addr, end_exclusive_page_addr)
}

/// The boot core stack pages.
pub fn virt_boot_core_stack_region() -> MemoryRegion<Virtual> {
    let num_pages = size_to_num_pages(super::boot_core_stack_size());

    let start_page_addr = super::virt_boot_core_stack_start();
    let end_exclusive_page_addr = start_page_addr.checked_offset(num_pages as isize).unwrap();

    MemoryRegion::new(start_page_addr, end_exclusive_page_addr)
}

/// Return a reference to the kernel's translation tables.
pub fn kernel_translation_tables() -> &'static InitStateLock<KernelTranslationTable> {
    &KERNEL_TABLES
}

/// The MMIO remap pages.
pub fn virt_mmio_remap_region() -> MemoryRegion<Virtual> {
    let num_pages = size_to_num_pages(super::mmio_remap_size());

    let start_page_addr = super::virt_mmio_remap_start();
    let end_exclusive_page_addr = start_page_addr.checked_offset(num_pages as isize).unwrap();

    MemoryRegion::new(start_page_addr, end_exclusive_page_addr)
}

/// Add mapping records for the kernel binary.
///
/// The actual translation table entries for the kernel binary are generated using the offline
/// `translation table tool` and patched into the kernel binary. This function just adds the mapping
/// record entries.
pub fn kernel_add_mapping_records_for_precomputed() {
    let virt_code_region = virt_code_region();
    generic_mmu::kernel_add_mapping_record(
        "Kernel code and RO data",
        &virt_code_region,
        &kernel_virt_to_phys_region(virt_code_region),
        &kernel_page_attributes(virt_code_region.start_page_addr()),
    );

    let virt_data_region = virt_data_region();
    generic_mmu::kernel_add_mapping_record(
        "Kernel data and bss",
        &virt_data_region,
        &kernel_virt_to_phys_region(virt_data_region),
        &kernel_page_attributes(virt_data_region.start_page_addr()),
    );

    let virt_boot_core_stack_region = virt_boot_core_stack_region();
    generic_mmu::kernel_add_mapping_record(
        "Kernel boot-core stack",
        &virt_boot_core_stack_region,
        &kernel_virt_to_phys_region(virt_boot_core_stack_region),
        &kernel_page_attributes(virt_boot_core_stack_region.start_page_addr()),
    );
}
