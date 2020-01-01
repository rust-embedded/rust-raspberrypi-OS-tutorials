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
    /// Console write functions.
    ///
    /// `core::fmt::Write` is exactly what we need for now. Re-export it here because
    /// implementing `console::Write` gives a better hint to the reader about the
    /// intention.
    pub use core::fmt::Write;

    /// Console read functions.
    pub trait Read {
        /// Read a single character.
        fn read_char(&self) -> char {
            ' '
        }
    }
}
