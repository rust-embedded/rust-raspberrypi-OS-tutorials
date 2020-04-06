// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020 Andre Richter <andre.o.richter@gmail.com>

//! ARM driver top level.

#[cfg(feature = "bsp_rpi4")]
pub mod gicv2;

#[cfg(feature = "bsp_rpi4")]
pub use gicv2::*;
