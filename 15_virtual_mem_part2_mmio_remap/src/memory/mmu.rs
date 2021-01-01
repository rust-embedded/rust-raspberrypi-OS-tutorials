// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020-2021 Andre Richter <andre.o.richter@gmail.com>

//! Memory Management Unit.

#[cfg(target_arch = "aarch64")]
#[path = "../_arch/aarch64/memory/mmu.rs"]
mod arch_mmu;
pub use arch_mmu::*;

mod mapping_record;
mod types;

use crate::{bsp, synchronization, warn};

pub use types::*;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Memory Management interfaces.
pub mod interface {
    use super::*;

    /// Describes the characteristics of a translation granule.
    #[allow(missing_docs)]
    pub trait TranslationGranule {
        const SIZE: usize;
        const MASK: usize = Self::SIZE - 1;
        const SHIFT: usize;
    }

    /// Translation table operations.
    pub trait TranslationTable {
        /// Anything that needs to run before any of the other provided functions can be used.
        ///
        /// # Safety
        ///
        /// - Implementor must ensure that this function can run only once or is harmless if invoked
        ///   multiple times.
        unsafe fn init(&mut self);

        /// The translation table's base address to be used for programming the MMU.
        fn phys_base_address(&self) -> Address<Physical>;

        /// Map the given physical pages to the given virtual pages.
        ///
        /// # Safety
        ///
        /// - Using wrong attributes can cause multiple issues of different nature in the system.
        /// - It is not required that the architectural implementation prevents aliasing. That is,
        ///   mapping to the same physical memory using multiple virtual addresses, which would
        ///   break Rust's ownership assumptions. This should be protected against in this module
        ///   (the kernel's generic MMU code).
        unsafe fn map_pages_at(
            &mut self,
            phys_pages: &PageSliceDescriptor<Physical>,
            virt_pages: &PageSliceDescriptor<Virtual>,
            attr: &AttributeFields,
        ) -> Result<(), &'static str>;

        /// Obtain a free virtual page slice in the MMIO region.
        ///
        /// The "MMIO region" is a distinct region of the implementor's choice, which allows
        /// differentiating MMIO addresses from others. This can speed up debugging efforts.
        /// Ideally, those MMIO addresses are also standing out visually so that a human eye can
        /// identify them. For example, by allocating them from near the end of the virtual address
        /// space.
        fn next_mmio_virt_page_slice(
            &mut self,
            num_pages: usize,
        ) -> Result<PageSliceDescriptor<Virtual>, &'static str>;

        /// Check if a virtual page splice is in the "MMIO region".
        fn is_virt_page_slice_mmio(&self, virt_pages: &PageSliceDescriptor<Virtual>) -> bool;
    }

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
            phys_kernel_table_base_addr: Address<Physical>,
        ) -> Result<(), &'static str>;
    }
}

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------
use interface::{TranslationTable, MMU};
use synchronization::interface::ReadWriteEx;

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
    phys_pages: &PageSliceDescriptor<Physical>,
    virt_pages: &PageSliceDescriptor<Virtual>,
    attr: &AttributeFields,
) -> Result<(), &'static str> {
    arch_mmu::kernel_translation_tables()
        .write(|tables| tables.map_pages_at(phys_pages, virt_pages, attr))?;

    if let Err(x) = mapping_record::kernel_add(name, phys_pages, virt_pages, attr) {
        warn!("{}", x);
    }

    Ok(())
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------
use interface::TranslationGranule;

/// Raw mapping of virtual to physical pages in the kernel translation tables.
///
/// Prevents mapping into the MMIO range of the tables.
///
/// # Safety
///
/// - See `kernel_map_pages_at_unchecked()`.
/// - Does not prevent aliasing. Currently, we have to trust the callers.
pub unsafe fn kernel_map_pages_at(
    name: &'static str,
    phys_pages: &PageSliceDescriptor<Physical>,
    virt_pages: &PageSliceDescriptor<Virtual>,
    attr: &AttributeFields,
) -> Result<(), &'static str> {
    let is_mmio = arch_mmu::kernel_translation_tables()
        .read(|tables| tables.is_virt_page_slice_mmio(virt_pages));
    if is_mmio {
        return Err("Attempt to manually map into MMIO region");
    }

    kernel_map_pages_at_unchecked(name, phys_pages, virt_pages, attr)?;

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
            &phys_pages,
            &virt_pages,
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

//--------------------------------------------------------------------------------------------------
// Testing
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use test_macros::kernel_test;

    /// Sanity checks for the kernel TranslationTable implementation.
    #[kernel_test]
    fn translationtable_implementation_sanity() {
        // Need to take care that `tables` fits into the stack.
        let mut tables = MinSizeArchTranslationTable::new();

        unsafe { tables.init() };

        let x = tables.next_mmio_virt_page_slice(0);
        assert!(x.is_err());

        let x = tables.next_mmio_virt_page_slice(1_0000_0000);
        assert!(x.is_err());

        let x = tables.next_mmio_virt_page_slice(2).unwrap();
        assert_eq!(x.size(), bsp::memory::mmu::KernelGranule::SIZE * 2);

        assert_eq!(tables.is_virt_page_slice_mmio(&x), true);

        assert_eq!(
            tables.is_virt_page_slice_mmio(&PageSliceDescriptor::from_addr(Address::new(0), 1)),
            false
        );
    }
}
