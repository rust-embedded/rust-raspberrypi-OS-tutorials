# 教程 02 - 执行初始化

## tl;dr

我们拓展了`cpu.S`，在第一次启动的时候调用Rust代码。在Rust的代码中先清零了[bss] section，然后通过调用`panic()`挂起CPU。再次运行`make qemu`看看新增加的代码是怎么运行的。

## 值得注意的变化

- 链接脚本（linker script）中有了更多的section。
     - `.rodata`, `.data`
     - `.bss`
- `_start()`:
     - 当核心不是`core0`第0号核心的时候，挂起该CPU核心。
     - `core0`会调用Rust的函数`runtime_init()`。
- `runtime_init.rs`内的`runtime_init()`
     - 清零了`.bss` section.
     - 它调用了`kernel_init()`, 这个函数又调用了`panic!()`, panic函数最终把`core0`和其他核心一样挂起了。

[bss]: https://en.wikipedia.org/wiki/.bss

## 相比之前的变化（diff）
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
