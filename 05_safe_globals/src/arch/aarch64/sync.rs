// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! Synchronization primitives.

use crate::interface;
use core::cell::UnsafeCell;

//--------------------------------------------------------------------------------------------------
// Arch-public
//--------------------------------------------------------------------------------------------------

/// A pseudo-lock for teaching purposes.
///
/// Used to introduce [interior mutability].
///
/// In contrast to a real Mutex implementation, does not protect against concurrent access to the
/// contained data. This part is preserved for later lessons.
///
/// The lock will only be used as long as it is safe to do so, i.e. as long as the kernel is
/// executing single-threaded, aka only running on a single core with interrupts disabled.
///
/// [interior mutability]: https://doc.rust-lang.org/std/cell/index.html
pub struct NullLock<T: ?Sized> {
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send> Send for NullLock<T> {}
unsafe impl<T: ?Sized + Send> Sync for NullLock<T> {}

impl<T> NullLock<T> {
    /// Wraps `data` into a new `NullLock`.
    pub const fn new(data: T) -> NullLock<T> {
        NullLock {
            data: UnsafeCell::new(data),
        }
    }
}

//--------------------------------------------------------------------------------------------------
// OS interface implementations
//--------------------------------------------------------------------------------------------------

impl<T> interface::sync::Mutex for &NullLock<T> {
    type Data = T;

    fn lock<R>(&mut self, f: impl FnOnce(&mut Self::Data) -> R) -> R {
        // In a real lock, there would be code encapsulating this line that ensures that this
        // mutable reference will ever only be given out once at a time.
        f(unsafe { &mut *self.data.get() })
    }
}
