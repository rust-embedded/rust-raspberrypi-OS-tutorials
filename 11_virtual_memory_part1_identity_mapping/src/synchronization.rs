// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020 Andre Richter <andre.o.richter@gmail.com>

//! Synchronization primitives.

use core::cell::UnsafeCell;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Synchronization interfaces.
pub mod interface {

    /// Any object implementing this trait guarantees exclusive access to the data contained within
    /// the Mutex for the duration of the provided closure.
    ///
    /// The trait follows the [Rust embedded WG's proposal] and therefore provides some goodness
    /// such as [deadlock prevention].
    ///
    /// # Example
    ///
    /// Since the lock function takes an `&mut self` to enable deadlock-prevention, the trait is
    /// best implemented **for a reference to a container struct**, and has a usage pattern that
    /// might feel strange at first:
    ///
    /// [Rust embedded WG's proposal]: https://github.com/rust-embedded/wg/blob/master/rfcs/0377-mutex-trait.md
    /// [deadlock prevention]: https://github.com/rust-embedded/wg/blob/master/rfcs/0377-mutex-trait.md#design-decisions-and-compatibility
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
        /// The type of encapsulated data.
        type Data;

        /// Creates a critical section and grants temporary mutable access to the encapsulated data.
        fn lock<R>(&mut self, f: impl FnOnce(&mut Self::Data) -> R) -> R;
    }
}

/// A pseudo-lock for teaching purposes.
///
/// Used to introduce [interior mutability].
///
/// In contrast to a real Mutex implementation, does not protect against concurrent access from
/// other cores to the contained data. This part is preserved for later lessons.
///
/// The lock will only be used as long as it is safe to do so, i.e. as long as the kernel is
/// executing single-threaded, aka only running on a single core with interrupts disabled.
///
/// [interior mutability]: https://doc.rust-lang.org/std/cell/index.html
pub struct NullLock<T: ?Sized> {
    data: UnsafeCell<T>,
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

unsafe impl<T: ?Sized> Sync for NullLock<T> {}

impl<T> NullLock<T> {
    /// Wraps `data` into a new `NullLock`.
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }
}

//------------------------------------------------------------------------------
// OS Interface Code
//------------------------------------------------------------------------------

impl<T> interface::Mutex for &NullLock<T> {
    type Data = T;

    fn lock<R>(&mut self, f: impl FnOnce(&mut Self::Data) -> R) -> R {
        // In a real lock, there would be code encapsulating this line that ensures that this
        // mutable reference will ever only be given out once at a time.
        let data = unsafe { &mut *self.data.get() };

        f(data)
    }
}
