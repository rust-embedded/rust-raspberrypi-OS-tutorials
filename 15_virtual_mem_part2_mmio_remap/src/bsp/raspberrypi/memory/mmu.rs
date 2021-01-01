// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! BSP Memory Management Unit.

use crate::{
    common,
    memory::{
        mmu as kernel_mmu,
        mmu::{
            interface, AccessPermissions, AttributeFields, Granule64KiB, MemAttributes, Page,
            PageSliceDescriptor, Physical, Virtual,
        },
    },
};

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// The translation granule chosen by this BSP. This will be used everywhere else in the kernel to
/// derive respective data structures and their sizes. For example, the `crate::memory::mmu::Page`.
pub type KernelGranule = Granule64KiB;

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------
use interface::TranslationGranule;

/// Helper function for calculating the number of pages the given parameter spans.
const fn size_to_num_pages(size: usize) -> usize {
    assert!(size > 0);
    assert!(size % KernelGranule::SIZE == 0);

    size >> KernelGranule::SHIFT
}

/// The boot core's stack.
fn virt_stack_page_desc() -> PageSliceDescriptor<Virtual> {
    let num_pages = size_to_num_pages(super::boot_core_stack_size());

    PageSliceDescriptor::from_addr(super::virt_boot_core_stack_start(), num_pages)
}

/// The Read-Only (RO) pages of the kernel binary.
fn virt_ro_page_desc() -> PageSliceDescriptor<Virtual> {
    let num_pages = size_to_num_pages(super::ro_size());

    PageSliceDescriptor::from_addr(super::virt_ro_start(), num_pages)
}

/// The data pages of the kernel binary.
fn virt_data_page_desc() -> PageSliceDescriptor<Virtual> {
    let num_pages = size_to_num_pages(super::data_size());

    PageSliceDescriptor::from_addr(super::virt_data_start(), num_pages)
}

// The binary is still identity mapped, so we don't need to convert in the following.

/// The boot core's stack.
fn phys_stack_page_desc() -> PageSliceDescriptor<Physical> {
    virt_stack_page_desc().into()
}

/// The Read-Only (RO) pages of the kernel binary.
fn phys_ro_page_desc() -> PageSliceDescriptor<Physical> {
    virt_ro_page_desc().into()
}

/// The data pages of the kernel binary.
fn phys_data_page_desc() -> PageSliceDescriptor<Physical> {
    virt_data_page_desc().into()
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Pointer to the last page of the physical address space.
pub fn phys_addr_space_end_page() -> *const Page<Physical> {
    common::align_down(
        super::phys_addr_space_end().into_usize(),
        KernelGranule::SIZE,
    ) as *const Page<_>
}

/// Map the kernel binary.
///
/// # Safety
///
/// - Any miscalculation or attribute error will likely be fatal. Needs careful manual checking.
pub unsafe fn kernel_map_binary() -> Result<(), &'static str> {
    kernel_mmu::kernel_map_pages_at(
        "Kernel boot-core stack",
        &phys_stack_page_desc(),
        &virt_stack_page_desc(),
        &AttributeFields {
            mem_attributes: MemAttributes::CacheableDRAM,
            acc_perms: AccessPermissions::ReadWrite,
            execute_never: true,
        },
    )?;

    kernel_mmu::kernel_map_pages_at(
        "Kernel code and RO data",
        &phys_ro_page_desc(),
        &virt_ro_page_desc(),
        &AttributeFields {
            mem_attributes: MemAttributes::CacheableDRAM,
            acc_perms: AccessPermissions::ReadOnly,
            execute_never: false,
        },
    )?;

    kernel_mmu::kernel_map_pages_at(
        "Kernel data and bss",
        &phys_data_page_desc(),
        &virt_data_page_desc(),
        &AttributeFields {
            mem_attributes: MemAttributes::CacheableDRAM,
            acc_perms: AccessPermissions::ReadWrite,
            execute_never: true,
        },
    )?;

    Ok(())
}

//--------------------------------------------------------------------------------------------------
// Testing
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use test_macros::kernel_test;

    /// Check alignment of the kernel's virtual memory layout sections.
    #[kernel_test]
    fn virt_mem_layout_sections_are_64KiB_aligned() {
        for i in [virt_stack_page_desc, virt_ro_page_desc, virt_data_page_desc].iter() {
            let start: usize = i().start_addr().into_usize();
            let end: usize = i().end_addr().into_usize();

            assert_eq!(start % KernelGranule::SIZE, 0);
            assert_eq!(end % KernelGranule::SIZE, 0);
            assert!(end >= start);
        }
    }

    /// Ensure the kernel's virtual memory layout is free of overlaps.
    #[kernel_test]
    fn virt_mem_layout_has_no_overlaps() {
        let layout = [
            virt_stack_page_desc().into_usize_range_inclusive(),
            virt_ro_page_desc().into_usize_range_inclusive(),
            virt_data_page_desc().into_usize_range_inclusive(),
        ];

        for (i, first_range) in layout.iter().enumerate() {
            for second_range in layout.iter().skip(i + 1) {
                assert!(!first_range.contains(second_range.start()));
                assert!(!first_range.contains(second_range.end()));
                assert!(!second_range.contains(first_range.start()));
                assert!(!second_range.contains(first_range.end()));
            }
        }
    }
}
