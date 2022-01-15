# 教程 02 - 执行初始化

## tl;dr

我们拓展了`boot.S`，在第一次启动的时候调用Rust代码。在Rust的代码中先清零了[bss] section，然后通过调用`panic()`挂起CPU。再次运行`make qemu`看看新增加的代码是怎么运行的。

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

diff -uNr 01_wait_forever/Cargo.toml 02_runtime_init/Cargo.toml
--- 01_wait_forever/Cargo.toml
+++ 02_runtime_init/Cargo.toml
@@ -4,6 +4,9 @@
 authors = ["Andre Richter <andre.o.richter@gmail.com>"]
 edition = "2018"

+[profile.release]
+lto = true
+
 # The features section is used to select the target board.
 [features]
 default = []

diff -uNr 01_wait_forever/src/_arch/aarch64/cpu/boot.S 02_runtime_init/src/_arch/aarch64/cpu/boot.S
--- 01_wait_forever/src/_arch/aarch64/cpu/boot.S
+++ 02_runtime_init/src/_arch/aarch64/cpu/boot.S
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

diff -uNr 01_wait_forever/src/_arch/aarch64/cpu.rs 02_runtime_init/src/_arch/aarch64/cpu.rs
--- 01_wait_forever/src/_arch/aarch64/cpu.rs
+++ 02_runtime_init/src/_arch/aarch64/cpu.rs
@@ -0,0 +1,30 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2022 Andre Richter <andre.o.richter@gmail.com>
+
+//! Architectural processor code.
+//!
+//! # Orientation
+//!
+//! Since arch modules are imported into generic modules using the path attribute, the path of this
+//! file is:
+//!
+//! crate::cpu::arch_cpu
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+/// Pause execution on the core.
+#[inline(always)]
+pub fn wait_forever() -> ! {
+    unsafe {
+        loop {
+            #[rustfmt::skip]
+            asm!(
+                "wfe",
+                options(nomem, nostack, preserves_flags)
+            );
+        }
+    }
+}

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
+// Copyright (c) 2018-2022 Andre Richter <andre.o.richter@gmail.com>
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

diff -uNr 01_wait_forever/src/cpu.rs 02_runtime_init/src/cpu.rs
--- 01_wait_forever/src/cpu.rs
+++ 02_runtime_init/src/cpu.rs
@@ -4,4 +4,13 @@

 //! Processor code.

+#[cfg(target_arch = "aarch64")]
+#[path = "_arch/aarch64/cpu.rs"]
+mod arch_cpu;
+
 mod boot;
+
+//--------------------------------------------------------------------------------------------------
+// Architectural Public Reexports
+//--------------------------------------------------------------------------------------------------
+pub use arch_cpu::wait_forever;

diff -uNr 01_wait_forever/src/main.rs 02_runtime_init/src/main.rs
--- 01_wait_forever/src/main.rs
+++ 02_runtime_init/src/main.rs
@@ -102,6 +102,7 @@
 //!
 //! 1. The kernel's entry point is the function [`cpu::boot::arch_boot::_start()`].
 //!     - It is implemented in `src/_arch/__arch_name__/cpu/boot.rs`.
+//! 2. Once finished with architectural setup, the arch code calls [`runtime_init::runtime_init()`].
 //!
 //! [`cpu::boot::arch_boot::_start()`]: cpu/boot/arch_boot/fn._start.html

@@ -112,6 +113,15 @@

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
+// Copyright (c) 2018-2022 Andre Richter <andre.o.richter@gmail.com>
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

diff -uNr 01_wait_forever/src/panic_wait.rs 02_runtime_init/src/panic_wait.rs
--- 01_wait_forever/src/panic_wait.rs
+++ 02_runtime_init/src/panic_wait.rs
@@ -4,9 +4,10 @@

 //! A panic handler that infinitely waits.

+use crate::cpu;
 use core::panic::PanicInfo;

 #[panic_handler]
 fn panic(_info: &PanicInfo) -> ! {
-    unimplemented!()
+    cpu::wait_forever()
 }

diff -uNr 01_wait_forever/src/runtime_init.rs 02_runtime_init/src/runtime_init.rs
--- 01_wait_forever/src/runtime_init.rs
+++ 02_runtime_init/src/runtime_init.rs
@@ -0,0 +1,38 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2022 Andre Richter <andre.o.richter@gmail.com>
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
