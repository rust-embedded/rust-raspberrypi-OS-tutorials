// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! Exception handling.

use cortex_a::regs::*;

//--------------------------------------------------------------------------------------------------
// Arch-public
//--------------------------------------------------------------------------------------------------

pub trait DaifField {
    fn daif_field() -> register::Field<u32, DAIF::Register>;
}

pub struct Debug;
pub struct SError;
pub struct IRQ;
pub struct FIQ;

impl DaifField for Debug {
    fn daif_field() -> register::Field<u32, DAIF::Register> {
        DAIF::D
    }
}

impl DaifField for SError {
    fn daif_field() -> register::Field<u32, DAIF::Register> {
        DAIF::A
    }
}

impl DaifField for IRQ {
    fn daif_field() -> register::Field<u32, DAIF::Register> {
        DAIF::I
    }
}

impl DaifField for FIQ {
    fn daif_field() -> register::Field<u32, DAIF::Register> {
        DAIF::F
    }
}

pub fn is_masked<T: DaifField>() -> bool {
    DAIF.is_set(T::daif_field())
}
