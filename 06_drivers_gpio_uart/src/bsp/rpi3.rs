// SPDX-License-Identifier: MIT
//
// Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>

//! Board Support Package for the Raspberry Pi 3.

mod memory_map;

use super::driver;
use crate::interface;

pub const BOOT_CORE_ID: u64 = 0;
pub const BOOT_CORE_STACK_START: u64 = 0x80_000;

//--------------------------------------------------------------------------------------------------
// Global BSP driver instances
//--------------------------------------------------------------------------------------------------

static GPIO: driver::GPIO = unsafe { driver::GPIO::new(memory_map::mmio::GPIO_BASE) };
static MINI_UART: driver::MiniUart =
    unsafe { driver::MiniUart::new(memory_map::mmio::MINI_UART_BASE) };

//--------------------------------------------------------------------------------------------------
// Implementation of the kernel's BSP calls
//--------------------------------------------------------------------------------------------------

/// Board identification.
pub fn board_name() -> &'static str {
    "Raspberry Pi 3"
}

/// Return a reference to a `console::All` implementation.
pub fn console() -> &'static impl interface::console::All {
    &MINI_UART
}

/// Return an array of references to all `DeviceDriver` compatible `BSP` drivers.
///
/// # Safety
///
/// The order of devices is the order in which `DeviceDriver::init()` is called.
pub fn device_drivers() -> [&'static dyn interface::driver::DeviceDriver; 2] {
    [&GPIO, &MINI_UART]
}

/// BSP initialization code that runs after driver init.
pub fn post_driver_init() {
    // Configure MiniUart's output pins.
    GPIO.map_mini_uart();
}
