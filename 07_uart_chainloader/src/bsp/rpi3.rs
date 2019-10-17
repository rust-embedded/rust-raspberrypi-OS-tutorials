// SPDX-License-Identifier: MIT
//
// Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>

//! Board Support Package for the Raspberry Pi 3.

mod memory_map;

use super::driver;
use crate::interface;

pub const BOOT_CORE_ID: u64 = 0;
pub const BOOT_CORE_STACK_START: u64 = 0x80_000;

/// The address on which the RPi3 firmware loads every binary by default.
pub const BOARD_DEFAULT_LOAD_ADDRESS: usize = 0x80_000;

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

/// The BSP's main initialization function.
///
/// Called early on kernel start.
pub fn init() {
    for i in device_drivers().iter() {
        if let Err(()) = i.init() {
            // This message will only be readable if, at the time of failure, the return value of
            // `bsp::console()` is already in functioning state.
            panic!("Error loading driver: {}", i.compatible())
        }
    }

    // Configure MiniUart's output pins.
    GPIO.map_mini_uart();
}
