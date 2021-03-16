# Tutorial 04 - Zero Overhead Abstraction

## tl;dr

- All hand-written assembly is replaced by Rust code from the [cortex-a] crate, which provides
  zero-overhead abstractions and wraps the `unsafe` parts.

[cortex-a]: https://github.com/rust-embedded/cortex-a

## Diff to previous
```diff

diff -uNr 03_hacky_hello_world/Cargo.toml 04_zero_overhead_abstraction/Cargo.toml
--- 03_hacky_hello_world/Cargo.toml
+++ 04_zero_overhead_abstraction/Cargo.toml
@@ -17,3 +17,8 @@
 ##--------------------------------------------------------------------------------------------------

 [dependencies]
+
+# Platform specific dependencies
+[target.'cfg(target_arch = "aarch64")'.dependencies]
+cortex-a = { version = "5.x.x" }
+

diff -uNr 03_hacky_hello_world/src/_arch/aarch64/cpu/boot.rs 04_zero_overhead_abstraction/src/_arch/aarch64/cpu/boot.rs
--- 03_hacky_hello_world/src/_arch/aarch64/cpu/boot.rs
+++ 04_zero_overhead_abstraction/src/_arch/aarch64/cpu/boot.rs
@@ -11,5 +11,31 @@
 //!
 //! crate::cpu::boot::arch_boot

-// Assembly counterpart to this file. Includes function _start().
-global_asm!(include_str!("boot.S"));
+use crate::{bsp, cpu};
+use cortex_a::regs::*;
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+/// The entry of the `kernel` binary.
+///
+/// The function must be named `_start`, because the linker is looking for this exact name.
+///
+/// # Safety
+///
+/// - Linker script must ensure to place this function where it is expected by the target machine.
+/// - We have to hope that the compiler omits any stack pointer usage before the stack pointer is
+///   actually set (`SP.set()`).
+#[no_mangle]
+pub unsafe fn _start() -> ! {
+    use crate::runtime_init;
+
+    if bsp::cpu::BOOT_CORE_ID == cpu::smp::core_id() {
+        SP.set(bsp::memory::boot_core_stack_end() as u64);
+        runtime_init::runtime_init()
+    } else {
+        // If not core0, infinitely wait for events.
+        cpu::wait_forever()
+    }
+}

diff -uNr 03_hacky_hello_world/src/_arch/aarch64/cpu/boot.S 04_zero_overhead_abstraction/src/_arch/aarch64/cpu/boot.S
--- 03_hacky_hello_world/src/_arch/aarch64/cpu/boot.S
+++ 04_zero_overhead_abstraction/src/_arch/aarch64/cpu/boot.S
@@ -1,21 +0,0 @@
-// SPDX-License-Identifier: MIT OR Apache-2.0
-//
-// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
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

diff -uNr 03_hacky_hello_world/src/_arch/aarch64/cpu/smp.rs 04_zero_overhead_abstraction/src/_arch/aarch64/cpu/smp.rs
--- 03_hacky_hello_world/src/_arch/aarch64/cpu/smp.rs
+++ 04_zero_overhead_abstraction/src/_arch/aarch64/cpu/smp.rs
@@ -0,0 +1,29 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
+
+//! Architectural symmetric multiprocessing.
+//!
+//! # Orientation
+//!
+//! Since arch modules are imported into generic modules using the path attribute, the path of this
+//! file is:
+//!
+//! crate::cpu::smp::arch_smp
+
+use cortex_a::regs::*;
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+/// Return the executing core's id.
+#[inline(always)]
+pub fn core_id<T>() -> T
+where
+    T: From<u8>,
+{
+    const CORE_MASK: u64 = 0b11;
+
+    T::from((MPIDR_EL1.get() & CORE_MASK) as u8)
+}

diff -uNr 03_hacky_hello_world/src/_arch/aarch64/cpu.rs 04_zero_overhead_abstraction/src/_arch/aarch64/cpu.rs
--- 03_hacky_hello_world/src/_arch/aarch64/cpu.rs
+++ 04_zero_overhead_abstraction/src/_arch/aarch64/cpu.rs
@@ -11,6 +11,8 @@
 //!
 //! crate::cpu::arch_cpu

+use cortex_a::asm;
+
 //--------------------------------------------------------------------------------------------------
 // Public Code
 //--------------------------------------------------------------------------------------------------
@@ -18,13 +20,7 @@
 /// Pause execution on the core.
 #[inline(always)]
 pub fn wait_forever() -> ! {
-    unsafe {
-        loop {
-            #[rustfmt::skip]
-            asm!(
-                "wfe",
-                options(nomem, nostack, preserves_flags)
-            );
-        }
+    loop {
+        asm::wfe()
     }
 }

diff -uNr 03_hacky_hello_world/src/bsp/raspberrypi/cpu.rs 04_zero_overhead_abstraction/src/bsp/raspberrypi/cpu.rs
--- 03_hacky_hello_world/src/bsp/raspberrypi/cpu.rs
+++ 04_zero_overhead_abstraction/src/bsp/raspberrypi/cpu.rs
@@ -0,0 +1,12 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
+
+//! BSP Processor code.
+
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
+/// Used by `arch` code to find the early boot core.
+pub const BOOT_CORE_ID: u64 = 0;

diff -uNr 03_hacky_hello_world/src/bsp/raspberrypi/link.ld 04_zero_overhead_abstraction/src/bsp/raspberrypi/link.ld
--- 03_hacky_hello_world/src/bsp/raspberrypi/link.ld
+++ 04_zero_overhead_abstraction/src/bsp/raspberrypi/link.ld
@@ -21,6 +21,7 @@
     /***********************************************************************************************
     * Code + RO Data + Global Offset Table
     ***********************************************************************************************/
+    __rx_start = .;
     .text :
     {
         KEEP(*(.text._start))

diff -uNr 03_hacky_hello_world/src/bsp/raspberrypi/memory.rs 04_zero_overhead_abstraction/src/bsp/raspberrypi/memory.rs
--- 03_hacky_hello_world/src/bsp/raspberrypi/memory.rs
+++ 04_zero_overhead_abstraction/src/bsp/raspberrypi/memory.rs
@@ -12,14 +12,36 @@

 // Symbols from the linker script.
 extern "Rust" {
+    static __rx_start: UnsafeCell<()>;
+
     static __bss_start: UnsafeCell<u64>;
     static __bss_end_inclusive: UnsafeCell<u64>;
 }

 //--------------------------------------------------------------------------------------------------
+// Private Code
+//--------------------------------------------------------------------------------------------------
+
+/// Start address of the Read+Execute (RX) range.
+///
+/// # Safety
+///
+/// - Value is provided by the linker script and must be trusted as-is.
+#[inline(always)]
+fn rx_start() -> usize {
+    unsafe { __rx_start.get() as usize }
+}
+
+//--------------------------------------------------------------------------------------------------
 // Public Code
 //--------------------------------------------------------------------------------------------------

+/// Exclusive end address of the boot core's stack.
+#[inline(always)]
+pub fn boot_core_stack_end() -> usize {
+    rx_start()
+}
+
 /// Return the inclusive range spanning the .bss section.
 ///
 /// # Safety

diff -uNr 03_hacky_hello_world/src/bsp/raspberrypi.rs 04_zero_overhead_abstraction/src/bsp/raspberrypi.rs
--- 03_hacky_hello_world/src/bsp/raspberrypi.rs
+++ 04_zero_overhead_abstraction/src/bsp/raspberrypi.rs
@@ -5,4 +5,5 @@
 //! Top-level BSP file for the Raspberry Pi 3 and 4.

 pub mod console;
+pub mod cpu;
 pub mod memory;

diff -uNr 03_hacky_hello_world/src/cpu/smp.rs 04_zero_overhead_abstraction/src/cpu/smp.rs
--- 03_hacky_hello_world/src/cpu/smp.rs
+++ 04_zero_overhead_abstraction/src/cpu/smp.rs
@@ -0,0 +1,14 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
+
+//! Symmetric multiprocessing.
+
+#[cfg(target_arch = "aarch64")]
+#[path = "../_arch/aarch64/cpu/smp.rs"]
+mod arch_smp;
+
+//--------------------------------------------------------------------------------------------------
+// Architectural Public Reexports
+//--------------------------------------------------------------------------------------------------
+pub use arch_smp::core_id;

diff -uNr 03_hacky_hello_world/src/cpu.rs 04_zero_overhead_abstraction/src/cpu.rs
--- 03_hacky_hello_world/src/cpu.rs
+++ 04_zero_overhead_abstraction/src/cpu.rs
@@ -10,6 +10,8 @@

 mod boot;

+pub mod smp;
+
 //--------------------------------------------------------------------------------------------------
 // Architectural Public Reexports
 //--------------------------------------------------------------------------------------------------

diff -uNr 03_hacky_hello_world/src/main.rs 04_zero_overhead_abstraction/src/main.rs
--- 03_hacky_hello_world/src/main.rs
+++ 04_zero_overhead_abstraction/src/main.rs
@@ -107,9 +107,7 @@
 //! [`cpu::boot::arch_boot::_start()`]: cpu/boot/arch_boot/fn._start.html
 //! [`runtime_init::runtime_init()`]: runtime_init/fn.runtime_init.html

-#![feature(asm)]
 #![feature(format_args_nl)]
-#![feature(global_asm)]
 #![feature(panic_info_message)]
 #![no_main]
 #![no_std]
@@ -128,7 +126,8 @@
 ///
 /// - Only a single core must be active and running this function.
 unsafe fn kernel_init() -> ! {
-    println!("[0] Hello from Rust!");
+    println!("[0] Hello from pure Rust!");

-    panic!("Stopping here.")
+    println!("[1] Stopping here.");
+    cpu::wait_forever()
 }

diff -uNr 03_hacky_hello_world/src/runtime_init.rs 04_zero_overhead_abstraction/src/runtime_init.rs
--- 03_hacky_hello_world/src/runtime_init.rs
+++ 04_zero_overhead_abstraction/src/runtime_init.rs
@@ -30,7 +30,6 @@
 /// # Safety
 ///
 /// - Only a single core must be active and running this function.
-#[no_mangle]
 pub unsafe fn runtime_init() -> ! {
     zero_bss();

```
