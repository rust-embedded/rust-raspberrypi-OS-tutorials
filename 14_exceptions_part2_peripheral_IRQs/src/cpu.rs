// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020-2021 Andre Richter <andre.o.richter@gmail.com>

//! Processor code.

#[cfg(target_arch = "aarch64")]
#[path = "_arch/aarch64/cpu.rs"]
mod arch_cpu;

mod boot;

pub mod smp;

//--------------------------------------------------------------------------------------------------
// Architectural Public Reexports
//--------------------------------------------------------------------------------------------------
pub use arch_cpu::{nop, qemu_exit_failure, qemu_exit_success, wait_forever};
