// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020-2021 Andre Richter <andre.o.richter@gmail.com>

//! Memory Management Unit.

#[cfg(target_arch = "aarch64")]
#[path = "../_arch/aarch64/memory/mmu.rs"]
mod arch_mmu;

mod mapping_record;
mod translation_table;
mod types;

use crate::{
    bsp,
    memory::{Address, Physical, Virtual},
    synchronization, warn,
};

pub use types::*;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Memory Management interfaces.
pub mod interface {
    use super::*;

    /// MMU functions.
    pub trait MMU {
        /// Turns on the MMU.
        ///
        /// # Safety
        ///
        /// - Must only be called after the kernel translation tables have been init()'ed.
        /// - Changes the HW's global state.
        unsafe fn enable(
            &self,
            kernel_table_phys_base_addr: Address<Physical>,
        ) -> Result<(), &'static str>;
    }
}

/// Describes the characteristics of a translation granule.
pub struct TranslationGranule<const GRANULE_SIZE: usize>;

/// Describes the size of an address space.
pub struct AddressSpaceSize<const AS_SIZE: usize>;

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------
use interface::MMU;
use synchronization::interface::ReadWriteEx;
use translation_table::interface::TranslationTable;

/// Map pages in the kernel's translation tables.
///
/// No input checks done, input is passed through to the architectural implementation.
///
/// # Safety
///
/// - See `map_pages_at()`.
/// - Does not prevent aliasing.
unsafe fn kernel_map_pages_at_unchecked(
    name: &'static str,
    virt_pages: &PageSliceDescriptor<Virtual>,
    phys_pages: &PageSliceDescriptor<Physical>,
    attr: &AttributeFields,
) -> Result<(), &'static str> {
    arch_mmu::kernel_translation_tables()
        .write(|tables| tables.map_pages_at(virt_pages, phys_pages, attr))?;

    if let Err(x) = mapping_record::kernel_add(name, virt_pages, phys_pages, attr) {
        warn!("{}", x);
    }

    Ok(())
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

impl<const GRANULE_SIZE: usize> TranslationGranule<GRANULE_SIZE> {
    /// The granule's size.
    pub const SIZE: usize = Self::size_checked();

    /// The granule's mask.
    pub const MASK: usize = Self::SIZE - 1;

    /// The granule's shift, aka log2(size).
    pub const SHIFT: usize = Self::SIZE.trailing_zeros() as usize;

    const fn size_checked() -> usize {
        assert!(GRANULE_SIZE.is_power_of_two());

        GRANULE_SIZE
    }
}

impl<const AS_SIZE: usize> AddressSpaceSize<AS_SIZE> {
    /// The address space size.
    pub const SIZE: usize = Self::size_checked();

    /// The address space shift, aka log2(size).
    pub const SHIFT: usize = Self::SIZE.trailing_zeros() as usize;

    const fn size_checked() -> usize {
        assert!(AS_SIZE.is_power_of_two());
        assert!(arch_mmu::MIN_ADDR_SPACE_SIZE.is_power_of_two());
        assert!(arch_mmu::MAX_ADDR_SPACE_SIZE.is_power_of_two());

        // Must adhere to architectural restrictions.
        assert!(AS_SIZE >= arch_mmu::MIN_ADDR_SPACE_SIZE);
        assert!(AS_SIZE <= arch_mmu::MAX_ADDR_SPACE_SIZE);
        assert!((AS_SIZE % arch_mmu::AddrSpaceSizeGranule::SIZE) == 0);

        AS_SIZE
    }
}

/// Raw mapping of virtual to physical pages in the kernel translation tables.
///
/// Prevents mapping into the MMIO range of the tables.
///
/// # Safety
///
/// - See `kernel_map_pages_at_unchecked()`.
/// - Does not prevent aliasing. Currently, the callers must be trusted.
pub unsafe fn kernel_map_pages_at(
    name: &'static str,
    virt_pages: &PageSliceDescriptor<Virtual>,
    phys_pages: &PageSliceDescriptor<Physical>,
    attr: &AttributeFields,
) -> Result<(), &'static str> {
    let is_mmio = arch_mmu::kernel_translation_tables()
        .read(|tables| tables.is_virt_page_slice_mmio(virt_pages));
    if is_mmio {
        return Err("Attempt to manually map into MMIO region");
    }

    kernel_map_pages_at_unchecked(name, virt_pages, phys_pages, attr)?;

    Ok(())
}

/// MMIO remapping in the kernel translation tables.
///
/// Typically used by device drivers.
///
/// # Safety
///
/// - Same as `kernel_map_pages_at_unchecked()`, minus the aliasing part.
pub unsafe fn kernel_map_mmio(
    name: &'static str,
    phys_mmio_descriptor: &MMIODescriptor<Physical>,
) -> Result<Address<Virtual>, &'static str> {
    let phys_pages: PageSliceDescriptor<Physical> = phys_mmio_descriptor.clone().into();
    let offset_into_start_page =
        phys_mmio_descriptor.start_addr().into_usize() & bsp::memory::mmu::KernelGranule::MASK;

    // Check if an identical page slice has been mapped for another driver. If so, reuse it.
    let virt_addr = if let Some(addr) =
        mapping_record::kernel_find_and_insert_mmio_duplicate(phys_mmio_descriptor, name)
    {
        addr
    // Otherwise, allocate a new virtual page slice and map it.
    } else {
        let virt_pages: PageSliceDescriptor<Virtual> = arch_mmu::kernel_translation_tables()
            .write(|tables| tables.next_mmio_virt_page_slice(phys_pages.num_pages()))?;

        kernel_map_pages_at_unchecked(
            name,
            &virt_pages,
            &phys_pages,
            &AttributeFields {
                mem_attributes: MemAttributes::Device,
                acc_perms: AccessPermissions::ReadWrite,
                execute_never: true,
            },
        )?;

        virt_pages.start_addr()
    };

    Ok(virt_addr + offset_into_start_page)
}

/// Map the kernel's binary and enable the MMU.
///
/// # Safety
///
/// - Crucial function during kernel init. Changes the the complete memory view of the processor.
pub unsafe fn kernel_map_binary_and_enable_mmu() -> Result<(), &'static str> {
    let phys_base_addr = arch_mmu::kernel_translation_tables().write(|tables| {
        tables.init();
        tables.phys_base_address()
    });

    bsp::memory::mmu::kernel_map_binary()?;
    arch_mmu::mmu().enable(phys_base_addr)
}

/// Human-readable print of all recorded kernel mappings.
pub fn kernel_print_mappings() {
    mapping_record::kernel_print()
}
