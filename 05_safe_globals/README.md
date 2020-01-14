# Tutorial 05 - Safe Globals

## A slightly longer tl;dr

When we introduced the globally usable `print!` macros in [tutorial 03], we
cheated a bit. Calling `core::fmt`'s `write_fmt()` function, which takes an
`&mut self`, was only working because on each call, a new instance of
`QEMUOutput` was created.

If we would want to preserve some state, e.g. statistics about the number of
characters written, we need to make a single global instance of `QEMUOutput` (in
Rust, using the `static` keyword).

A `static QEMU_OUTPUT`, however, would not allow to call functions taking `&mut
self`. For that, we would need a `static mut`, but calling functions that mutate
state on `static mut`s is unsafe. The Rust compiler's reasoning for this is that
it can then not prevent anymore that multiple cores/threads are mutating the
data concurrently (it is a global, so everyone can reference it from anywhere.
The borrow checker can't help here).

The solution to this problem is to wrap the global into a synchronization
primitive. In our case, a variant of a *MUTual EXclusion* primivite. `Mutex` is
introduced as a trait in `interfaces.rs`, and implemented by the name of
`NullLock` in `sync.rs` in the `arch` folder. For teaching purposes, to make the
code lean, it leaves out the actual architecture-specific logic for protection
against concurrent access, since we don't need it as long as the kernel only
executes on a single core with interrupts disabled.

Instead, it focuses on showcasing the Rust core concept of [interior mutability].
Make sure to read up on it. I also recommend to read this article about an
[accurate mental model for Rust's reference types].

If you want to compare the `NullLock` to some real-world mutex implementations,
you can check out implemntations in the [spin crate] or the [parking lot crate].

[tutorial 03]: ../03_hacky_hello_world
[interior mutability]: https://doc.rust-lang.org/std/cell/index.html
[accurate mental model for Rust's reference types]: https://docs.rs/dtolnay/0.0.6/dtolnay/macro._02__reference_types.html
[spin crate]: https://github.com/mvdnes/spin-rs
[parking lot crate]: https://github.com/Amanieu/parking_lot

### Test it

```console
» make qemu
[...]
[0] Hello from pure Rust!
[1] Chars written: 26
[2] Stopping here.
```

## Diff to previous
```diff

diff -uNr 04_zero_overhead_abstraction/src/arch/aarch64/sync.rs 05_safe_globals/src/arch/aarch64/sync.rs
--- 04_zero_overhead_abstraction/src/arch/aarch64/sync.rs
+++ 05_safe_globals/src/arch/aarch64/sync.rs
@@ -0,0 +1,53 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
+
+//! Synchronization primitives.
+
+use crate::interface;
+use core::cell::UnsafeCell;
+
+//--------------------------------------------------------------------------------------------------
+// Arch-public
+//--------------------------------------------------------------------------------------------------
+
+/// A pseudo-lock for teaching purposes.
+///
+/// Used to introduce [interior mutability].
+///
+/// In contrast to a real Mutex implementation, does not protect against concurrent access to the
+/// contained data. This part is preserved for later lessons.
+///
+/// The lock will only be used as long as it is safe to do so, i.e. as long as the kernel is
+/// executing single-threaded, aka only running on a single core with interrupts disabled.
+///
+/// [interior mutability]: https://doc.rust-lang.org/std/cell/index.html
+pub struct NullLock<T: ?Sized> {
+    data: UnsafeCell<T>,
+}
+
+unsafe impl<T: ?Sized + Send> Send for NullLock<T> {}
+unsafe impl<T: ?Sized + Send> Sync for NullLock<T> {}
+
+impl<T> NullLock<T> {
+    /// Wraps `data` into a new `NullLock`.
+    pub const fn new(data: T) -> NullLock<T> {
+        NullLock {
+            data: UnsafeCell::new(data),
+        }
+    }
+}
+
+//--------------------------------------------------------------------------------------------------
+// OS interface implementations
+//--------------------------------------------------------------------------------------------------
+
+impl<T> interface::sync::Mutex for &NullLock<T> {
+    type Data = T;
+
+    fn lock<R>(&mut self, f: impl FnOnce(&mut Self::Data) -> R) -> R {
+        // In a real lock, there would be code encapsulating this line that ensures that this
+        // mutable reference will ever only be given out once at a time.
+        f(unsafe { &mut *self.data.get() })
+    }
+}

diff -uNr 04_zero_overhead_abstraction/src/arch/aarch64.rs 05_safe_globals/src/arch/aarch64.rs
--- 04_zero_overhead_abstraction/src/arch/aarch64.rs
+++ 05_safe_globals/src/arch/aarch64.rs
@@ -4,6 +4,8 @@

 //! AArch64.

+pub mod sync;
+
 use crate::bsp;
 use cortex_a::{asm, regs::*};


diff -uNr 04_zero_overhead_abstraction/src/bsp/rpi.rs 05_safe_globals/src/bsp/rpi.rs
--- 04_zero_overhead_abstraction/src/bsp/rpi.rs
+++ 05_safe_globals/src/bsp/rpi.rs
@@ -4,7 +4,7 @@

 //! Board Support Package for the Raspberry Pi.

-use crate::interface;
+use crate::{arch::sync::NullLock, interface};
 use core::fmt;

 /// Used by `arch` code to find the early boot core.
@@ -14,31 +14,107 @@
 pub const BOOT_CORE_STACK_START: u64 = 0x80_000;

 /// A mystical, magical device for generating QEMU output out of the void.
-struct QEMUOutput;
+///
+/// The mutex protected part.
+struct QEMUOutputInner {
+    chars_written: usize,
+}
+
+impl QEMUOutputInner {
+    const fn new() -> QEMUOutputInner {
+        QEMUOutputInner { chars_written: 0 }
+    }
+
+    /// Send a character.
+    fn write_char(&mut self, c: char) {
+        unsafe {
+            core::ptr::write_volatile(0x3F20_1000 as *mut u8, c as u8);
+        }
+    }
+}

-/// Implementing `console::Write` enables usage of the `format_args!` macros, which in turn are used
-/// to implement the `kernel`'s `print!` and `println!` macros.
+/// Implementing `core::fmt::Write` enables usage of the `format_args!` macros, which in turn are
+/// used to implement the `kernel`'s `print!` and `println!` macros. By implementing `write_str()`,
+/// we get `write_fmt()` automatically.
+///
+/// The function takes an `&mut self`, so it must be implemented for the inner struct.
 ///
 /// See [`src/print.rs`].
 ///
 /// [`src/print.rs`]: ../../print/index.html
-impl interface::console::Write for QEMUOutput {
+impl fmt::Write for QEMUOutputInner {
     fn write_str(&mut self, s: &str) -> fmt::Result {
         for c in s.chars() {
-            unsafe {
-                core::ptr::write_volatile(0x3F20_1000 as *mut u8, c as u8);
+            // Convert newline to carrige return + newline.
+            if c == '\n' {
+                self.write_char('\r')
             }
+
+            self.write_char(c);
         }

+        self.chars_written += s.len();
+
         Ok(())
     }
 }

 //--------------------------------------------------------------------------------------------------
+// BSP-public
+//--------------------------------------------------------------------------------------------------
+
+/// The main struct.
+pub struct QEMUOutput {
+    inner: NullLock<QEMUOutputInner>,
+}
+
+impl QEMUOutput {
+    pub const fn new() -> QEMUOutput {
+        QEMUOutput {
+            inner: NullLock::new(QEMUOutputInner::new()),
+        }
+    }
+}
+
+//--------------------------------------------------------------------------------------------------
+// OS interface implementations
+//--------------------------------------------------------------------------------------------------
+
+/// Passthrough of `args` to the `core::fmt::Write` implementation, but guarded by a Mutex to
+/// serialize access.
+impl interface::console::Write for QEMUOutput {
+    fn write_fmt(&self, args: core::fmt::Arguments) -> fmt::Result {
+        use interface::sync::Mutex;
+
+        // Fully qualified syntax for the call to `core::fmt::Write::write:fmt()` to increase
+        // readability.
+        let mut r = &self.inner;
+        r.lock(|inner| fmt::Write::write_fmt(inner, args))
+    }
+}
+
+impl interface::console::Read for QEMUOutput {}
+
+impl interface::console::Statistics for QEMUOutput {
+    fn chars_written(&self) -> usize {
+        use interface::sync::Mutex;
+
+        let mut r = &self.inner;
+        r.lock(|inner| inner.chars_written)
+    }
+}
+
+//--------------------------------------------------------------------------------------------------
+// Global instances
+//--------------------------------------------------------------------------------------------------
+
+static QEMU_OUTPUT: QEMUOutput = QEMUOutput::new();
+
+//--------------------------------------------------------------------------------------------------
 // Implementation of the kernel's BSP calls
 //--------------------------------------------------------------------------------------------------

-/// Returns a ready-to-use `console::Write` implementation.
-pub fn console() -> impl interface::console::Write {
-    QEMUOutput {}
+/// Return a reference to a `console::All` implementation.
+pub fn console() -> &'static impl interface::console::All {
+    &QEMU_OUTPUT
 }

diff -uNr 04_zero_overhead_abstraction/src/interface.rs 05_safe_globals/src/interface.rs
--- 04_zero_overhead_abstraction/src/interface.rs
+++ 05_safe_globals/src/interface.rs
@@ -20,12 +20,13 @@

 /// System console operations.
 pub mod console {
+    use core::fmt;
+
     /// Console write functions.
-    ///
-    /// `core::fmt::Write` is exactly what we need for now. Re-export it here because
-    /// implementing `console::Write` gives a better hint to the reader about the
-    /// intention.
-    pub use core::fmt::Write;
+    pub trait Write {
+        /// Write a Rust format string.
+        fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result;
+    }

     /// Console read functions.
     pub trait Read {
@@ -34,4 +35,53 @@
             ' '
         }
     }
+
+    /// Console statistics.
+    pub trait Statistics {
+        /// Return the number of characters written.
+        fn chars_written(&self) -> usize {
+            0
+        }
+
+        /// Return the number of characters read.
+        fn chars_read(&self) -> usize {
+            0
+        }
+    }
+
+    /// Trait alias for a full-fledged console.
+    pub trait All = Write + Read + Statistics;
+}
+
+/// Synchronization primitives.
+pub mod sync {
+    /// Any object implementing this trait guarantees exclusive access to the data contained within
+    /// the mutex for the duration of the lock.
+    ///
+    /// The trait follows the [Rust embedded WG's
+    /// proposal](https://github.com/korken89/wg/blob/master/rfcs/0377-mutex-trait.md) and therefore
+    /// provides some goodness such as [deadlock
+    /// prevention](https://github.com/korken89/wg/blob/master/rfcs/0377-mutex-trait.md#design-decisions-and-compatibility).
+    ///
+    /// # Example
+    ///
+    /// Since the lock function takes an `&mut self` to enable deadlock-prevention, the trait is
+    /// best implemented **for a reference to a container struct**, and has a usage pattern that
+    /// might feel strange at first:
+    ///
+    /// ```
+    /// static MUT: Mutex<RefCell<i32>> = Mutex::new(RefCell::new(0));
+    ///
+    /// fn foo() {
+    ///     let mut r = &MUT; // Note that r is mutable
+    ///     r.lock(|data| *data += 1);
+    /// }
+    /// ```
+    pub trait Mutex {
+        /// Type of data encapsulated by the mutex.
+        type Data;
+
+        /// Creates a critical section and grants temporary mutable access to the encapsulated data.
+        fn lock<R>(&mut self, f: impl FnOnce(&mut Self::Data) -> R) -> R;
+    }
 }

diff -uNr 04_zero_overhead_abstraction/src/main.rs 05_safe_globals/src/main.rs
--- 04_zero_overhead_abstraction/src/main.rs
+++ 05_safe_globals/src/main.rs
@@ -21,6 +21,7 @@

 #![feature(format_args_nl)]
 #![feature(panic_info_message)]
+#![feature(trait_alias)]
 #![no_main]
 #![no_std]

@@ -45,8 +46,12 @@
 ///
 /// - Only a single core must be active and running this function.
 unsafe fn kernel_init() -> ! {
+    use interface::console::Statistics;
+
     println!("[0] Hello from pure Rust!");

-    println!("[1] Stopping here.");
+    println!("[1] Chars written: {}", bsp::console().chars_written());
+
+    println!("[2] Stopping here.");
     arch::wait_forever()
 }

```
