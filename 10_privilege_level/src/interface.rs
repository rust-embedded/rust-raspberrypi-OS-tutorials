// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! Trait definitions for coupling `kernel` and `BSP` code.
//!
//! ```
//!         +-------------------+
//!         | Interface (Trait) |
//!         |                   |
//!         +--+-------------+--+
//!            ^             ^
//!            |             |
//!            |             |
//! +----------+--+       +--+----------+
//! | Kernel code |       |  BSP Code   |
//! |             |       |             |
//! +-------------+       +-------------+
//! ```

/// System console operations.
pub mod console {
    use core::fmt;

    /// Console write functions.
    pub trait Write {
        /// Write a single character.
        fn write_char(&self, c: char);

        /// Write a Rust format string.
        fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result;

        /// Block execution until the last character has been physically put on the TX wire
        /// (draining TX buffers/FIFOs, if any).
        fn flush(&self);
    }

    /// Console read functions.
    pub trait Read {
        /// Read a single character.
        fn read_char(&self) -> char {
            ' '
        }

        /// Clear RX buffers, if any.
        fn clear(&self);
    }

    /// Console statistics.
    pub trait Statistics {
        /// Return the number of characters written.
        fn chars_written(&self) -> usize {
            0
        }

        /// Return the number of characters read.
        fn chars_read(&self) -> usize {
            0
        }
    }

    /// Trait alias for a full-fledged console.
    pub trait All = Write + Read + Statistics;
}

/// Synchronization primitives.
pub mod sync {
    /// Any object implementing this trait guarantees exclusive access to the data contained within
    /// the mutex for the duration of the lock.
    ///
    /// The trait follows the [Rust embedded WG's
    /// proposal](https://github.com/korken89/wg/blob/master/rfcs/0377-mutex-trait.md) and therefore
    /// provides some goodness such as [deadlock
    /// prevention](https://github.com/korken89/wg/blob/master/rfcs/0377-mutex-trait.md#design-decisions-and-compatibility).
    ///
    /// # Example
    ///
    /// Since the lock function takes an `&mut self` to enable deadlock-prevention, the trait is
    /// best implemented **for a reference to a container struct**, and has a usage pattern that
    /// might feel strange at first:
    ///
    /// ```
    /// static MUT: Mutex<RefCell<i32>> = Mutex::new(RefCell::new(0));
    ///
    /// fn foo() {
    ///     let mut r = &MUT; // Note that r is mutable
    ///     r.lock(|data| *data += 1);
    /// }
    /// ```
    pub trait Mutex {
        /// Type of data encapsulated by the mutex.
        type Data;

        /// Creates a critical section and grants temporary mutable access to the encapsulated data.
        fn lock<R>(&mut self, f: impl FnOnce(&mut Self::Data) -> R) -> R;
    }
}

/// Driver interfaces.
pub mod driver {
    /// Driver result type, e.g. for indicating successful driver init.
    pub type Result = core::result::Result<(), ()>;

    /// Device Driver functions.
    pub trait DeviceDriver {
        /// Return a compatibility string for identifying the driver.
        fn compatible(&self) -> &str;

        /// Called by the kernel to bring up the device.
        fn init(&self) -> Result {
            Ok(())
        }
    }
}

/// Timekeeping interfaces.
pub mod time {
    use core::time::Duration;

    /// Timer functions.
    pub trait Timer {
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
