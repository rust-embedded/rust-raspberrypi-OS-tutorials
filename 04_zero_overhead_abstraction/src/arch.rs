// SPDX-License-Identifier: MIT
//
// Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>

//! Conditional exporting of processor architecture code.

#[cfg(feature = "bsp_rpi3")]
mod aarch64;

#[cfg(feature = "bsp_rpi3")]
pub use aarch64::*;
