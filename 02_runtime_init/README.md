# Tutorial 02 - Runtime Init

## tl;dr

- We extend `cpu.S` to call into Rust code for the first time. There, we zero the [bss] section
  before execution is halted with a call to `panic()`.
- Check out `make qemu` again to see the additional code run.

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
@@ -13,5 +13,27 @@
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
+
+        /* Fill for the bss == 0 case, so that __bss_start <= __bss_end_inclusive holds */
+        . += 8;
+        __bss_end_inclusive = . - 8;
+    }
+
     /DISCARD/ : { *(.comment*) }
 }

diff -uNr 01_wait_forever/src/bsp/raspberrypi/memory.rs 02_runtime_init/src/bsp/raspberrypi/memory.rs
--- 01_wait_forever/src/bsp/raspberrypi/memory.rs
+++ 02_runtime_init/src/bsp/raspberrypi/memory.rs
@@ -0,0 +1,37 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
+
+//! BSP Memory Management.
+
+use core::{cell::UnsafeCell, ops::RangeInclusive};
+
+//--------------------------------------------------------------------------------------------------
+// Private Definitions
+//--------------------------------------------------------------------------------------------------
+
+// Symbols from the linker script.
+extern "Rust" {
+    static __bss_start: UnsafeCell<u64>;
+    static __bss_end_inclusive: UnsafeCell<u64>;
+}
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+/// Return the inclusive range spanning the .bss section.
+///
+/// # Safety
+///
+/// - Values are provided by the linker script and must be trusted as-is.
+/// - The linker-provided addresses must be u64 aligned.
+pub fn bss_range_inclusive() -> RangeInclusive<*mut u64> {
+    let range;
+    unsafe {
+        range = RangeInclusive::new(__bss_start.get(), __bss_end_inclusive.get());
+    }
+    assert!(!range.is_empty());
+
+    range
+}

diff -uNr 01_wait_forever/src/bsp/raspberrypi.rs 02_runtime_init/src/bsp/raspberrypi.rs
--- 01_wait_forever/src/bsp/raspberrypi.rs
+++ 02_runtime_init/src/bsp/raspberrypi.rs
@@ -4,4 +4,4 @@

 //! Top-level BSP file for the Raspberry Pi 3 and 4.

-// Coming soon.
+pub mod memory;

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
@@ -0,0 +1,30 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
+
+//! Memory Management.
+
+use core::ops::RangeInclusive;
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+/// Zero out an inclusive memory range.
+///
+/// # Safety
+///
+/// - `range.start` and `range.end` must be valid.
+/// - `range.start` and `range.end` must be `T` aligned.
+pub unsafe fn zero_volatile<T>(range: RangeInclusive<*mut T>)
+where
+    T: From<u8>,
+{
+    let mut ptr = *range.start();
+    let end_inclusive = *range.end();
+
+    while ptr <= end_inclusive {
+        core::ptr::write_volatile(ptr, T::from(0));
+        ptr = ptr.offset(1);
+    }
+}

diff -uNr 01_wait_forever/src/runtime_init.rs 02_runtime_init/src/runtime_init.rs
--- 01_wait_forever/src/runtime_init.rs
+++ 02_runtime_init/src/runtime_init.rs
@@ -0,0 +1,38 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
+
+//! Rust runtime initialization code.
+
+use crate::{bsp, memory};
+
+//--------------------------------------------------------------------------------------------------
+// Private Code
+//--------------------------------------------------------------------------------------------------
+
+/// Zero out the .bss section.
+///
+/// # Safety
+///
+/// - Must only be called pre `kernel_init()`.
+#[inline(always)]
+unsafe fn zero_bss() {
+    memory::zero_volatile(bsp::memory::bss_range_inclusive());
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
+pub unsafe fn runtime_init() -> ! {
+    zero_bss();
+
+    crate::kernel_init()
+}

```
