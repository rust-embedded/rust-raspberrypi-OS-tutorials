// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! BSP driver support.

use crate::driver;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Device Driver Manager type.
pub struct BSPDriverManager {
    device_drivers: [&'static (dyn DeviceDriver + Sync); 3],
}

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

static BSP_DRIVER_MANAGER: BSPDriverManager = BSPDriverManager {
    device_drivers: [
        &super::GPIO,
        &super::PL011_UART,
        &super::INTERRUPT_CONTROLLER,
    ],
};

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Return a reference to the driver manager.
pub fn driver_manager() -> &'static impl driver::interface::DriverManager {
    &BSP_DRIVER_MANAGER
}

//------------------------------------------------------------------------------
// OS Interface Code
//------------------------------------------------------------------------------
use driver::interface::DeviceDriver;

impl driver::interface::DriverManager for BSPDriverManager {
    fn all_device_drivers(&self) -> &[&'static (dyn DeviceDriver + Sync)] {
        &self.device_drivers[..]
    }

    fn post_device_driver_init(&self) {
        // Configure PL011Uart's output pins.
        super::GPIO.map_pl011_uart();
    }
}
