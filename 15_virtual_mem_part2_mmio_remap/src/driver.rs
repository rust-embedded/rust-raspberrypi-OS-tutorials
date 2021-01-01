// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! Driver support.

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Driver interfaces.
pub mod interface {
    /// Device Driver functions.
    pub trait DeviceDriver {
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

        /// Called by the kernel to register and enable the device's IRQ handlers, if any.
        ///
        /// Rust's type system will prevent a call to this function unless the calling instance
        /// itself has static lifetime.
        fn register_and_enable_irq_handler(&'static self) -> Result<(), &'static str> {
            Ok(())
        }

        /// After MMIO remapping, returns the new virtual start address.
        ///
        /// This API assumes a driver has only a single, contiguous MMIO aperture, which will not be
        /// the case for more complex devices. This API will likely change in future tutorials.
        fn virt_mmio_start_addr(&self) -> Option<usize> {
            None
        }
    }

    /// Device driver management functions.
    ///
    /// The `BSP` is supposed to supply one global instance.
    pub trait DriverManager {
        /// Return a slice of references to all `BSP`-instantiated drivers.
        fn all_device_drivers(&self) -> &[&'static (dyn DeviceDriver + Sync)];

        /// Return only those drivers needed for the BSP's early printing functionality.
        ///
        /// For example, the default UART.
        fn early_print_device_drivers(&self) -> &[&'static (dyn DeviceDriver + Sync)];

        /// Return all drivers minus early-print drivers.
        fn non_early_print_device_drivers(&self) -> &[&'static (dyn DeviceDriver + Sync)];

        /// Initialization code that runs after the early print driver init.
        fn post_early_print_device_driver_init(&self);
    }
}
