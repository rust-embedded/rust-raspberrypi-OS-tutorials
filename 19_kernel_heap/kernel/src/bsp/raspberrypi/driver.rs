// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2022 Andre Richter <andre.o.richter@gmail.com>

//! BSP driver support.

use super::{exception, memory::map::mmio};
use crate::{
    bsp::device_driver,
    console, driver, exception as generic_exception, memory,
    memory::mmu::MMIODescriptor,
    synchronization::{interface::ReadWriteEx, InitStateLock},
};
use alloc::vec::Vec;
use core::{
    mem::MaybeUninit,
    sync::atomic::{AtomicBool, Ordering},
};
pub use device_driver::IRQNumber;

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

/// Device Driver Manager type.
struct BSPDriverManager {
    device_drivers: InitStateLock<Vec<&'static (dyn DeviceDriver + Sync)>>,
    init_done: AtomicBool,
}

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

static mut PL011_UART: MaybeUninit<device_driver::PL011Uart> = MaybeUninit::uninit();
static mut GPIO: MaybeUninit<device_driver::GPIO> = MaybeUninit::uninit();

#[cfg(feature = "bsp_rpi3")]
static mut INTERRUPT_CONTROLLER: MaybeUninit<device_driver::InterruptController> =
    MaybeUninit::uninit();

#[cfg(feature = "bsp_rpi4")]
static mut INTERRUPT_CONTROLLER: MaybeUninit<device_driver::GICv2> = MaybeUninit::uninit();

static BSP_DRIVER_MANAGER: BSPDriverManager = BSPDriverManager {
    device_drivers: InitStateLock::new(Vec::new()),
    init_done: AtomicBool::new(false),
};

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

impl BSPDriverManager {
    unsafe fn instantiate_uart(&self) -> Result<(), &'static str> {
        let mmio_descriptor = MMIODescriptor::new(mmio::PL011_UART_START, mmio::PL011_UART_SIZE);
        let virt_addr =
            memory::mmu::kernel_map_mmio(device_driver::PL011Uart::COMPATIBLE, &mmio_descriptor)?;

        // This is safe to do, because it is only called from the init'ed instance itself.
        fn uart_post_init() {
            console::register_console(unsafe { PL011_UART.assume_init_ref() });
        }

        PL011_UART.write(device_driver::PL011Uart::new(
            virt_addr,
            exception::asynchronous::irq_map::PL011_UART,
            uart_post_init,
        ));

        Ok(())
    }

    unsafe fn instantiate_gpio(&self) -> Result<(), &'static str> {
        let mmio_descriptor = MMIODescriptor::new(mmio::GPIO_START, mmio::GPIO_SIZE);
        let virt_addr =
            memory::mmu::kernel_map_mmio(device_driver::GPIO::COMPATIBLE, &mmio_descriptor)?;

        // This is safe to do, because it is only called from the init'ed instance itself.
        fn gpio_post_init() {
            unsafe { GPIO.assume_init_ref().map_pl011_uart() };
        }

        GPIO.write(device_driver::GPIO::new(virt_addr, gpio_post_init));

        Ok(())
    }

    #[cfg(feature = "bsp_rpi3")]
    unsafe fn instantiate_interrupt_controller(&self) -> Result<(), &'static str> {
        let periph_mmio_descriptor =
            MMIODescriptor::new(mmio::PERIPHERAL_IC_START, mmio::PERIPHERAL_IC_SIZE);
        let periph_virt_addr = memory::mmu::kernel_map_mmio(
            device_driver::InterruptController::COMPATIBLE,
            &periph_mmio_descriptor,
        )?;

        // This is safe to do, because it is only called from the init'ed instance itself.
        fn interrupt_controller_post_init() {
            generic_exception::asynchronous::register_irq_manager(unsafe {
                INTERRUPT_CONTROLLER.assume_init_ref()
            });
        }

        INTERRUPT_CONTROLLER.write(device_driver::InterruptController::new(
            periph_virt_addr,
            interrupt_controller_post_init,
        ));

        Ok(())
    }

    #[cfg(feature = "bsp_rpi4")]
    unsafe fn instantiate_interrupt_controller(&self) -> Result<(), &'static str> {
        let gicd_mmio_descriptor = MMIODescriptor::new(mmio::GICD_START, mmio::GICD_SIZE);
        let gicd_virt_addr = memory::mmu::kernel_map_mmio("GICv2 GICD", &gicd_mmio_descriptor)?;

        let gicc_mmio_descriptor = MMIODescriptor::new(mmio::GICC_START, mmio::GICC_SIZE);
        let gicc_virt_addr = memory::mmu::kernel_map_mmio("GICV2 GICC", &gicc_mmio_descriptor)?;

        // This is safe to do, because it is only called from the init'ed instance itself.
        fn interrupt_controller_post_init() {
            generic_exception::asynchronous::register_irq_manager(unsafe {
                INTERRUPT_CONTROLLER.assume_init_ref()
            });
        }

        INTERRUPT_CONTROLLER.write(device_driver::GICv2::new(
            gicd_virt_addr,
            gicc_virt_addr,
            interrupt_controller_post_init,
        ));

        Ok(())
    }

    unsafe fn register_drivers(&self) {
        self.device_drivers.write(|drivers| {
            drivers.push(PL011_UART.assume_init_ref());
            drivers.push(GPIO.assume_init_ref());
            drivers.push(INTERRUPT_CONTROLLER.assume_init_ref());
        });
    }
}

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
    unsafe fn instantiate_drivers(&self) -> Result<(), &'static str> {
        if self.init_done.load(Ordering::Relaxed) {
            return Err("Drivers already instantiated");
        }

        self.instantiate_uart()?;
        self.instantiate_gpio()?;
        self.instantiate_interrupt_controller()?;

        self.register_drivers();

        self.init_done.store(true, Ordering::Relaxed);
        Ok(())
    }

    fn all_device_drivers(&self) -> &Vec<&(dyn DeviceDriver + Sync)> {
        self.device_drivers.read(|drivers| drivers)
    }

    #[cfg(feature = "test_build")]
    fn qemu_bring_up_console(&self) {
        use crate::cpu;

        unsafe {
            self.instantiate_uart()
                .unwrap_or_else(|_| cpu::qemu_exit_failure());
            console::register_console(PL011_UART.assume_init_ref());
        };
    }
}
