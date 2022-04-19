// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2022 Andre Richter <andre.o.richter@gmail.com>

//! BSP console facilities.

use super::memory;
use crate::{bsp::device_driver, console, cpu, driver};
use core::fmt;

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// In case of a panic, the panic handler uses this function to take a last shot at printing
/// something before the system is halted.
///
/// We try to init panic-versions of the GPIO and the UART. The panic versions are not protected
/// with synchronization primitives, which increases chances that we get to print something, even
/// when the kernel's default GPIO or UART instances happen to be locked at the time of the panic.
///
/// # Safety
///
/// - Use only for printing during a panic.
#[cfg(not(feature = "test_build"))]
pub unsafe fn panic_console_out() -> impl fmt::Write {
    use driver::interface::DeviceDriver;

    let mut panic_gpio = device_driver::PanicGPIO::new(memory::map::mmio::GPIO_START.as_usize());
    let mut panic_uart =
        device_driver::PanicUart::new(memory::map::mmio::PL011_UART_START.as_usize());

    // If remapping of the driver's MMIO already happened, take the remapped start address.
    // Otherwise, take a chance with the default physical address.
    let maybe_gpio_mmio_start_addr = super::GPIO.virt_mmio_start_addr();
    let maybe_uart_mmio_start_addr = super::PL011_UART.virt_mmio_start_addr();

    panic_gpio
        .init(maybe_gpio_mmio_start_addr)
        .unwrap_or_else(|_| cpu::wait_forever());
    panic_gpio.map_pl011_uart();
    panic_uart
        .init(maybe_uart_mmio_start_addr)
        .unwrap_or_else(|_| cpu::wait_forever());

    panic_uart
}

/// Reduced version for test builds.
///
/// # Safety
///
/// - Use only for printing during a panic.
#[cfg(feature = "test_build")]
pub unsafe fn panic_console_out() -> impl fmt::Write {
    use driver::interface::DeviceDriver;

    let mut panic_uart =
        device_driver::PanicUart::new(memory::map::mmio::PL011_UART_START.as_usize());

    let maybe_uart_mmio_start_addr = super::PL011_UART.virt_mmio_start_addr();

    panic_uart
        .init(maybe_uart_mmio_start_addr)
        .unwrap_or_else(|_| cpu::qemu_exit_failure());

    panic_uart
}

/// Return a reference to the console.
pub fn console() -> &'static impl console::interface::All {
    &super::PL011_UART
}

//--------------------------------------------------------------------------------------------------
// Testing
//--------------------------------------------------------------------------------------------------

/// Minimal code needed to bring up the console in QEMU (for testing only). This is often less steps
/// than on real hardware due to QEMU's abstractions.
#[cfg(feature = "test_build")]
pub fn qemu_bring_up_console() {
    use driver::interface::DeviceDriver;

    // Calling the UART's init ensures that the BSP's instance of the UART does remap the MMIO
    // addresses.
    unsafe {
        super::PL011_UART
            .init()
            .unwrap_or_else(|_| cpu::qemu_exit_failure());
    }
}
