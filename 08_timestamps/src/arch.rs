// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! Conditional exporting of processor architecture code.

#[cfg(any(feature = "bsp_rpi3", feature = "bsp_rpi4"))]
mod aarch64;

#[cfg(any(feature = "bsp_rpi3", feature = "bsp_rpi4"))]
pub use aarch64::*;
