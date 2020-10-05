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
@@ -7,7 +7,10 @@
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
+cortex-a = { version = "3.0.x", optional = true }

diff -uNr 03_hacky_hello_world/src/_arch/aarch64/cpu/smp.rs 04_zero_overhead_abstraction/src/_arch/aarch64/cpu/smp.rs
--- 03_hacky_hello_world/src/_arch/aarch64/cpu/smp.rs
+++ 04_zero_overhead_abstraction/src/_arch/aarch64/cpu/smp.rs
@@ -0,0 +1,22 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
+
+//! Architectural symmetric multiprocessing.
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
@@ -4,8 +4,34 @@

 //! Architectural processor code.

-// Assembly counterpart to this file.
-global_asm!(include_str!("cpu.S"));
+use crate::{bsp, cpu};
+use cortex_a::{asm, regs::*};
+
+//--------------------------------------------------------------------------------------------------
+// Boot Code
+//--------------------------------------------------------------------------------------------------
+
+/// The entry of the `kernel` binary.
+///
+/// The function must be named `_start`, because the linker is looking for this exact name.
+///
+/// # Safety
+///
+/// - Linker script must ensure to place this function at `0x80_000`.
+#[naked]
+#[no_mangle]
+pub unsafe fn _start() -> ! {
+    use crate::runtime_init;
+
+    // Expect the boot core to start in EL2.
+    if bsp::cpu::BOOT_CORE_ID == cpu::smp::core_id() {
+        SP.set(bsp::memory::boot_core_stack_end() as u64);
+        runtime_init::runtime_init()
+    } else {
+        // If not core0, infinitely wait for events.
+        wait_forever()
+    }
+}

 //--------------------------------------------------------------------------------------------------
 // Public Code
@@ -14,13 +40,7 @@
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

diff -uNr 03_hacky_hello_world/src/_arch/aarch64/cpu.S 04_zero_overhead_abstraction/src/_arch/aarch64/cpu.S
--- 03_hacky_hello_world/src/_arch/aarch64/cpu.S
+++ 04_zero_overhead_abstraction/src/_arch/aarch64/cpu.S
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

diff -uNr 03_hacky_hello_world/src/bsp/raspberrypi/cpu.rs 04_zero_overhead_abstraction/src/bsp/raspberrypi/cpu.rs
--- 03_hacky_hello_world/src/bsp/raspberrypi/cpu.rs
+++ 04_zero_overhead_abstraction/src/bsp/raspberrypi/cpu.rs
@@ -0,0 +1,12 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
+
+//! BSP Processor code.
+
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
+/// Used by `arch` code to find the early boot core.
+pub const BOOT_CORE_ID: usize = 0;

diff -uNr 03_hacky_hello_world/src/bsp/raspberrypi/memory.rs 04_zero_overhead_abstraction/src/bsp/raspberrypi/memory.rs
--- 03_hacky_hello_world/src/bsp/raspberrypi/memory.rs
+++ 04_zero_overhead_abstraction/src/bsp/raspberrypi/memory.rs
@@ -17,9 +17,25 @@
 }

 //--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
+/// The board's memory map.
+#[rustfmt::skip]
+pub(super) mod map {
+    pub const BOOT_CORE_STACK_END: usize = 0x8_0000;
+}
+
+//--------------------------------------------------------------------------------------------------
 // Public Code
 //--------------------------------------------------------------------------------------------------

+/// Exclusive end address of the boot core's stack.
+#[inline(always)]
+pub fn boot_core_stack_end() -> usize {
+    map::BOOT_CORE_STACK_END
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
@@ -0,0 +1,10 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
+
+//! Symmetric multiprocessing.
+
+#[cfg(target_arch = "aarch64")]
+#[path = "../_arch/aarch64/cpu/smp.rs"]
+mod arch_cpu_smp;
+pub use arch_cpu_smp::*;

diff -uNr 03_hacky_hello_world/src/cpu.rs 04_zero_overhead_abstraction/src/cpu.rs
--- 03_hacky_hello_world/src/cpu.rs
+++ 04_zero_overhead_abstraction/src/cpu.rs
@@ -8,3 +8,5 @@
 #[path = "_arch/aarch64/cpu.rs"]
 mod arch_cpu;
 pub use arch_cpu::*;
+
+pub mod smp;

diff -uNr 03_hacky_hello_world/src/main.rs 04_zero_overhead_abstraction/src/main.rs
--- 03_hacky_hello_world/src/main.rs
+++ 04_zero_overhead_abstraction/src/main.rs
@@ -92,9 +92,8 @@
 //! - `crate::memory::*`
 //! - `crate::bsp::memory::*`

-#![feature(asm)]
 #![feature(format_args_nl)]
-#![feature(global_asm)]
+#![feature(naked_functions)]
 #![feature(panic_info_message)]
 #![no_main]
 #![no_std]
@@ -116,7 +115,8 @@
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
