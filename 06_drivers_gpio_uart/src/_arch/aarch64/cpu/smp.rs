// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! Architectural symmetric multiprocessing.

use cortex_a::regs::*;

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Return the executing core's id.
#[inline(always)]
pub fn core_id<T>() -> T
where
    T: From<u8>,
{
    const CORE_MASK: u64 = 0b11;

    T::from((MPIDR_EL1.get() & CORE_MASK) as u8)
}
