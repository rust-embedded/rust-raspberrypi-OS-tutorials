// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>

//! BSP driver support.

use super::{exception, memory::map::mmio};
use crate::{
    bsp::device_driver,
    console, driver as generic_driver,
    exception::{self as generic_exception},
};
use core::sync::atomic::{AtomicBool, Ordering};

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

static PL011_UART: device_driver::PL011Uart =
    unsafe { device_driver::PL011Uart::new(mmio::PL011_UART_START) };
static GPIO: device_driver::GPIO = unsafe { device_driver::GPIO::new(mmio::GPIO_START) };

#[cfg(feature = "bsp_rpi3")]
static INTERRUPT_CONTROLLER: device_driver::InterruptController =
    unsafe { device_driver::InterruptController::new(mmio::PERIPHERAL_IC_START) };

#[cfg(feature = "bsp_rpi4")]
static INTERRUPT_CONTROLLER: device_driver::GICv2 =
    unsafe { device_driver::GICv2::new(mmio::GICD_START, mmio::GICC_START) };

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

/// This must be called only after successful init of the UART driver.
fn post_init_uart() -> Result<(), &'static str> {
    console::register_console(&PL011_UART);

    Ok(())
}

/// This must be called only after successful init of the GPIO driver.
fn post_init_gpio() -> Result<(), &'static str> {
    GPIO.map_pl011_uart();
    Ok(())
}

/// This must be called only after successful init of the interrupt controller driver.
fn post_init_interrupt_controller() -> Result<(), &'static str> {
    generic_exception::asynchronous::register_irq_manager(&INTERRUPT_CONTROLLER);

    Ok(())
}

fn driver_uart() -> Result<(), &'static str> {
    let uart_descriptor = generic_driver::DeviceDriverDescriptor::new(
        &PL011_UART,
        Some(post_init_uart),
        Some(exception::asynchronous::irq_map::PL011_UART),
    );
    generic_driver::driver_manager().register_driver(uart_descriptor);

    Ok(())
}

fn driver_gpio() -> Result<(), &'static str> {
    let gpio_descriptor =
        generic_driver::DeviceDriverDescriptor::new(&GPIO, Some(post_init_gpio), None);
    generic_driver::driver_manager().register_driver(gpio_descriptor);

    Ok(())
}

fn driver_interrupt_controller() -> Result<(), &'static str> {
    let interrupt_controller_descriptor = generic_driver::DeviceDriverDescriptor::new(
        &INTERRUPT_CONTROLLER,
        Some(post_init_interrupt_controller),
        None,
    );
    generic_driver::driver_manager().register_driver(interrupt_controller_descriptor);

    Ok(())
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Initialize the driver subsystem.
///
/// # Safety
///
/// See child function calls.
pub unsafe fn init() -> Result<(), &'static str> {
    static INIT_DONE: AtomicBool = AtomicBool::new(false);
    if INIT_DONE.load(Ordering::Relaxed) {
        return Err("Init already done");
    }

    driver_uart()?;
    driver_gpio()?;
    driver_interrupt_controller()?;

    INIT_DONE.store(true, Ordering::Relaxed);
    Ok(())
}

/// Minimal code needed to bring up the console in QEMU (for testing only). This is often less steps
/// than on real hardware due to QEMU's abstractions.
#[cfg(feature = "test_build")]
pub fn qemu_bring_up_console() {
    console::register_console(&PL011_UART);
}
