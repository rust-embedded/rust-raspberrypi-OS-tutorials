// SPDX-License-Identifier: MIT
//
// Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>

//! Collection of device drivers.

mod gpio;
mod mini_uart;

pub use gpio::GPIO;
pub use mini_uart::MiniUart;
