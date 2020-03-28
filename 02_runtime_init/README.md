# Tutorial 02 - Runtime Init

## tl;dr

We extend `cpu.S` to call into Rust code for the first time. There,we zero the [bss] section before
execution is halted with a call to `panic()`. Check out `make qemu` again to see the additional code
run.

## Notable additions

- More sections in linker script:
     - `.rodata`, `.data`
     - `.bss`
- `_start()`:
     - Halt core if core != `core0`.
     - `core0` jumps to the `runtime_init()` Rust function.
- `runtime_init()` in `runtime_init.rs`
     - Zeros the `.bss` section.
     - Calls `kernel_init()`, which calls `panic!()`, which eventually halts `core0` as well.

[bss]: https://en.wikipedia.org/wiki/.bss

## Diff to previous
```diff

diff -uNr 01_wait_forever/src/_arch/aarch64/cpu.S 02_runtime_init/src/_arch/aarch64/cpu.S
--- 01_wait_forever/src/_arch/aarch64/cpu.S
+++ 02_runtime_init/src/_arch/aarch64/cpu.S
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

diff -uNr 01_wait_forever/src/bsp/raspberrypi/link.ld 02_runtime_init/src/bsp/raspberrypi/link.ld
--- 01_wait_forever/src/bsp/raspberrypi/link.ld
+++ 02_runtime_init/src/bsp/raspberrypi/link.ld
@@ -13,5 +13,24 @@
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
+    /* Section is zeroed in u64 chunks, align start and end to 8 bytes */
+    .bss ALIGN(8):
+    {
+        __bss_start = .;
+        *(.bss*);
+        . = ALIGN(8);
+        __bss_end = .;
+    }
+
     /DISCARD/ : { *(.comment*) }
 }

diff -uNr 01_wait_forever/src/main.rs 02_runtime_init/src/main.rs
--- 01_wait_forever/src/main.rs
+++ 02_runtime_init/src/main.rs
@@ -97,10 +97,20 @@
 #![no_main]
 #![no_std]

-// `mod cpu` provides the `_start()` function, the first function to run.
+// `mod cpu` provides the `_start()` function, the first function to run. `_start()` then calls
+// `runtime_init()`, which jumps to `kernel_init()`.

 mod bsp;
 mod cpu;
+mod memory;
 mod panic_wait;
+mod runtime_init;

-// Kernel code coming next tutorial.
+/// Early init code.
+///
+/// # Safety
+///
+/// - Only a single core must be active and running this function.
+unsafe fn kernel_init() -> ! {
+    panic!()
+}

diff -uNr 01_wait_forever/src/memory.rs 02_runtime_init/src/memory.rs
--- 01_wait_forever/src/memory.rs
+++ 02_runtime_init/src/memory.rs
@@ -0,0 +1,29 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
+
+//! Memory Management.
+
+use core::ops::Range;
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+/// Zero out a memory region.
+///
+/// # Safety
+///
+/// - `range.start` and `range.end` must be valid.
+/// - `range.start` and `range.end` must be `T` aligned.
+pub unsafe fn zero_volatile<T>(range: Range<*mut T>)
+where
+    T: From<u8>,
+{
+    let mut ptr = range.start;
+
+    while ptr < range.end {
+        core::ptr::write_volatile(ptr, T::from(0));
+        ptr = ptr.offset(1);
+    }
+}

diff -uNr 01_wait_forever/src/runtime_init.rs 02_runtime_init/src/runtime_init.rs
--- 01_wait_forever/src/runtime_init.rs
+++ 02_runtime_init/src/runtime_init.rs
@@ -0,0 +1,58 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
+
+//! Rust runtime initialization code.
+
+use crate::memory;
+use core::ops::Range;
+
+//--------------------------------------------------------------------------------------------------
+// Private Code
+//--------------------------------------------------------------------------------------------------
+
+/// Return the range spanning the .bss section.
+///
+/// # Safety
+///
+/// - The symbol-provided addresses must be valid.
+/// - The symbol-provided addresses must be usize aligned.
+unsafe fn bss_range() -> Range<*mut usize> {
+    extern "C" {
+        // Boundaries of the .bss section, provided by linker script symbols.
+        static mut __bss_start: usize;
+        static mut __bss_end: usize;
+    }
+
+    Range {
+        start: &mut __bss_start,
+        end: &mut __bss_end,
+    }
+}
+
+/// Zero out the .bss section.
+///
+/// # Safety
+///
+/// - Must only be called pre `kernel_init()`.
+#[inline(always)]
+unsafe fn zero_bss() {
+    memory::zero_volatile(bss_range());
+}
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+/// Equivalent to `crt0` or `c0` code in C/C++ world. Clears the `bss` section, then jumps to kernel
+/// init code.
+///
+/// # Safety
+///
+/// - Only a single core must be active and running this function.
+#[no_mangle]
+pub unsafe extern "C" fn runtime_init() -> ! {
+    zero_bss();
+
+    crate::kernel_init()
+}

```
