// SPDX-License-Identifier: MIT
//
// Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>

//! Conditional exporting of Board Support Packages.

#[cfg(feature = "bsp_rpi3")]
mod rpi;

#[cfg(feature = "bsp_rpi3")]
pub use rpi::*;
