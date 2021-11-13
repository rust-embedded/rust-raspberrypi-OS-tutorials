// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020-2021 Andre Richter <andre.o.richter@gmail.com>

//! Memory Management Unit types.

use crate::{
    bsp, common,
    memory::{Address, AddressType, Physical},
};
use core::{convert::From, marker::PhantomData};

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Generic page type.
#[repr(C)]
pub struct Page<ATYPE: AddressType> {
    inner: [u8; bsp::memory::mmu::KernelGranule::SIZE],
    _address_type: PhantomData<ATYPE>,
}

/// Type describing a slice of pages.
#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub struct PageSliceDescriptor<ATYPE: AddressType> {
    start: Address<ATYPE>,
    num_pages: usize,
}

/// Architecture agnostic memory attributes.
#[allow(missing_docs)]
#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub enum MemAttributes {
    CacheableDRAM,
    Device,
}

/// Architecture agnostic access permissions.
#[allow(missing_docs)]
#[derive(Copy, Clone)]
pub enum AccessPermissions {
    ReadOnly,
    ReadWrite,
}

/// Collection of memory attributes.
#[allow(missing_docs)]
#[derive(Copy, Clone)]
pub struct AttributeFields {
    pub mem_attributes: MemAttributes,
    pub acc_perms: AccessPermissions,
    pub execute_never: bool,
}

/// An MMIO descriptor for use in device drivers.
#[derive(Copy, Clone)]
pub struct MMIODescriptor {
    start_addr: Address<Physical>,
    size: usize,
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

//------------------------------------------------------------------------------
// Page
//------------------------------------------------------------------------------

impl<ATYPE: AddressType> Page<ATYPE> {
    /// Get a pointer to the instance.
    pub const fn as_ptr(&self) -> *const Page<ATYPE> {
        self as *const _
    }
}

//------------------------------------------------------------------------------
// PageSliceDescriptor
//------------------------------------------------------------------------------

impl<ATYPE: AddressType> PageSliceDescriptor<ATYPE> {
    /// Create an instance.
    pub const fn from_addr(start: Address<ATYPE>, num_pages: usize) -> Self {
        assert!(common::is_aligned(
            start.into_usize(),
            bsp::memory::mmu::KernelGranule::SIZE
        ));
        assert!(num_pages > 0);

        Self { start, num_pages }
    }

    /// Return a pointer to the first page of the described slice.
    const fn first_page(&self) -> *const Page<ATYPE> {
        self.start.into_usize() as *const _
    }

    /// Return the number of pages the slice describes.
    pub const fn num_pages(&self) -> usize {
        self.num_pages
    }

    /// Return the memory size this descriptor spans.
    pub const fn size(&self) -> usize {
        self.num_pages * bsp::memory::mmu::KernelGranule::SIZE
    }

    /// Return the start address.
    pub const fn start_addr(&self) -> Address<ATYPE> {
        self.start
    }

    /// Return the exclusive end address.
    pub fn end_addr(&self) -> Address<ATYPE> {
        self.start + self.size()
    }

    /// Return the inclusive end address.
    pub fn end_addr_inclusive(&self) -> Address<ATYPE> {
        self.start + (self.size() - 1)
    }

    /// Check if an address is contained within this descriptor.
    pub fn contains(&self, addr: Address<ATYPE>) -> bool {
        (addr >= self.start_addr()) && (addr <= self.end_addr_inclusive())
    }

    /// Return a non-mutable slice of pages.
    ///
    /// # Safety
    ///
    /// - Same as applies for `core::slice::from_raw_parts`.
    pub unsafe fn as_slice(&self) -> &[Page<ATYPE>] {
        core::slice::from_raw_parts(self.first_page(), self.num_pages)
    }
}

impl From<MMIODescriptor> for PageSliceDescriptor<Physical> {
    fn from(desc: MMIODescriptor) -> Self {
        let start_page_addr = desc
            .start_addr
            .align_down(bsp::memory::mmu::KernelGranule::SIZE);

        let len = ((desc.end_addr_inclusive().into_usize() - start_page_addr.into_usize())
            >> bsp::memory::mmu::KernelGranule::SHIFT)
            + 1;

        Self {
            start: start_page_addr,
            num_pages: len,
        }
    }
}

//------------------------------------------------------------------------------
// MMIODescriptor
//------------------------------------------------------------------------------

impl MMIODescriptor {
    /// Create an instance.
    pub const fn new(start_addr: Address<Physical>, size: usize) -> Self {
        assert!(size > 0);

        Self { start_addr, size }
    }

    /// Return the start address.
    pub const fn start_addr(&self) -> Address<Physical> {
        self.start_addr
    }

    /// Return the inclusive end address.
    pub fn end_addr_inclusive(&self) -> Address<Physical> {
        self.start_addr + (self.size - 1)
    }

    /// Return the size.
    pub const fn size(&self) -> usize {
        self.size
    }
}

//--------------------------------------------------------------------------------------------------
// Testing
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use test_macros::kernel_test;

    /// Check if the size of `struct Page` is as expected.
    #[kernel_test]
    fn size_of_page_equals_granule_size() {
        assert_eq!(
            core::mem::size_of::<Page<Physical>>(),
            bsp::memory::mmu::KernelGranule::SIZE
        );
    }
}
