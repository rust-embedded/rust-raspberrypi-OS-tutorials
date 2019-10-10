// SPDX-License-Identifier: MIT
//
// Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>

//! Drivers.

#[cfg(feature = "bsp_rpi3")]
mod bcm2837_gpio;

#[cfg(feature = "bsp_rpi3")]
mod bcm2xxx_mini_uart;

#[cfg(feature = "bsp_rpi3")]
pub use bcm2837_gpio::GPIO;

#[cfg(feature = "bsp_rpi3")]
pub use bcm2xxx_mini_uart::MiniUart;
