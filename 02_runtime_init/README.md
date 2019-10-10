# Tutorial 02 - Runtime Init

## tl;dr

We are calling into Rust code for the first time.

- More sections in linker script:
     - `.rodata`, `.data`
     - `.bss`
- `_start()`:
     - Halt core if core != `core0`.
     - `core0` jumps to `init()` Rust function.
- `init()`
     - Zeros the `.bss` section.
     - Calls `kernel_entry()`, which calls `panic!()`, which eventually halts
       `core0` as well.

## Diff to previous
```diff

diff -uNr 01_wait_forever/Cargo.toml 02_runtime_init/Cargo.toml
--- 01_wait_forever/Cargo.toml
+++ 02_runtime_init/Cargo.toml
@@ -13,4 +13,4 @@
 bsp_rpi3 = []

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
-    b       1b  // In case an event happend, jump back to 1
+    mrs     x1, mpidr_el1   // Read Multiprocessor Affinity Register
+    and     x1, x1, #3      // Clear all bits except [1:0], which hold core id
+    cbz     x1, 2f          // Jump to label 2 if we are core 0
+1:  wfe                     // Wait for event
+    b       1b              // In case an event happend, jump back to 1
+2:                          // If we are here, we are core0
+    ldr     x1, =_start     // Load address of function "_start()"
+    mov     sp, x1          // Set start of stack to before our code, aka first
+                            // address before "_start()"
+    bl      init            // Jump to the "init()" kernel function
+    b       1b              // We should never reach here. But just in case,
+                            // park this core aswell

diff -uNr 01_wait_forever/src/bsp/rpi3/link.ld 02_runtime_init/src/bsp/rpi3/link.ld
--- 01_wait_forever/src/bsp/rpi3/link.ld
+++ 02_runtime_init/src/bsp/rpi3/link.ld
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
@@ -16,9 +16,16 @@
 // `_start()` function, the first function to run.
 mod arch;

+// `_start()` then calls `runtime_init::init()`, which on completion, jumps to
+// `kernel_entry()`.
+mod runtime_init;
+
 // Conditionally includes the selected `BSP` code.
 mod bsp;

 mod panic_wait;

-// Kernel code coming next tutorial.
+/// Entrypoint of the `kernel`.
+fn kernel_entry() -> ! {
+    panic!()
+}

diff -uNr 01_wait_forever/src/runtime_init.rs 02_runtime_init/src/runtime_init.rs
--- 01_wait_forever/src/runtime_init.rs
+++ 02_runtime_init/src/runtime_init.rs
@@ -0,0 +1,27 @@
+// SPDX-License-Identifier: MIT
+//
+// Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>
+
+//! Rust runtime initialization code.
+
+/// Equivalent to `crt0` or `c0` code in C/C++ world. Clears the `bss` section,
+/// then calls the kernel entry.
+///
+/// Called from `BSP` code.
+///
+/// # Safety
+///
+/// - Only a single core must be active and running this function.
+#[no_mangle]
+pub unsafe extern "C" fn init() -> ! {
+    extern "C" {
+        // Boundaries of the .bss section, provided by the linker script
+        static mut __bss_start: u64;
+        static mut __bss_end: u64;
+    }
+
+    // Zero out the .bss section
+    r0::zero_bss(&mut __bss_start, &mut __bss_end);
+
+    crate::kernel_entry()
+}
```
