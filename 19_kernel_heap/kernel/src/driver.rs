// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>

//! Driver support.

use crate::{
    exception, info,
    synchronization::{interface::ReadWriteEx, InitStateLock},
};
use alloc::vec::Vec;
use core::fmt;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Driver interfaces.
pub mod interface {
    /// Device Driver functions.
    pub trait DeviceDriver {
        /// Different interrupt controllers might use different types for IRQ number.
        type IRQNumberType: super::fmt::Display;

        /// Return a compatibility string for identifying the driver.
        fn compatible(&self) -> &'static str;

        /// Called by the kernel to bring up the device.
        ///
        /// # Safety
        ///
        /// - During init, drivers might do stuff with system-wide impact.
        unsafe fn init(&self) -> Result<(), &'static str> {
            Ok(())
        }

        /// Called by the kernel to register and enable the device's IRQ handler.
        ///
        /// Rust's type system will prevent a call to this function unless the calling instance
        /// itself has static lifetime.
        fn register_and_enable_irq_handler(
            &'static self,
            irq_number: &Self::IRQNumberType,
        ) -> Result<(), &'static str> {
            panic!(
                "Attempt to enable IRQ {} for device {}, but driver does not support this",
                irq_number,
                self.compatible()
            )
        }
    }
}

/// Tpye to be used as an optional callback after a driver's init() has run.
pub type DeviceDriverPostInitCallback = unsafe fn() -> Result<(), &'static str>;

/// A descriptor for device drivers.
pub struct DeviceDriverDescriptor<T>
where
    T: 'static,
{
    device_driver: &'static (dyn interface::DeviceDriver<IRQNumberType = T> + Sync),
    post_init_callback: Option<DeviceDriverPostInitCallback>,
    irq_number: Option<T>,
}

/// Provides device driver management functions.
pub struct DriverManager<T>
where
    T: 'static,
{
    descriptors: InitStateLock<Vec<DeviceDriverDescriptor<T>>>,
}

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

static DRIVER_MANAGER: DriverManager<exception::asynchronous::IRQNumber> = DriverManager::new();

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

impl<T> DeviceDriverDescriptor<T> {
    /// Create an instance.
    pub fn new(
        device_driver: &'static (dyn interface::DeviceDriver<IRQNumberType = T> + Sync),
        post_init_callback: Option<DeviceDriverPostInitCallback>,
        irq_number: Option<T>,
    ) -> Self {
        Self {
            device_driver,
            post_init_callback,
            irq_number,
        }
    }
}

/// Return a reference to the global DriverManager.
pub fn driver_manager() -> &'static DriverManager<exception::asynchronous::IRQNumber> {
    &DRIVER_MANAGER
}

impl<T> DriverManager<T>
where
    T: fmt::Display,
{
    /// Create an instance.
    pub const fn new() -> Self {
        Self {
            descriptors: InitStateLock::new(Vec::new()),
        }
    }

    /// Register a device driver with the kernel.
    pub fn register_driver(&self, descriptor: DeviceDriverDescriptor<T>) {
        self.descriptors
            .write(|descriptors| descriptors.push(descriptor));
    }

    /// Fully initialize all drivers and their interrupts handlers.
    ///
    /// # Safety
    ///
    /// - During init, drivers might do stuff with system-wide impact.
    pub unsafe fn init_drivers_and_irqs(&self) {
        self.descriptors.read(|descriptors| {
            for descriptor in descriptors {
                // 1. Initialize driver.
                if let Err(x) = descriptor.device_driver.init() {
                    panic!(
                        "Error initializing driver: {}: {}",
                        descriptor.device_driver.compatible(),
                        x
                    );
                }

                // 2. Call corresponding post init callback.
                if let Some(callback) = &descriptor.post_init_callback {
                    if let Err(x) = callback() {
                        panic!(
                            "Error during driver post-init callback: {}: {}",
                            descriptor.device_driver.compatible(),
                            x
                        );
                    }
                }
            }

            // 3. After all post-init callbacks were done, the interrupt controller should be
            //    registered and functional. So let drivers register with it now.
            for descriptor in descriptors {
                if let Some(irq_number) = &descriptor.irq_number {
                    if let Err(x) = descriptor
                        .device_driver
                        .register_and_enable_irq_handler(irq_number)
                    {
                        panic!(
                            "Error during driver interrupt handler registration: {}: {}",
                            descriptor.device_driver.compatible(),
                            x
                        );
                    }
                }
            }
        })
    }

    /// Enumerate all registered device drivers.
    pub fn enumerate(&self) {
        self.descriptors.read(|descriptors| {
            for (i, desc) in descriptors.iter().enumerate() {
                info!("      {}. {}", i + 1, desc.device_driver.compatible());
            }
        });
    }
}
