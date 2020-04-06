// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! Top-level BSP file for the Raspberry Pi 3 and 4.

pub mod console;
pub mod cpu;
pub mod driver;
pub mod exception;
pub mod memory;

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------
use super::device_driver;

static GPIO: device_driver::GPIO =
    unsafe { device_driver::GPIO::new(memory::map::mmio::GPIO_BASE) };

static PL011_UART: device_driver::PL011Uart = unsafe {
    device_driver::PL011Uart::new(
        memory::map::mmio::PL011_UART_BASE,
        exception::asynchronous::irq_map::PL011_UART,
    )
};

#[cfg(feature = "bsp_rpi3")]
static INTERRUPT_CONTROLLER: device_driver::InterruptController = unsafe {
    device_driver::InterruptController::new(
        memory::map::mmio::LOCAL_INTERRUPT_CONTROLLER_BASE,
        memory::map::mmio::PERIPHERAL_INTERRUPT_CONTROLLER_BASE,
    )
};

#[cfg(feature = "bsp_rpi4")]
static INTERRUPT_CONTROLLER: device_driver::GICv2 = unsafe {
    device_driver::GICv2::new(memory::map::mmio::GICD_BASE, memory::map::mmio::GICC_BASE)
};

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Board identification.
pub fn board_name() -> &'static str {
    #[cfg(feature = "bsp_rpi3")]
    {
        "Raspberry Pi 3"
    }

    #[cfg(feature = "bsp_rpi4")]
    {
        "Raspberry Pi 4"
    }
}
