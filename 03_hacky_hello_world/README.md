# Tutorial 03 - Hacky Hello World

## tl;dr

- Introducing global `println!()` macros to enable "printf debugging" at the earliest.
- To keep tutorial length reasonable, printing functions for now "abuse" a QEMU property that lets
  us use the Raspberry's `UART` without setting it up properly.
- Using the real hardware `UART` is enabled step-by-step in following tutorials.

## Notable additions

- `src/console.rs` introduces interface `Traits` for console commands and global access to the
  kernel's console through `console::console()`.
- `src/bsp/raspberrypi/console.rs` implements the interface for QEMU's emulated UART.
- The panic handler makes use of the new `println!()` to display user error messages.
- There is a new Makefile target, `make test`, intended for automated testing. It boots the compiled
  kernel in `QEMU`, and checks for an expected output string produced by the kernel.
  - In this tutorial, it checks for the string `Stopping here`, which is emitted by the `panic!()`
    at the end of `main.rs`.

## Test it

QEMU is no longer running in assembly mode. It will from now on show the output of the `console`.

```console
$ make qemu
[...]

Hello from Rust!
Kernel panic!

Panic location:
      File 'src/main.rs', line 126, column 5

Stopping here.
```

## Diff to previous
```diff

diff -uNr 02_runtime_init/Cargo.toml 03_hacky_hello_world/Cargo.toml
--- 02_runtime_init/Cargo.toml
+++ 03_hacky_hello_world/Cargo.toml
@@ -1,6 +1,6 @@
 [package]
 name = "mingo"
-version = "0.2.0"
+version = "0.3.0"
 authors = ["Andre Richter <andre.o.richter@gmail.com>"]
 edition = "2021"


diff -uNr 02_runtime_init/Makefile 03_hacky_hello_world/Makefile
--- 02_runtime_init/Makefile
+++ 03_hacky_hello_world/Makefile
@@ -25,7 +25,7 @@
     KERNEL_BIN        = kernel8.img
     QEMU_BINARY       = qemu-system-aarch64
     QEMU_MACHINE_TYPE = raspi3
-    QEMU_RELEASE_ARGS = -d in_asm -display none
+    QEMU_RELEASE_ARGS = -serial stdio -display none
     OBJDUMP_BINARY    = aarch64-none-elf-objdump
     NM_BINARY         = aarch64-none-elf-nm
     READELF_BINARY    = aarch64-none-elf-readelf
@@ -36,7 +36,7 @@
     KERNEL_BIN        = kernel8.img
     QEMU_BINARY       = qemu-system-aarch64
     QEMU_MACHINE_TYPE =
-    QEMU_RELEASE_ARGS = -d in_asm -display none
+    QEMU_RELEASE_ARGS = -serial stdio -display none
     OBJDUMP_BINARY    = aarch64-none-elf-objdump
     NM_BINARY         = aarch64-none-elf-nm
     READELF_BINARY    = aarch64-none-elf-readelf
@@ -86,17 +86,20 @@
     --strip-all            \
     -O binary

-EXEC_QEMU = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)
+EXEC_QEMU          = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)
+EXEC_TEST_DISPATCH = ruby ../common/tests/dispatch.rb

 ##------------------------------------------------------------------------------
 ## Dockerization
 ##------------------------------------------------------------------------------
-DOCKER_CMD          = docker run -t --rm -v $(shell pwd):/work/tutorial -w /work/tutorial
-DOCKER_CMD_INTERACT = $(DOCKER_CMD) -i
+DOCKER_CMD            = docker run -t --rm -v $(shell pwd):/work/tutorial -w /work/tutorial
+DOCKER_CMD_INTERACT   = $(DOCKER_CMD) -i
+DOCKER_ARG_DIR_COMMON = -v $(shell pwd)/../common:/work/common

 # DOCKER_IMAGE defined in include file (see top of this file).
 DOCKER_QEMU  = $(DOCKER_CMD_INTERACT) $(DOCKER_IMAGE)
 DOCKER_TOOLS = $(DOCKER_CMD) $(DOCKER_IMAGE)
+DOCKER_TEST  = $(DOCKER_CMD) $(DOCKER_ARG_DIR_COMMON) $(DOCKER_IMAGE)



@@ -191,3 +194,27 @@
 	$(call color_header, "Launching nm")
 	@$(DOCKER_TOOLS) $(NM_BINARY) --demangle --print-size $(KERNEL_ELF) | sort | rustfilt

+
+
+##--------------------------------------------------------------------------------------------------
+## Testing targets
+##--------------------------------------------------------------------------------------------------
+.PHONY: test test_boot
+
+ifeq ($(QEMU_MACHINE_TYPE),) # QEMU is not supported for the board.
+
+test_boot test:
+	$(call color_header, "$(QEMU_MISSING_STRING)")
+
+else # QEMU is supported.
+
+##------------------------------------------------------------------------------
+## Run boot test
+##------------------------------------------------------------------------------
+test_boot: $(KERNEL_BIN)
+	$(call color_header, "Boot test - $(BSP)")
+	@$(DOCKER_TEST) $(EXEC_TEST_DISPATCH) $(EXEC_QEMU) $(QEMU_RELEASE_ARGS) -kernel $(KERNEL_BIN)
+
+test: test_boot
+
+endif

diff -uNr 02_runtime_init/src/bsp/raspberrypi/console.rs 03_hacky_hello_world/src/bsp/raspberrypi/console.rs
--- 02_runtime_init/src/bsp/raspberrypi/console.rs
+++ 03_hacky_hello_world/src/bsp/raspberrypi/console.rs
@@ -0,0 +1,47 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
+
+//! BSP console facilities.
+
+use crate::console;
+use core::fmt;
+
+//--------------------------------------------------------------------------------------------------
+// Private Definitions
+//--------------------------------------------------------------------------------------------------
+
+/// A mystical, magical device for generating QEMU output out of the void.
+struct QEMUOutput;
+
+//--------------------------------------------------------------------------------------------------
+// Private Code
+//--------------------------------------------------------------------------------------------------
+
+/// Implementing `core::fmt::Write` enables usage of the `format_args!` macros, which in turn are
+/// used to implement the `kernel`'s `print!` and `println!` macros. By implementing `write_str()`,
+/// we get `write_fmt()` automatically.
+///
+/// See [`src/print.rs`].
+///
+/// [`src/print.rs`]: ../../print/index.html
+impl fmt::Write for QEMUOutput {
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
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+/// Return a reference to the console.
+pub fn console() -> impl console::interface::Write {
+    QEMUOutput {}
+}

diff -uNr 02_runtime_init/src/bsp/raspberrypi.rs 03_hacky_hello_world/src/bsp/raspberrypi.rs
--- 02_runtime_init/src/bsp/raspberrypi.rs
+++ 03_hacky_hello_world/src/bsp/raspberrypi.rs
@@ -4,4 +4,5 @@

 //! Top-level BSP file for the Raspberry Pi 3 and 4.

+pub mod console;
 pub mod cpu;

diff -uNr 02_runtime_init/src/console.rs 03_hacky_hello_world/src/console.rs
--- 02_runtime_init/src/console.rs
+++ 03_hacky_hello_world/src/console.rs
@@ -0,0 +1,32 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
+
+//! System console.
+
+use crate::bsp;
+
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
+/// Console interfaces.
+pub mod interface {
+    /// Console write functions.
+    ///
+    /// `core::fmt::Write` is exactly what we need for now. Re-export it here because
+    /// implementing `console::Write` gives a better hint to the reader about the
+    /// intention.
+    pub use core::fmt::Write;
+}
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+/// Return a reference to the console.
+///
+/// This is the global console used by all printing macros.
+pub fn console() -> impl interface::Write {
+    bsp::console::console()
+}

diff -uNr 02_runtime_init/src/main.rs 03_hacky_hello_world/src/main.rs
--- 02_runtime_init/src/main.rs
+++ 03_hacky_hello_world/src/main.rs
@@ -107,12 +107,16 @@
 //! 2. Once finished with architectural setup, the arch code calls `kernel_init()`.

 #![feature(asm_const)]
+#![feature(format_args_nl)]
+#![feature(panic_info_message)]
 #![no_main]
 #![no_std]

 mod bsp;
+mod console;
 mod cpu;
 mod panic_wait;
+mod print;

 /// Early init code.
 ///
@@ -120,5 +124,7 @@
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
@@ -4,14 +4,61 @@

 //! A panic handler that infinitely waits.

-use crate::cpu;
+use crate::{cpu, println};
 use core::panic::PanicInfo;

 //--------------------------------------------------------------------------------------------------
 // Private Code
 //--------------------------------------------------------------------------------------------------

+/// Stop immediately if called a second time.
+///
+/// # Note
+///
+/// Using atomics here relieves us from needing to use `unsafe` for the static variable.
+///
+/// On `AArch64`, which is the only implemented architecture at the time of writing this,
+/// [`AtomicBool::load`] and [`AtomicBool::store`] are lowered to ordinary load and store
+/// instructions. They are therefore safe to use even with MMU + caching deactivated.
+///
+/// [`AtomicBool::load`]: core::sync::atomic::AtomicBool::load
+/// [`AtomicBool::store`]: core::sync::atomic::AtomicBool::store
+fn panic_prevent_reenter() {
+    use core::sync::atomic::{AtomicBool, Ordering};
+
+    #[cfg(not(target_arch = "aarch64"))]
+    compile_error!("Add the target_arch to above's check if the following code is safe to use");
+
+    static PANIC_IN_PROGRESS: AtomicBool = AtomicBool::new(false);
+
+    if !PANIC_IN_PROGRESS.load(Ordering::Relaxed) {
+        PANIC_IN_PROGRESS.store(true, Ordering::Relaxed);
+
+        return;
+    }
+
+    cpu::wait_forever()
+}
+
 #[panic_handler]
-fn panic(_info: &PanicInfo) -> ! {
+fn panic(info: &PanicInfo) -> ! {
+    // Protect against panic infinite loops if any of the following code panics itself.
+    panic_prevent_reenter();
+
+    let (location, line, column) = match info.location() {
+        Some(loc) => (loc.file(), loc.line(), loc.column()),
+        _ => ("???", 0, 0),
+    };
+
+    println!(
+        "Kernel panic!\n\n\
+        Panic location:\n      File '{}', line {}, column {}\n\n\
+        {}",
+        location,
+        line,
+        column,
+        info.message().unwrap_or(&format_args!("")),
+    );
+
     cpu::wait_forever()
 }

diff -uNr 02_runtime_init/src/print.rs 03_hacky_hello_world/src/print.rs
--- 02_runtime_init/src/print.rs
+++ 03_hacky_hello_world/src/print.rs
@@ -0,0 +1,38 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
+
+//! Printing.
+
+use crate::console;
+use core::fmt;
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+#[doc(hidden)]
+pub fn _print(args: fmt::Arguments) {
+    use console::interface::Write;
+
+    console::console().write_fmt(args).unwrap();
+}
+
+/// Prints without a newline.
+///
+/// Carbon copy from <https://doc.rust-lang.org/src/std/macros.rs.html>
+#[macro_export]
+macro_rules! print {
+    ($($arg:tt)*) => ($crate::print::_print(format_args!($($arg)*)));
+}
+
+/// Prints with a newline.
+///
+/// Carbon copy from <https://doc.rust-lang.org/src/std/macros.rs.html>
+#[macro_export]
+macro_rules! println {
+    () => ($crate::print!("\n"));
+    ($($arg:tt)*) => ({
+        $crate::print::_print(format_args_nl!($($arg)*));
+    })
+}

diff -uNr 02_runtime_init/tests/boot_test_string.rb 03_hacky_hello_world/tests/boot_test_string.rb
--- 02_runtime_init/tests/boot_test_string.rb
+++ 03_hacky_hello_world/tests/boot_test_string.rb
@@ -0,0 +1,3 @@
+# frozen_string_literal: true
+
+EXPECTED_PRINT = 'Stopping here'

```
