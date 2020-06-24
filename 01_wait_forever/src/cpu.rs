// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020 Andre Richter <andre.o.richter@gmail.com>

//! Processor code.

#[cfg(target_arch = "aarch64")]
#[path = "_arch/aarch64/cpu.rs"]
mod arch_cpu;

#[cfg(target_arch = "riscv32")]
#[path = "_arch/riscv32/cpu.rs"]
mod arch_cpu;

#[cfg(target_arch = "riscv64")]
#[path = "_arch/riscv64/cpu.rs"]
mod arch_cpu;

pub use arch_cpu::*;
