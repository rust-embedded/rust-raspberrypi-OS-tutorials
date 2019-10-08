# Tutorial 03 - Hacky Hello World

## tl;dr

Introducing global `print!()` macros to enable "printf debugging" at the
earliest; To keep tutorial length reasonable, printing function for now abuses a
QEMU property and doesn't really use the RPi3's `UART`; Using real `UART` is
enabled step-by-step in following tutorials.

- `interface.rs` is introduced:
	- Provides `Traits` for abstracting `kernel` from `BSP` code.
- Panic handler `print!()`s supplied error messages.
    - This is showcased in `main()`.

## Diff to previous
```diff

diff -uNr 02_runtime_init/Makefile 03_hacky_hello_world/Makefile
--- 02_runtime_init/Makefile
+++ 03_hacky_hello_world/Makefile
@@ -13,7 +13,7 @@
 	OUTPUT = kernel8.img
 	QEMU_BINARY = qemu-system-aarch64
 	QEMU_MACHINE_TYPE = raspi3
-	QEMU_MISC_ARGS = -d in_asm
+	QEMU_MISC_ARGS = -serial null -serial stdio
 	LINKER_FILE = src/bsp/rpi3/link.ld
 	RUSTC_MISC_ARGS = -C target-cpu=cortex-a53
 endif

diff -uNr 02_runtime_init/src/bsp/rpi3/panic_wait.rs 03_hacky_hello_world/src/bsp/rpi3/panic_wait.rs
--- 02_runtime_init/src/bsp/rpi3/panic_wait.rs
+++ 03_hacky_hello_world/src/bsp/rpi3/panic_wait.rs
@@ -4,10 +4,17 @@

 //! A panic handler that infinitely waits.

+use crate::println;
 use core::panic::PanicInfo;

 #[panic_handler]
-fn panic(_info: &PanicInfo) -> ! {
+fn panic(info: &PanicInfo) -> ! {
+    if let Some(args) = info.message() {
+        println!("Kernel panic: {}", args);
+    } else {
+        println!("Kernel panic!");
+    }
+
     unsafe {
         loop {
             asm!("wfe" :::: "volatile")

diff -uNr 02_runtime_init/src/bsp/rpi3.rs 03_hacky_hello_world/src/bsp/rpi3.rs
--- 02_runtime_init/src/bsp/rpi3.rs
+++ 03_hacky_hello_world/src/bsp/rpi3.rs
@@ -6,4 +6,38 @@

 mod panic_wait;

+use crate::interface;
+use core::fmt;
+
 global_asm!(include_str!("rpi3/start.S"));
+
+/// A mystical, magical device for generating QEMU output out of the void.
+struct QEMUOutput;
+
+/// Implementing `console::Write` enables usage of the `format_args!` macros,
+/// which in turn are used to implement the `kernel`'s `print!` and `println!`
+/// macros.
+///
+/// See [`src/print.rs`].
+///
+/// [`src/print.rs`]: ../../print/index.html
+impl interface::console::Write for QEMUOutput {
+    fn write_str(&mut self, s: &str) -> fmt::Result {
+        for c in s.chars() {
+            unsafe {
+                core::ptr::write_volatile(0x3F21_5040 as *mut u8, c as u8);
+            }
+        }
+
+        Ok(())
+    }
+}
+
+////////////////////////////////////////////////////////////////////////////////
+// Implementation of the kernel's BSP calls
+////////////////////////////////////////////////////////////////////////////////
+
+/// Returns a ready-to-use `console::Write` implementation.
+pub fn console() -> impl interface::console::Write {
+    QEMUOutput {}
+}

diff -uNr 02_runtime_init/src/interface.rs 03_hacky_hello_world/src/interface.rs
--- 02_runtime_init/src/interface.rs
+++ 03_hacky_hello_world/src/interface.rs
@@ -0,0 +1,36 @@
+// SPDX-License-Identifier: MIT
+//
+// Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>
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
+    /// `core::fmt::Write` is exactly what we need for now. Re-export it here
+    /// because implementing `console::Write` gives a better hint to the reader
+    /// about the intention.
+    pub use core::fmt::Write;
+
+    /// Console read functions.
+    pub trait Read {
+        fn read_char(&mut self) -> char {
+            ' '
+        }
+    }
+}

diff -uNr 02_runtime_init/src/main.rs 03_hacky_hello_world/src/main.rs
--- 02_runtime_init/src/main.rs
+++ 03_hacky_hello_world/src/main.rs
@@ -6,9 +6,17 @@
 #![doc(html_logo_url = "https://git.io/JeGIp")]

 //! The `kernel`
+//!
+//! The `kernel` is composed by glueing together hardware-specific Board Support
+//! Package (`BSP`) code and hardware-agnostic `kernel` code through the
+//! [`kernel::interface`] traits.
+//!
+//! [`kernel::interface`]: interface/index.html

 #![feature(asm)]
+#![feature(format_args_nl)]
 #![feature(global_asm)]
+#![feature(panic_info_message)]
 #![no_main]
 #![no_std]

@@ -20,7 +28,12 @@
 // module, which on completion, jumps to `kernel_entry()`.
 mod runtime_init;

+mod interface;
+mod print;
+
 /// Entrypoint of the `kernel`.
 fn kernel_entry() -> ! {
-    panic!()
+    println!("Hello from Rust!");
+
+    panic!("Stopping here.")
 }

diff -uNr 02_runtime_init/src/print.rs 03_hacky_hello_world/src/print.rs
--- 02_runtime_init/src/print.rs
+++ 03_hacky_hello_world/src/print.rs
@@ -0,0 +1,34 @@
+// SPDX-License-Identifier: MIT
+//
+// Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>
+
+//! Printing facilities.
+
+use crate::bsp;
+use crate::interface;
+use core::fmt;
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
+    () => ($crate::print!("
"));
+    ($($arg:tt)*) => ({
+        $crate::print::_print(format_args_nl!($($arg)*));
+    })
+}
+
+pub fn _print(args: fmt::Arguments) {
+    use interface::console::Write;
+
+    bsp::console().write_fmt(args).unwrap();
+}
```
