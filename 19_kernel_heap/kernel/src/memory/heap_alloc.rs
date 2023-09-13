// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2022-2023 Andre Richter <andre.o.richter@gmail.com>

//! Heap allocation.

use crate::{
    backtrace, bsp, common, debug, info,
    memory::{Address, Virtual},
    synchronization,
    synchronization::IRQSafeNullLock,
    warn,
};
use alloc::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicBool, Ordering};
use linked_list_allocator::Heap as LinkedListHeap;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// A heap allocator that can be lazyily initialized.
pub struct HeapAllocator {
    inner: IRQSafeNullLock<LinkedListHeap>,
}

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

#[global_allocator]
static KERNEL_HEAP_ALLOCATOR: HeapAllocator = HeapAllocator::new();

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

#[inline(always)]
fn debug_print_alloc_dealloc(operation: &'static str, ptr: *mut u8, layout: Layout) {
    let size = layout.size();
    let (size_h, size_unit) = common::size_human_readable_ceil(size);
    let addr = Address::<Virtual>::new(ptr as usize);

    debug!(
        "Kernel Heap: {}\n      \
        Size:     {:#x} ({} {})\n      \
        Start:    {}\n      \
        End excl: {}\n\n      \
        {}",
        operation,
        size,
        size_h,
        size_unit,
        addr,
        addr + size,
        backtrace::Backtrace
    );
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------
use synchronization::interface::Mutex;

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("Allocation error: {:?}", layout)
}

/// Return a reference to the kernel's heap allocator.
pub fn kernel_heap_allocator() -> &'static HeapAllocator {
    &KERNEL_HEAP_ALLOCATOR
}

impl HeapAllocator {
    /// Create an instance.
    pub const fn new() -> Self {
        Self {
            inner: IRQSafeNullLock::new(LinkedListHeap::empty()),
        }
    }

    /// Print the current heap usage.
    pub fn print_usage(&self) {
        let (used, free) = KERNEL_HEAP_ALLOCATOR
            .inner
            .lock(|inner| (inner.used(), inner.free()));

        if used >= 1024 {
            let (used_h, used_unit) = common::size_human_readable_ceil(used);
            info!("      Used: {} Byte ({} {})", used, used_h, used_unit);
        } else {
            info!("      Used: {} Byte", used);
        }

        if free >= 1024 {
            let (free_h, free_unit) = common::size_human_readable_ceil(free);
            info!("      Free: {} Byte ({} {})", free, free_h, free_unit);
        } else {
            info!("      Free: {} Byte", free);
        }
    }
}

unsafe impl GlobalAlloc for HeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let result = KERNEL_HEAP_ALLOCATOR
            .inner
            .lock(|inner| inner.allocate_first_fit(layout).ok());

        match result {
            None => core::ptr::null_mut(),
            Some(allocation) => {
                let ptr = allocation.as_ptr();

                debug_print_alloc_dealloc("Allocation", ptr, layout);

                ptr
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        KERNEL_HEAP_ALLOCATOR
            .inner
            .lock(|inner| inner.deallocate(core::ptr::NonNull::new_unchecked(ptr), layout));

        debug_print_alloc_dealloc("Free", ptr, layout);
    }
}

/// Query the BSP for the heap region and initialize the kernel's heap allocator with it.
pub fn kernel_init_heap_allocator() {
    static INIT_DONE: AtomicBool = AtomicBool::new(false);
    if INIT_DONE.load(Ordering::Relaxed) {
        warn!("Already initialized");
        return;
    }

    let region = bsp::memory::mmu::virt_heap_region();

    KERNEL_HEAP_ALLOCATOR.inner.lock(|inner| unsafe {
        inner.init(region.start_addr().as_usize() as *mut u8, region.size())
    });

    INIT_DONE.store(true, Ordering::Relaxed);
}
