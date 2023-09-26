// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2021-2023 Andre Richter <andre.o.richter@gmail.com>

//! Translation table.

#[cfg(target_arch = "aarch64")]
#[path = "../../_arch/aarch64/memory/mmu/translation_table.rs"]
mod arch_translation_table;

use super::{AttributeFields, MemoryRegion};
use crate::memory::{Address, Physical, Virtual};

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
    use crate::memory::mmu::PageAddress;

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

        /// Map the given virtual memory region to the given physical memory region.
        ///
        /// # Safety
        ///
        /// - Using wrong attributes can cause multiple issues of different nature in the system.
        /// - It is not required that the architectural implementation prevents aliasing. That is,
        ///   mapping to the same physical memory using multiple virtual addresses, which would
        ///   break Rust's ownership assumptions. This should be protected against in the kernel's
        ///   generic MMU code.
        unsafe fn map_at(
            &mut self,
            virt_region: &MemoryRegion<Virtual>,
            phys_region: &MemoryRegion<Physical>,
            attr: &AttributeFields,
        ) -> Result<(), &'static str>;

        /// Try to translate a virtual page address to a physical page address.
        ///
        /// Will only succeed if there exists a valid mapping for the input page.
        fn try_virt_page_addr_to_phys_page_addr(
            &self,
            virt_page_addr: PageAddress<Virtual>,
        ) -> Result<PageAddress<Physical>, &'static str>;

        /// Try to get the attributes of a page.
        ///
        /// Will only succeed if there exists a valid mapping for the input page.
        fn try_page_attributes(
            &self,
            virt_page_addr: PageAddress<Virtual>,
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
    use crate::memory::mmu::{AccessPermissions, MemAttributes, PageAddress};
    use arch_translation_table::MinSizeTranslationTable;
    use interface::TranslationTable;
    use test_macros::kernel_test;

    /// Sanity checks for the TranslationTable implementation.
    #[kernel_test]
    fn translationtable_implementation_sanity() {
        // This will occupy a lot of space on the stack.
        let mut tables = MinSizeTranslationTable::new_for_runtime();

        assert_eq!(tables.init(), Ok(()));

        let virt_start_page_addr: PageAddress<Virtual> = PageAddress::from(0);
        let virt_end_exclusive_page_addr: PageAddress<Virtual> =
            virt_start_page_addr.checked_offset(5).unwrap();

        let phys_start_page_addr: PageAddress<Physical> = PageAddress::from(0);
        let phys_end_exclusive_page_addr: PageAddress<Physical> =
            phys_start_page_addr.checked_offset(5).unwrap();

        let virt_region = MemoryRegion::new(virt_start_page_addr, virt_end_exclusive_page_addr);
        let phys_region = MemoryRegion::new(phys_start_page_addr, phys_end_exclusive_page_addr);

        let attr = AttributeFields {
            mem_attributes: MemAttributes::CacheableDRAM,
            acc_perms: AccessPermissions::ReadWrite,
            execute_never: true,
        };

        unsafe { assert_eq!(tables.map_at(&virt_region, &phys_region, &attr), Ok(())) };

        assert_eq!(
            tables.try_virt_page_addr_to_phys_page_addr(virt_start_page_addr),
            Ok(phys_start_page_addr)
        );

        assert_eq!(
            tables.try_page_attributes(virt_start_page_addr.checked_offset(6).unwrap()),
            Err("Page marked invalid")
        );

        assert_eq!(tables.try_page_attributes(virt_start_page_addr), Ok(attr));

        let virt_addr = virt_start_page_addr.into_inner() + 0x100;
        let phys_addr = phys_start_page_addr.into_inner() + 0x100;
        assert_eq!(tables.try_virt_addr_to_phys_addr(virt_addr), Ok(phys_addr));
    }
}
