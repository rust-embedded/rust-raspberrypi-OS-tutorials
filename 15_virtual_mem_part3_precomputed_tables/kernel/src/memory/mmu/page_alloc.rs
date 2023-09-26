// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2021-2023 Andre Richter <andre.o.richter@gmail.com>

//! Page allocation.

use super::MemoryRegion;
use crate::{
    memory::{AddressType, Virtual},
    synchronization::IRQSafeNullLock,
    warn,
};
use core::num::NonZeroUsize;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// A page allocator that can be lazyily initialized.
pub struct PageAllocator<ATYPE: AddressType> {
    pool: Option<MemoryRegion<ATYPE>>,
}

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

static KERNEL_MMIO_VA_ALLOCATOR: IRQSafeNullLock<PageAllocator<Virtual>> =
    IRQSafeNullLock::new(PageAllocator::new());

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Return a reference to the kernel's MMIO virtual address allocator.
pub fn kernel_mmio_va_allocator() -> &'static IRQSafeNullLock<PageAllocator<Virtual>> {
    &KERNEL_MMIO_VA_ALLOCATOR
}

impl<ATYPE: AddressType> PageAllocator<ATYPE> {
    /// Create an instance.
    pub const fn new() -> Self {
        Self { pool: None }
    }

    /// Initialize the allocator.
    pub fn init(&mut self, pool: MemoryRegion<ATYPE>) {
        if self.pool.is_some() {
            warn!("Already initialized");
            return;
        }

        self.pool = Some(pool);
    }

    /// Allocate a number of pages.
    pub fn alloc(
        &mut self,
        num_requested_pages: NonZeroUsize,
    ) -> Result<MemoryRegion<ATYPE>, &'static str> {
        if self.pool.is_none() {
            return Err("Allocator not initialized");
        }

        self.pool
            .as_mut()
            .unwrap()
            .take_first_n_pages(num_requested_pages)
    }
}
