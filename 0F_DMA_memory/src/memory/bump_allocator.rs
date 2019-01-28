/*
 * MIT License
 *
 * Copyright (c) 2019 Andre Richter <andre.o.richter@gmail.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

use crate::println;
use core::alloc::{Alloc, AllocErr, Layout};
use core::mem;
use core::ptr::NonNull;
use core::slice;

pub struct BumpAllocator {
    next: usize,
    pool_end: usize,
    name: &'static str,
}

unsafe impl Alloc for BumpAllocator {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
        let start = super::aligned_addr_unchecked(self.next, layout.align());
        let end = start + layout.size();

        if end <= self.pool_end {
            self.next = end;

            println!(
                "[i] {}:\n      Allocated Addr {:#010X} Size {:#X}",
                self.name,
                start,
                layout.size()
            );

            Ok(NonNull::new_unchecked(start as *mut u8))
        } else {
            Err(AllocErr)
        }
    }

    // A bump allocator doesn't care
    unsafe fn dealloc(&mut self, _ptr: NonNull<u8>, _layout: Layout) {}
}

impl BumpAllocator {
    pub const fn new(pool_start: usize, pool_end: usize, name: &'static str) -> Self {
        Self {
            next: pool_start,
            pool_end,
            name,
        }
    }

    /// Allocate a zeroed slice
    pub fn alloc_slice_zeroed<'a, T>(
        &mut self,
        count_of_items: usize,
        alignment: usize,
    ) -> Result<&'a mut [T], ()> {
        let l;
        let size_in_byte = count_of_items * mem::size_of::<T>();
        match Layout::from_size_align(size_in_byte, alignment) {
            Ok(layout) => l = layout,

            Err(_) => {
                println!("[e] Layout Error!");
                return Err(());
            }
        }

        let ptr;
        match unsafe { self.alloc_zeroed(l) } {
            Ok(i) => ptr = i.as_ptr(),

            Err(_) => {
                println!("[e] Layout Error!");
                return Err(());
            }
        }

        Ok(unsafe { slice::from_raw_parts_mut(ptr as *mut T, count_of_items) })
    }
}
