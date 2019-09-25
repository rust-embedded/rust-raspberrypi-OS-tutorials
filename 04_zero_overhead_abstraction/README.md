# Tutorial 04 - Zero Overhead Abstraction

## tl;dr

All hand-written assembly is replaced by Rust code from the [cortex-a] crate,
which provides zero-overhead abstractions and wraps the `unsafe` parts.

[cortex-a]: https://github.com/rust-embedded/cortex-a

## Diff to previous
```diff

diff -uNr 03_hacky_hello_world/Cargo.toml 04_zero_overhead_abstraction/Cargo.toml
--- 03_hacky_hello_world/Cargo.toml	2019-09-25 14:41:51.089487788 +0200
+++ 04_zero_overhead_abstraction/Cargo.toml	2019-09-25 13:59:33.588482692 +0200
@@ -14,3 +14,4 @@

 [dependencies]
 r0 = "0.2.2"
+cortex-a = "2.7.0"

diff -uNr 03_hacky_hello_world/src/bsp/rpi3/panic_wait.rs 04_zero_overhead_abstraction/src/bsp/rpi3/panic_wait.rs
--- 03_hacky_hello_world/src/bsp/rpi3/panic_wait.rs	2019-09-25 14:41:51.093487759 +0200
+++ 04_zero_overhead_abstraction/src/bsp/rpi3/panic_wait.rs	2019-09-25 15:26:48.988205284 +0200
@@ -6,6 +6,7 @@

 use crate::println;
 use core::panic::PanicInfo;
+use cortex_a::asm;

 #[panic_handler]
 fn panic(info: &PanicInfo) -> ! {
@@ -15,9 +16,7 @@
         println!("Kernel panic!");
     }

-    unsafe {
-        loop {
-            asm!("wfe" :::: "volatile")
-        }
+    loop {
+        asm::wfe();
     }
 }

diff -uNr 03_hacky_hello_world/src/bsp/rpi3/start.S 04_zero_overhead_abstraction/src/bsp/rpi3/start.S
--- 03_hacky_hello_world/src/bsp/rpi3/start.S	2019-09-25 15:07:28.593140386 +0200
+++ 04_zero_overhead_abstraction/src/bsp/rpi3/start.S	1970-01-01 01:00:00.000000000 +0100
@@ -1,21 +0,0 @@
-// SPDX-License-Identifier: MIT
-//
-// Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>
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
-    b       1b              // In case an event happend, jump back to 1
-2:                          // If we are here, we are core0
-    ldr     x1, =_start     // Load address of function "_start()"
-    mov     sp, x1          // Set start of stack to before our code, aka first
-                            // address before "_start()"
-    bl      init            // Jump to the "init()" kernel function
-    b       1b              // We should never reach here. But just in case,
-                            // park this core aswell

diff -uNr 03_hacky_hello_world/src/bsp/rpi3.rs 04_zero_overhead_abstraction/src/bsp/rpi3.rs
--- 03_hacky_hello_world/src/bsp/rpi3.rs	2019-09-25 14:41:51.093487759 +0200
+++ 04_zero_overhead_abstraction/src/bsp/rpi3.rs	2019-09-25 15:19:14.474175689 +0200
@@ -8,8 +8,34 @@

 use crate::interface::console;
 use core::fmt;
+use cortex_a::{asm, regs::*};

-global_asm!(include_str!("rpi3/start.S"));
+/// The entry of the `kernel` binary.
+///
+/// The function must be named `_start`, because the linker is looking for this
+/// exact name.
+///
+/// # Safety
+///
+/// - Linker script must ensure to place this function at `0x80_000`.
+#[no_mangle]
+pub unsafe extern "C" fn _start() -> ! {
+    use crate::runtime_init;
+
+    const CORE_0: u64 = 0;
+    const CORE_MASK: u64 = 0x3;
+    const STACK_START: u64 = 0x80_000;
+
+    if CORE_0 == MPIDR_EL1.get() & CORE_MASK {
+        SP.set(STACK_START);
+        runtime_init::init()
+    } else {
+        // if not core0, infinitely wait for events
+        loop {
+            asm::wfe();
+        }
+    }
+}

 /// A mystical, magical device for generating QEMU output out of the void.
 struct QEMUOutput;

diff -uNr 03_hacky_hello_world/src/main.rs 04_zero_overhead_abstraction/src/main.rs
--- 03_hacky_hello_world/src/main.rs	2019-09-25 14:41:52.341478676 +0200
+++ 04_zero_overhead_abstraction/src/main.rs	2019-09-25 15:22:45.433268740 +0200
@@ -13,9 +13,7 @@
 //!
 //! [`kernel::interface`]: interface/index.html

-#![feature(asm)]
 #![feature(format_args_nl)]
-#![feature(global_asm)]
 #![feature(panic_info_message)]
 #![no_main]
 #![no_std]
@@ -33,7 +31,7 @@

 /// Entrypoint of the `kernel`.
 fn kernel_entry() -> ! {
-    println!("Hello from Rust!");
+    println!("Hello from pure Rust!");

     panic!("Stopping here.")
 }

diff -uNr 03_hacky_hello_world/src/runtime_init.rs 04_zero_overhead_abstraction/src/runtime_init.rs
--- 03_hacky_hello_world/src/runtime_init.rs	2019-09-25 14:41:51.093487759 +0200
+++ 04_zero_overhead_abstraction/src/runtime_init.rs	2019-09-25 14:00:32.560262587 +0200
@@ -13,7 +13,7 @@
 ///
 /// - Only a single core must be active and running this function.
 #[no_mangle]
-pub unsafe extern "C" fn init() -> ! {
+pub unsafe fn init() -> ! {
     extern "C" {
         // Boundaries of the .bss section, provided by the linker script
         static mut __bss_start: u64;
```
