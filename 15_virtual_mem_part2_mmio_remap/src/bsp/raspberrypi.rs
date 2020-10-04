// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! Top-level BSP file for the Raspberry Pi 3 and 4.

pub mod console;
pub mod cpu;
pub mod driver;
pub mod exception;
pub mod memory;

use super::device_driver;
use crate::memory::mmu::MMIODescriptor;
use memory::map::mmio;

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

static GPIO: device_driver::GPIO =
    unsafe { device_driver::GPIO::new(MMIODescriptor::new(mmio::GPIO_START, mmio::GPIO_SIZE)) };

static PL011_UART: device_driver::PL011Uart = unsafe {
    device_driver::PL011Uart::new(
        MMIODescriptor::new(mmio::PL011_UART_START, mmio::PL011_UART_SIZE),
        exception::asynchronous::irq_map::PL011_UART,
    )
};

#[cfg(feature = "bsp_rpi3")]
static INTERRUPT_CONTROLLER: device_driver::InterruptController = unsafe {
    device_driver::InterruptController::new(
        MMIODescriptor::new(mmio::LOCAL_IC_START, mmio::LOCAL_IC_SIZE),
        MMIODescriptor::new(mmio::PERIPHERAL_IC_START, mmio::PERIPHERAL_IC_SIZE),
    )
};

#[cfg(feature = "bsp_rpi4")]
static INTERRUPT_CONTROLLER: device_driver::GICv2 = unsafe {
    device_driver::GICv2::new(
        MMIODescriptor::new(mmio::GICD_START, mmio::GICD_SIZE),
        MMIODescriptor::new(mmio::GICC_START, mmio::GICC_SIZE),
    )
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
