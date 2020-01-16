# Tutorial 04 - Zero Overhead Abstraction

## tl;dr

All hand-written assembly is replaced by Rust code from the [cortex-a] crate,
which provides zero-overhead abstractions and wraps the `unsafe` parts.

[cortex-a]: https://github.com/rust-embedded/cortex-a

## Diff to previous
```diff

diff -uNr 03_hacky_hello_world/Cargo.toml 04_zero_overhead_abstraction/Cargo.toml
--- 03_hacky_hello_world/Cargo.toml
+++ 04_zero_overhead_abstraction/Cargo.toml
@@ -10,7 +10,10 @@
 # The features section is used to select the target board.
 [features]
 default = []
-bsp_rpi3 = []
-bsp_rpi4 = []
+bsp_rpi3 = ["cortex-a"]
+bsp_rpi4 = ["cortex-a"]

 [dependencies]
+
+# Optional dependencies
+cortex-a = { version = "2.9.x", optional = true }

diff -uNr 03_hacky_hello_world/src/arch/aarch64/start.S 04_zero_overhead_abstraction/src/arch/aarch64/start.S
--- 03_hacky_hello_world/src/arch/aarch64/start.S
+++ 04_zero_overhead_abstraction/src/arch/aarch64/start.S
@@ -1,21 +0,0 @@
-// SPDX-License-Identifier: MIT OR Apache-2.0
-//
-// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
-
-.section ".text._start"
-
-.global _start
-
-_start:
-    mrs     x1, mpidr_el1   // Read Multiprocessor Affinity Register
-    and     x1, x1, #3      // Clear all bits except [1:0], which hold core id
-    cbz     x1, 2f          // Jump to label 2 if we are core 0
-1:  wfe                     // Wait for event
-    b       1b              // In case an event happened, jump back to 1
-2:                          // If we are here, we are core0
-    ldr     x1, =_start     // Load address of function "_start()"
-    mov     sp, x1          // Set start of stack to before our code, aka first
-                            // address before "_start()"
-    bl      runtime_init    // Jump to the "runtime_init()" kernel function
-    b       1b              // We should never reach here. But just in case,
-                            // park this core aswell

diff -uNr 03_hacky_hello_world/src/arch/aarch64.rs 04_zero_overhead_abstraction/src/arch/aarch64.rs
--- 03_hacky_hello_world/src/arch/aarch64.rs
+++ 04_zero_overhead_abstraction/src/arch/aarch64.rs
@@ -4,7 +4,28 @@

 //! AArch64.

-global_asm!(include_str!("aarch64/start.S"));
+use crate::bsp;
+use cortex_a::{asm, regs::*};
+
+/// The entry of the `kernel` binary.
+///
+/// The function must be named `_start`, because the linker is looking for this exact name.
+///
+/// # Safety
+///
+/// - Linker script must ensure to place this function at `0x80_000`.
+#[no_mangle]
+pub unsafe extern "C" fn _start() -> ! {
+    const CORE_MASK: u64 = 0x3;
+
+    if bsp::BOOT_CORE_ID == MPIDR_EL1.get() & CORE_MASK {
+        SP.set(bsp::BOOT_CORE_STACK_START);
+        crate::runtime_init::runtime_init()
+    } else {
+        // If not core0, infinitely wait for events.
+        wait_forever()
+    }
+}

 //--------------------------------------------------------------------------------------------------
 // Implementation of the kernel's architecture abstraction code
@@ -13,9 +34,7 @@
 /// Pause execution on the calling CPU core.
 #[inline(always)]
 pub fn wait_forever() -> ! {
-    unsafe {
-        loop {
-            asm!("wfe" :::: "volatile")
-        }
+    loop {
+        asm::wfe()
     }
 }

diff -uNr 03_hacky_hello_world/src/bsp/rpi.rs 04_zero_overhead_abstraction/src/bsp/rpi.rs
--- 03_hacky_hello_world/src/bsp/rpi.rs
+++ 04_zero_overhead_abstraction/src/bsp/rpi.rs
@@ -7,6 +7,12 @@
 use crate::interface;
 use core::fmt;

+/// Used by `arch` code to find the early boot core.
+pub const BOOT_CORE_ID: u64 = 0;
+
+/// The early boot core's stack address.
+pub const BOOT_CORE_STACK_START: u64 = 0x80_000;
+
 /// A mystical, magical device for generating QEMU output out of the void.
 struct QEMUOutput;


diff -uNr 03_hacky_hello_world/src/main.rs 04_zero_overhead_abstraction/src/main.rs
--- 03_hacky_hello_world/src/main.rs
+++ 04_zero_overhead_abstraction/src/main.rs
@@ -19,9 +19,7 @@
 //! [Architecture-specific code]: arch/index.html
 //! [`kernel::interface`]: interface/index.html

-#![feature(asm)]
 #![feature(format_args_nl)]
-#![feature(global_asm)]
 #![feature(panic_info_message)]
 #![no_main]
 #![no_std]
@@ -47,7 +45,8 @@
 ///
 /// - Only a single core must be active and running this function.
 unsafe fn kernel_init() -> ! {
-    println!("Hello from Rust!");
+    println!("[0] Hello from pure Rust!");

-    panic!("Stopping here.")
+    println!("[1] Stopping here.");
+    arch::wait_forever()
 }

diff -uNr 03_hacky_hello_world/src/runtime_init.rs 04_zero_overhead_abstraction/src/runtime_init.rs
--- 03_hacky_hello_world/src/runtime_init.rs
+++ 04_zero_overhead_abstraction/src/runtime_init.rs
@@ -42,8 +42,7 @@
 /// # Safety
 ///
 /// - Only a single core must be active and running this function.
-#[no_mangle]
-pub unsafe extern "C" fn runtime_init() -> ! {
+pub unsafe fn runtime_init() -> ! {
     zero_bss();

     crate::kernel_init()

```
