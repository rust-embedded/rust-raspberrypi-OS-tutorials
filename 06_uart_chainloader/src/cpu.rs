// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020-2023 Andre Richter <andre.o.richter@gmail.com>

//! Processor code.

#[cfg(target_arch = "aarch64")]
#[path = "_arch/aarch64/cpu.rs"]
mod arch_cpu;

mod boot;

//--------------------------------------------------------------------------------------------------
// Architectural Public Reexports
//--------------------------------------------------------------------------------------------------
pub use arch_cpu::{nop, wait_forever};

#[cfg(feature = "bsp_rpi3")]
pub use arch_cpu::spin_for_cycles;
