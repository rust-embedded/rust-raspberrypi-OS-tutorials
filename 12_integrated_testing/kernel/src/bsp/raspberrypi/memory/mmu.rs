// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>

//! BSP Memory Management Unit.

use super::map as memory_map;
use crate::memory::mmu::*;
use core::ops::RangeInclusive;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// The kernel's address space defined by this BSP.
pub type KernelAddrSpace = AddressSpace<{ memory_map::END_INCLUSIVE + 1 }>;

const NUM_MEM_RANGES: usize = 2;

/// The virtual memory layout.
///
/// The layout must contain only special ranges, aka anything that is _not_ normal cacheable DRAM.
/// It is agnostic of the paging granularity that the architecture's MMU will use.
pub static LAYOUT: KernelVirtualLayout<NUM_MEM_RANGES> = KernelVirtualLayout::new(
    memory_map::END_INCLUSIVE,
    [
        TranslationDescriptor {
            name: "Kernel code and RO data",
            virtual_range: code_range_inclusive,
            physical_range_translation: Translation::Identity,
            attribute_fields: AttributeFields {
                mem_attributes: MemAttributes::CacheableDRAM,
                acc_perms: AccessPermissions::ReadOnly,
                execute_never: false,
            },
        },
        TranslationDescriptor {
            name: "Device MMIO",
            virtual_range: mmio_range_inclusive,
            physical_range_translation: Translation::Identity,
            attribute_fields: AttributeFields {
                mem_attributes: MemAttributes::Device,
                acc_perms: AccessPermissions::ReadWrite,
                execute_never: true,
            },
        },
    ],
);

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

fn code_range_inclusive() -> RangeInclusive<usize> {
    // Notice the subtraction to turn the exclusive end into an inclusive end.
    #[allow(clippy::range_minus_one)]
    RangeInclusive::new(super::code_start(), super::code_end_exclusive() - 1)
}

fn mmio_range_inclusive() -> RangeInclusive<usize> {
    RangeInclusive::new(memory_map::mmio::START, memory_map::mmio::END_INCLUSIVE)
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Return a reference to the virtual memory layout.
pub fn virt_mem_layout() -> &'static KernelVirtualLayout<NUM_MEM_RANGES> {
    &LAYOUT
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
        const SIXTYFOUR_KIB: usize = 65536;

        for i in LAYOUT.inner().iter() {
            let start: usize = *(i.virtual_range)().start();
            let end: usize = *(i.virtual_range)().end() + 1;

            assert_eq!(start % SIXTYFOUR_KIB, 0);
            assert_eq!(end % SIXTYFOUR_KIB, 0);
            assert!(end >= start);
        }
    }

    /// Ensure the kernel's virtual memory layout is free of overlaps.
    #[kernel_test]
    fn virt_mem_layout_has_no_overlaps() {
        let layout = virt_mem_layout().inner();

        for (i, first) in layout.iter().enumerate() {
            for second in layout.iter().skip(i + 1) {
                let first_range = first.virtual_range;
                let second_range = second.virtual_range;

                assert!(!first_range().contains(second_range().start()));
                assert!(!first_range().contains(second_range().end()));
                assert!(!second_range().contains(first_range().start()));
                assert!(!second_range().contains(first_range().end()));
            }
        }
    }
}
