// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! Memory Management.

pub mod mmu;

use crate::common;
use core::{
    fmt,
    marker::PhantomData,
    ops::{AddAssign, SubAssign},
};

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

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
    _address_type: PhantomData<fn() -> ATYPE>,
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

impl AddressType for Physical {}
impl AddressType for Virtual {}

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

impl<ATYPE: AddressType> AddAssign for Address<ATYPE> {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            value: self.value + other.into_usize(),
            _address_type: PhantomData,
        };
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

impl<ATYPE: AddressType> SubAssign for Address<ATYPE> {
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            value: self.value - other.into_usize(),
            _address_type: PhantomData,
        };
    }
}

impl fmt::Display for Address<Physical> {
    // Don't expect to see physical addresses greater than 40 bit.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let q3: u8 = ((self.value >> 32) & 0xff) as u8;
        let q2: u16 = ((self.value >> 16) & 0xffff) as u16;
        let q1: u16 = (self.value & 0xffff) as u16;

        write!(f, "0x")?;
        write!(f, "{:02x}_", q3)?;
        write!(f, "{:04x}_", q2)?;
        write!(f, "{:04x}", q1)
    }
}

impl fmt::Display for Address<Virtual> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let q4: u16 = ((self.value >> 48) & 0xffff) as u16;
        let q3: u16 = ((self.value >> 32) & 0xffff) as u16;
        let q2: u16 = ((self.value >> 16) & 0xffff) as u16;
        let q1: u16 = (self.value & 0xffff) as u16;

        write!(f, "0x")?;
        write!(f, "{:04x}_", q4)?;
        write!(f, "{:04x}_", q3)?;
        write!(f, "{:04x}_", q2)?;
        write!(f, "{:04x}", q1)
    }
}
