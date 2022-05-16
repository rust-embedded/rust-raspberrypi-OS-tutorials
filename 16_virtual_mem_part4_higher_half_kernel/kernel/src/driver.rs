// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2022 Andre Richter <andre.o.richter@gmail.com>

//! Driver support.

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Driver interfaces.
pub mod interface {
    use crate::bsp;

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
    }

    /// Device driver management functions.
    ///
    /// The `BSP` is supposed to supply one global instance.
    pub trait DriverManager {
        /// Instantiate all drivers.
        ///
        /// # Safety
        ///
        /// Must be called before `all_device_drivers`.
        unsafe fn instantiate_drivers(&self) -> Result<(), &'static str>;

        /// Return a slice of references to all `BSP`-instantiated drivers.
        fn all_device_drivers(&self) -> [&(dyn DeviceDriver + Sync); bsp::driver::NUM_DRIVERS];

        /// Minimal code needed to bring up the console in QEMU (for testing only). This is often
        /// less steps than on real hardware due to QEMU's abstractions.
        #[cfg(feature = "test_build")]
        fn qemu_bring_up_console(&self);
    }
}
