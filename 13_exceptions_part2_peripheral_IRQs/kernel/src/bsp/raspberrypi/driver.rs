// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2022 Andre Richter <andre.o.richter@gmail.com>

//! BSP driver support.

use super::{exception, memory::map::mmio};
use crate::{bsp::device_driver, driver};

pub use device_driver::IRQNumber;

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

/// Device Driver Manager type.
struct BSPDriverManager {
    device_drivers: [&'static (dyn DeviceDriver + Sync); 3],
}

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

pub(super) static PL011_UART: device_driver::PL011Uart = unsafe {
    device_driver::PL011Uart::new(
        mmio::PL011_UART_START,
        exception::asynchronous::irq_map::PL011_UART,
    )
};

static GPIO: device_driver::GPIO = unsafe { device_driver::GPIO::new(mmio::GPIO_START) };

#[cfg(feature = "bsp_rpi3")]
pub(super) static INTERRUPT_CONTROLLER: device_driver::InterruptController =
    unsafe { device_driver::InterruptController::new(mmio::PERIPHERAL_INTERRUPT_CONTROLLER_START) };

#[cfg(feature = "bsp_rpi4")]
pub(super) static INTERRUPT_CONTROLLER: device_driver::GICv2 =
    unsafe { device_driver::GICv2::new(mmio::GICD_START, mmio::GICC_START) };

static BSP_DRIVER_MANAGER: BSPDriverManager = BSPDriverManager {
    device_drivers: [&PL011_UART, &GPIO, &INTERRUPT_CONTROLLER],
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
        GPIO.map_pl011_uart();
    }

    #[cfg(feature = "test_build")]
    fn qemu_bring_up_console(&self) {}
}
