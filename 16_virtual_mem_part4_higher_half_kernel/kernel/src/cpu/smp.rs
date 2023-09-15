// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>

//! Symmetric multiprocessing.

#[cfg(target_arch = "aarch64")]
#[path = "../_arch/aarch64/cpu/smp.rs"]
mod arch_smp;

//--------------------------------------------------------------------------------------------------
// Architectural Public Reexports
//--------------------------------------------------------------------------------------------------
pub use arch_smp::core_id;
