// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2022-2023 Andre Richter <andre.o.richter@gmail.com>

//! Types for implementing debug symbol support.

#![no_std]

use core::ops::Range;

/// A symbol containing a size.
#[repr(C)]
#[derive(Clone)]
pub struct Symbol {
    addr_range: Range<usize>,
    name: &'static str,
}

impl Symbol {
    /// Create an instance.
    pub const fn new(start: usize, size: usize, name: &'static str) -> Symbol {
        Symbol {
            addr_range: Range {
                start,
                end: start + size,
            },
            name,
        }
    }

    /// Returns true if addr is contained in the range.
    pub fn contains(&self, addr: usize) -> bool {
        self.addr_range.contains(&addr)
    }

    /// Returns the symbol's name.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Returns the symbol's size.
    pub fn size(&self) -> usize {
        self.addr_range.end - self.addr_range.start
    }
}
