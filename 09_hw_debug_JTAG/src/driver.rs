// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! Driver support.

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Driver interfaces.
pub mod interface {

    /// Device Driver functions.
    pub trait DeviceDriver {
        /// Return a compatibility string for identifying the driver.
        fn compatible(&self) -> &str;

        /// Called by the kernel to bring up the device.
        fn init(&self) -> Result<(), ()> {
            Ok(())
        }
    }

    /// Device driver management functions.
    ///
    /// The `BSP` is supposed to supply one global instance.
    pub trait DriverManager {
        /// Return a slice of references to all `BSP`-instantiated drivers.
        ///
        /// # Safety
        ///
        /// - The order of devices is the order in which `DeviceDriver::init()` is called.
        fn all_device_drivers(&self) -> &[&'static (dyn DeviceDriver + Sync)];

        /// Initialization code that runs after driver init.
        ///
        /// For example, device driver code that depends on other drivers already being online.
        fn post_device_driver_init(&self);
    }
}
