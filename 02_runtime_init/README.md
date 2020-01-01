# Tutorial 02 - Runtime Init

## tl;dr

We are calling into Rust code for the first time and zero the [bss] section.
Check out `make qemu` again to see the additional code run.

- More sections in linker script:
     - `.rodata`, `.data`
     - `.bss`
- `_start()`:
     - Halt core if core != `core0`.
     - `core0` jumps to `runtime_init()` Rust function.
- `runtime_init()` in `runtime_init.rs`
     - Zeros the `.bss` section.
     - Calls `kernel_init()`, which calls `panic!()`, which eventually halts
       `core0` as well.

[bss]: https://en.wikipedia.org/wiki/.bss

## Diff to previous
```diff

diff -uNr 01_wait_forever/Cargo.toml 02_runtime_init/Cargo.toml
--- 01_wait_forever/Cargo.toml
+++ 02_runtime_init/Cargo.toml
@@ -14,4 +14,4 @@
 bsp_rpi4 = []

 [dependencies]
-
+r0 = "0.2.*"

diff -uNr 01_wait_forever/src/arch/aarch64/start.S 02_runtime_init/src/arch/aarch64/start.S
--- 01_wait_forever/src/arch/aarch64/start.S
+++ 02_runtime_init/src/arch/aarch64/start.S
@@ -7,5 +7,15 @@
 .global _start

 _start:
-1:  wfe         // Wait for event
-    b       1b  // In case an event happened, jump back to 1
+    mrs     x1, mpidr_el1   // Read Multiprocessor Affinity Register
+    and     x1, x1, #3      // Clear all bits except [1:0], which hold core id
+    cbz     x1, 2f          // Jump to label 2 if we are core 0
+1:  wfe                     // Wait for event
+    b       1b              // In case an event happened, jump back to 1
+2:                          // If we are here, we are core0
+    ldr     x1, =_start     // Load address of function "_start()"
+    mov     sp, x1          // Set start of stack to before our code, aka first
+                            // address before "_start()"
+    bl      runtime_init    // Jump to the "runtime_init()" kernel function
+    b       1b              // We should never reach here. But just in case,
+                            // park this core aswell

diff -uNr 01_wait_forever/src/bsp/rpi/link.ld 02_runtime_init/src/bsp/rpi/link.ld
--- 01_wait_forever/src/bsp/rpi/link.ld
+++ 02_runtime_init/src/bsp/rpi/link.ld
@@ -13,5 +13,23 @@
         *(.text._start) *(.text*)
     }

+    .rodata :
+    {
+        *(.rodata*)
+    }
+
+    .data :
+    {
+        *(.data*)
+    }
+
+    /* Align to 8 byte boundary */
+    .bss ALIGN(8):
+    {
+        __bss_start = .;
+        *(.bss*);
+        __bss_end = .;
+    }
+
     /DISCARD/ : { *(.comment*) }
 }

diff -uNr 01_wait_forever/src/main.rs 02_runtime_init/src/main.rs
--- 01_wait_forever/src/main.rs
+++ 02_runtime_init/src/main.rs
@@ -16,9 +16,19 @@
 // the first function to run.
 mod arch;

+// `_start()` then calls `runtime_init()`, which on completion, jumps to `kernel_init()`.
+mod runtime_init;
+
 // Conditionally includes the selected `BSP` code.
 mod bsp;

 mod panic_wait;

-// Kernel code coming next tutorial.
+/// Early init code.
+///
+/// # Safety
+///
+/// - Only a single core must be active and running this function.
+unsafe fn kernel_init() -> ! {
+    panic!()
+}

diff -uNr 01_wait_forever/src/runtime_init.rs 02_runtime_init/src/runtime_init.rs
--- 01_wait_forever/src/runtime_init.rs
+++ 02_runtime_init/src/runtime_init.rs
@@ -0,0 +1,25 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
+
+//! Rust runtime initialization code.
+
+/// Equivalent to `crt0` or `c0` code in C/C++ world. Clears the `bss` section, then jumps to kernel
+/// init code.
+///
+/// # Safety
+///
+/// - Only a single core must be active and running this function.
+#[no_mangle]
+pub unsafe extern "C" fn runtime_init() -> ! {
+    extern "C" {
+        // Boundaries of the .bss section, provided by the linker script.
+        static mut __bss_start: u64;
+        static mut __bss_end: u64;
+    }
+
+    // Zero out the .bss section.
+    r0::zero_bss(&mut __bss_start, &mut __bss_end);
+
+    crate::kernel_init()
+}

```
