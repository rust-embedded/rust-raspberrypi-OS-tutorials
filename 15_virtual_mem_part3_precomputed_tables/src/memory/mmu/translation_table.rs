// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2021 Andre Richter <andre.o.richter@gmail.com>

//! Translation table.

#[cfg(target_arch = "aarch64")]
#[path = "../../_arch/aarch64/memory/mmu/translation_table.rs"]
mod arch_translation_table;

use crate::memory::{
    mmu::{AttributeFields, PageSliceDescriptor},
    Address, Page, Physical, Virtual,
};

//--------------------------------------------------------------------------------------------------
// Architectural Public Reexports
//--------------------------------------------------------------------------------------------------
#[cfg(target_arch = "aarch64")]
pub use arch_translation_table::FixedSizeTranslationTable;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Translation table interfaces.
pub mod interface {
    use super::*;

    /// Translation table operations.
    pub trait TranslationTable {
        /// Anything that needs to run before any of the other provided functions can be used.
        ///
        /// # Safety
        ///
        /// - Implementor must ensure that this function can run only once or is harmless if invoked
        ///   multiple times.
        fn init(&mut self) -> Result<(), &'static str>;

        /// Map the given virtual pages to the given physical pages.
        ///
        /// # Safety
        ///
        /// - Using wrong attributes can cause multiple issues of different nature in the system.
        /// - It is not required that the architectural implementation prevents aliasing. That is,
        ///   mapping to the same physical memory using multiple virtual addresses, which would
        ///   break Rust's ownership assumptions. This should be protected against in the kernel's
        ///   generic MMU code.
        unsafe fn map_pages_at(
            &mut self,
            virt_pages: &PageSliceDescriptor<Virtual>,
            phys_pages: &PageSliceDescriptor<Physical>,
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

        /// Try to translate a virtual page pointer to a physical page pointer.
        ///
        /// Will only succeed if there exists a valid mapping for the input page.
        fn try_virt_page_ptr_to_phys_page_ptr(
            &self,
            virt_page_ptr: *const Page<Virtual>,
        ) -> Result<*const Page<Physical>, &'static str>;

        /// Try to get the attributes of a page.
        ///
        /// Will only succeed if there exists a valid mapping for the input page.
        fn try_page_attributes(
            &self,
            virt_page_ptr: *const Page<Virtual>,
        ) -> Result<AttributeFields, &'static str>;

        /// Try to translate a virtual address to a physical address.
        ///
        /// Will only succeed if there exists a valid mapping for the input address.
        fn try_virt_addr_to_phys_addr(
            &self,
            virt_addr: Address<Virtual>,
        ) -> Result<Address<Physical>, &'static str>;
    }
}

//--------------------------------------------------------------------------------------------------
// Testing
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{bsp, memory::Address};
    use arch_translation_table::MinSizeTranslationTable;
    use interface::TranslationTable;
    use test_macros::kernel_test;

    /// Sanity checks for the TranslationTable implementation.
    #[kernel_test]
    fn translationtable_implementation_sanity() {
        // This will occupy a lot of space on the stack.
        let mut tables = MinSizeTranslationTable::new_for_runtime();

        assert!(tables.init().is_ok());

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
