// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020-2021 Andre Richter <andre.o.richter@gmail.com>

//! Timer primitives.

#[cfg(target_arch = "aarch64")]
#[path = "_arch/aarch64/time.rs"]
mod arch_time;

//--------------------------------------------------------------------------------------------------
// Architectural Public Reexports
//--------------------------------------------------------------------------------------------------
pub use arch_time::time_manager;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Timekeeping interfaces.
pub mod interface {
    use core::time::Duration;

    /// Time management functions.
    pub trait TimeManager {
        /// The timer's resolution.
        fn resolution(&self) -> Duration;

        /// The uptime since power-on of the device.
        ///
        /// This includes time consumed by firmware and bootloaders.
        fn uptime(&self) -> Duration;

        /// Spin for a given duration.
        fn spin_for(&self, duration: Duration);
    }
}
