// SPDX-License-Identifier: MIT
//
// Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>

//! Conditional exporting of Board Support Packages.

pub mod driver;

#[cfg(any(feature = "bsp_rpi3", feature = "bsp_rpi4"))]
mod rpi;

#[cfg(any(feature = "bsp_rpi3", feature = "bsp_rpi4"))]
pub use rpi::*;
