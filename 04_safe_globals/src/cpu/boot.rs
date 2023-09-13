// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2021-2023 Andre Richter <andre.o.richter@gmail.com>

//! Boot code.

#[cfg(target_arch = "aarch64")]
#[path = "../_arch/aarch64/cpu/boot.rs"]
mod arch_boot;
