# Tutorial 03 - Hacky Hello World

## tl;dr

Introducing global `print!()` macros to enable "printf debugging" at the
earliest; To keep tutorial length reasonable, printing functions for now "abuse" a
QEMU property that lets us use the RPi's `UART` without setting it up properly;
Using  the real hardware `UART` is enabled step-by-step in following tutorials.

- `interface.rs` is introduced:
	- Provides `Traits` for abstracting `kernel` from `BSP` and `arch` code.
- Panic handler `print!()`s supplied error messages.
    - This is showcased in `main()`.

### Test it

QEMU is no longer running in assembly mode. It will from now on show the output
of the `console`.

```console
» make qemu
[...]
Hello from Rust!
Kernel panic: Stopping here.
```

## Diff to previous
```diff

diff -uNr 02_runtime_init/src/bsp/rpi.rs 03_hacky_hello_world/src/bsp/rpi.rs
--- 02_runtime_init/src/bsp/rpi.rs
+++ 03_hacky_hello_world/src/bsp/rpi.rs
@@ -4,4 +4,35 @@

 //! Board Support Package for the Raspberry Pi.

-// Coming soon.
+use crate::interface;
+use core::fmt;
+
+/// A mystical, magical device for generating QEMU output out of the void.
+struct QEMUOutput;
+
+/// Implementing `console::Write` enables usage of the `format_args!` macros, which in turn are used
+/// to implement the `kernel`'s `print!` and `println!` macros.
+///
+/// See [`src/print.rs`].
+///
+/// [`src/print.rs`]: ../../print/index.html
+impl interface::console::Write for QEMUOutput {
+    fn write_str(&mut self, s: &str) -> fmt::Result {
+        for c in s.chars() {
+            unsafe {
+                core::ptr::write_volatile(0x3F20_1000 as *mut u8, c as u8);
+            }
+        }
+
+        Ok(())
+    }
+}
+
+//--------------------------------------------------------------------------------------------------
+// Implementation of the kernel's BSP calls
+//--------------------------------------------------------------------------------------------------
+
+/// Returns a ready-to-use `console::Write` implementation.
+pub fn console() -> impl interface::console::Write {
+    QEMUOutput {}
+}

diff -uNr 02_runtime_init/src/interface.rs 03_hacky_hello_world/src/interface.rs
--- 02_runtime_init/src/interface.rs
+++ 03_hacky_hello_world/src/interface.rs
@@ -0,0 +1,37 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
+
+//! Trait definitions for coupling `kernel` and `BSP` code.
+//!
+//! ```
+//!         +-------------------+
+//!         | Interface (Trait) |
+//!         |                   |
+//!         +--+-------------+--+
+//!            ^             ^
+//!            |             |
+//!            |             |
+//! +----------+--+       +--+----------+
+//! | Kernel code |       |  BSP Code   |
+//! |             |       |             |
+//! +-------------+       +-------------+
+//! ```
+
+/// System console operations.
+pub mod console {
+    /// Console write functions.
+    ///
+    /// `core::fmt::Write` is exactly what we need for now. Re-export it here because
+    /// implementing `console::Write` gives a better hint to the reader about the
+    /// intention.
+    pub use core::fmt::Write;
+
+    /// Console read functions.
+    pub trait Read {
+        /// Read a single character.
+        fn read_char(&self) -> char {
+            ' '
+        }
+    }
+}

diff -uNr 02_runtime_init/src/main.rs 03_hacky_hello_world/src/main.rs
--- 02_runtime_init/src/main.rs
+++ 03_hacky_hello_world/src/main.rs
@@ -6,9 +6,23 @@
 #![doc(html_logo_url = "https://git.io/JeGIp")]

 //! The `kernel`
+//!
+//! The `kernel` is composed by glueing together code from
+//!
+//!   - [Hardware-specific Board Support Packages] (`BSPs`).
+//!   - [Architecture-specific code].
+//!   - HW- and architecture-agnostic `kernel` code.
+//!
+//! using the [`kernel::interface`] traits.
+//!
+//! [Hardware-specific Board Support Packages]: bsp/index.html
+//! [Architecture-specific code]: arch/index.html
+//! [`kernel::interface`]: interface/index.html

 #![feature(asm)]
+#![feature(format_args_nl)]
 #![feature(global_asm)]
+#![feature(panic_info_message)]
 #![no_main]
 #![no_std]

@@ -22,8 +36,10 @@
 // Conditionally includes the selected `BSP` code.
 mod bsp;

+mod interface;
 mod memory;
 mod panic_wait;
+mod print;

 /// Early init code.
 ///
@@ -31,5 +47,7 @@
 ///
 /// - Only a single core must be active and running this function.
 unsafe fn kernel_init() -> ! {
-    panic!()
+    println!("Hello from Rust!");
+
+    panic!("Stopping here.")
 }

diff -uNr 02_runtime_init/src/panic_wait.rs 03_hacky_hello_world/src/panic_wait.rs
--- 02_runtime_init/src/panic_wait.rs
+++ 03_hacky_hello_world/src/panic_wait.rs
@@ -4,9 +4,16 @@

 //! A panic handler that infinitely waits.

+use crate::{arch, println};
 use core::panic::PanicInfo;

 #[panic_handler]
-fn panic(_info: &PanicInfo) -> ! {
-    crate::arch::wait_forever()
+fn panic(info: &PanicInfo) -> ! {
+    if let Some(args) = info.message() {
+        println!("\nKernel panic: {}", args);
+    } else {
+        println!("\nKernel panic!");
+    }
+
+    arch::wait_forever()
 }

diff -uNr 02_runtime_init/src/print.rs 03_hacky_hello_world/src/print.rs
--- 02_runtime_init/src/print.rs
+++ 03_hacky_hello_world/src/print.rs
@@ -0,0 +1,34 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
+
+//! Printing facilities.
+
+use crate::{bsp, interface};
+use core::fmt;
+
+#[doc(hidden)]
+pub fn _print(args: fmt::Arguments) {
+    use interface::console::Write;
+
+    bsp::console().write_fmt(args).unwrap();
+}
+
+/// Prints without a newline.
+///
+/// Carbon copy from https://doc.rust-lang.org/src/std/macros.rs.html
+#[macro_export]
+macro_rules! print {
+    ($($arg:tt)*) => ($crate::print::_print(format_args!($($arg)*)));
+}
+
+/// Prints with a newline.
+///
+/// Carbon copy from https://doc.rust-lang.org/src/std/macros.rs.html
+#[macro_export]
+macro_rules! println {
+    () => ($crate::print!("\n"));
+    ($($arg:tt)*) => ({
+        $crate::print::_print(format_args_nl!($($arg)*));
+    })
+}

```
