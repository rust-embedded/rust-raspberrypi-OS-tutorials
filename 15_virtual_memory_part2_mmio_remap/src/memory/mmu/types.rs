// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020 Andre Richter <andre.o.richter@gmail.com>

//! Memory Management Unit Types.

use crate::{bsp, common};
use core::{convert::From, marker::PhantomData, ops::RangeInclusive};

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------
use super::interface::TranslationGranule;

/// Metadata trait for marking the type of an address.
pub trait AddressType: Copy + Clone + PartialOrd + PartialEq {}

/// Zero-sized type to mark a physical address.
#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub enum Physical {}

/// Zero-sized type to mark a virtual address.
#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub enum Virtual {}

/// Generic address type.
#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub struct Address<ATYPE: AddressType> {
    value: usize,
    _address_type: PhantomData<ATYPE>,
}

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
pub struct MMIODescriptor<ATYPE: AddressType> {
    start_addr: Address<ATYPE>,
    size: usize,
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

impl AddressType for Physical {}
impl AddressType for Virtual {}

//------------------------------------------------------------------------------
// Address
//------------------------------------------------------------------------------

impl<ATYPE: AddressType> Address<ATYPE> {
    /// Create an instance.
    pub const fn new(value: usize) -> Self {
        Self {
            value,
            _address_type: PhantomData,
        }
    }

    /// Align down.
    pub const fn align_down(self, alignment: usize) -> Self {
        let aligned = common::align_down(self.value, alignment);

        Self {
            value: aligned,
            _address_type: PhantomData,
        }
    }

    /// Converts `Address` into an usize.
    pub const fn into_usize(self) -> usize {
        self.value
    }
}

impl<ATYPE: AddressType> core::ops::Add<usize> for Address<ATYPE> {
    type Output = Self;

    fn add(self, other: usize) -> Self {
        Self {
            value: self.value + other,
            _address_type: PhantomData,
        }
    }
}

impl<ATYPE: AddressType> core::ops::Sub<usize> for Address<ATYPE> {
    type Output = Self;

    fn sub(self, other: usize) -> Self {
        Self {
            value: self.value - other,
            _address_type: PhantomData,
        }
    }
}

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
    const fn first_page_ptr(&self) -> *const Page<ATYPE> {
        self.start.into_usize() as *const _
    }

    /// Return the number of Pages the slice describes.
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

    /// Return a non-mutable slice of Pages.
    ///
    /// # Safety
    ///
    /// - Same as applies for `core::slice::from_raw_parts`.
    pub unsafe fn as_slice(&self) -> &[Page<ATYPE>] {
        core::slice::from_raw_parts(self.first_page_ptr(), self.num_pages)
    }

    /// Return the inclusive address range of the slice.
    pub fn into_usize_range_inclusive(self) -> RangeInclusive<usize> {
        RangeInclusive::new(
            self.start_addr().into_usize(),
            self.end_addr_inclusive().into_usize(),
        )
    }
}

impl From<PageSliceDescriptor<Virtual>> for PageSliceDescriptor<Physical> {
    fn from(desc: PageSliceDescriptor<Virtual>) -> Self {
        Self {
            start: Address::new(desc.start.into_usize()),
            num_pages: desc.num_pages,
        }
    }
}

impl<ATYPE: AddressType> From<MMIODescriptor<ATYPE>> for PageSliceDescriptor<ATYPE> {
    fn from(desc: MMIODescriptor<ATYPE>) -> Self {
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

impl<ATYPE: AddressType> MMIODescriptor<ATYPE> {
    /// Create an instance.
    pub const fn new(start_addr: Address<ATYPE>, size: usize) -> Self {
        assert!(size > 0);

        Self { start_addr, size }
    }

    /// Return the start address.
    pub const fn start_addr(&self) -> Address<ATYPE> {
        self.start_addr
    }

    /// Return the inclusive end address.
    pub fn end_addr_inclusive(&self) -> Address<ATYPE> {
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
