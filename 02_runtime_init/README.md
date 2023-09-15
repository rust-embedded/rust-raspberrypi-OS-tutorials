# Tutorial 02 - Runtime Init

## tl;dr

- We extend `boot.s` to call into Rust code for the first time. Before the jump
  to Rust happens, a bit of runtime init work is done.
- The Rust code being called just halts execution with a call to `panic!()`.
- Check out `make qemu` again to see the additional code run.

## Notable additions

- More additions to the linker script:
     - New sections: `.rodata`, `.got`, `.data`, `.bss`.
     - A dedicated place for linking boot-time arguments that need to be read by `_start()`.
- `_start()` in `_arch/__arch_name__/cpu/boot.s`:
     1. Halts core if core != core0.
     1. Initializes the `DRAM` by zeroing the [bss] section.
     1. Sets up the `stack pointer`.
     1. Jumps to the `_start_rust()` function, defined in `arch/__arch_name__/cpu/boot.rs`.
- `_start_rust()`:
     - Calls `kernel_init()`, which calls `panic!()`, which eventually halts core0 as well.
- The library now uses the [aarch64-cpu] crate, which provides zero-overhead abstractions and wraps
  `unsafe` parts when dealing with the CPU's resources.
    - See it in action in `_arch/__arch_name__/cpu.rs`.

[bss]: https://en.wikipedia.org/wiki/.bss
[aarch64-cpu]: https://github.com/rust-embedded/aarch64-cpu

## Diff to previous
```diff

diff -uNr 01_wait_forever/Cargo.toml 02_runtime_init/Cargo.toml
--- 01_wait_forever/Cargo.toml
+++ 02_runtime_init/Cargo.toml
@@ -1,6 +1,6 @@
 [package]
 name = "mingo"
-version = "0.1.0"
+version = "0.2.0"
 authors = ["Andre Richter <andre.o.richter@gmail.com>"]
 edition = "2021"

@@ -21,3 +21,7 @@
 ##--------------------------------------------------------------------------------------------------

 [dependencies]
+
+# Platform specific dependencies
+[target.'cfg(target_arch = "aarch64")'.dependencies]
+aarch64-cpu = { version = "9.x.x" }

diff -uNr 01_wait_forever/Makefile 02_runtime_init/Makefile
--- 01_wait_forever/Makefile
+++ 02_runtime_init/Makefile
@@ -181,6 +181,7 @@
 	$(call color_header, "Launching objdump")
 	@$(DOCKER_TOOLS) $(OBJDUMP_BINARY) --disassemble --demangle \
                 --section .text   \
+                --section .rodata \
                 $(KERNEL_ELF) | rustfilt
 ##------------------------------------------------------------------------------

diff -uNr 01_wait_forever/src/_arch/aarch64/cpu/boot.rs 02_runtime_init/src/_arch/aarch64/cpu/boot.rs
--- 01_wait_forever/src/_arch/aarch64/cpu/boot.rs
+++ 02_runtime_init/src/_arch/aarch64/cpu/boot.rs
@@ -14,4 +14,19 @@
 use core::arch::global_asm;

 // Assembly counterpart to this file.
-global_asm!(include_str!("boot.s"));
+global_asm!(
+    include_str!("boot.s"),
+    CONST_CORE_ID_MASK = const 0b11
+);
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+/// The Rust entry of the `kernel` binary.
+///
+/// The function is called from the assembly `_start` function.
+#[no_mangle]
+pub unsafe fn _start_rust() -> ! {
+    crate::kernel_init()
+}

diff -uNr 01_wait_forever/src/_arch/aarch64/cpu/boot.s 02_runtime_init/src/_arch/aarch64/cpu/boot.s
--- 01_wait_forever/src/_arch/aarch64/cpu/boot.s
+++ 02_runtime_init/src/_arch/aarch64/cpu/boot.s
@@ -3,6 +3,22 @@
 // Copyright (c) 2021-2023 Andre Richter <andre.o.richter@gmail.com>

 //--------------------------------------------------------------------------------------------------
+// Definitions
+//--------------------------------------------------------------------------------------------------
+
+// Load the address of a symbol into a register, PC-relative.
+//
+// The symbol must lie within +/- 4 GiB of the Program Counter.
+//
+// # Resources
+//
+// - https://sourceware.org/binutils/docs-2.36/as/AArch64_002dRelocations.html
+.macro ADR_REL register, symbol
+	adrp	\register, \symbol
+	add	\register, \register, #:lo12:\symbol
+.endm
+
+//--------------------------------------------------------------------------------------------------
 // Public Code
 //--------------------------------------------------------------------------------------------------
 .section .text._start
@@ -11,6 +27,34 @@
 // fn _start()
 //------------------------------------------------------------------------------
 _start:
+	// Only proceed on the boot core. Park it otherwise.
+	mrs	x0, MPIDR_EL1
+	and	x0, x0, {CONST_CORE_ID_MASK}
+	ldr	x1, BOOT_CORE_ID      // provided by bsp/__board_name__/cpu.rs
+	cmp	x0, x1
+	b.ne	.L_parking_loop
+
+	// If execution reaches here, it is the boot core.
+
+	// Initialize DRAM.
+	ADR_REL	x0, __bss_start
+	ADR_REL x1, __bss_end_exclusive
+
+.L_bss_init_loop:
+	cmp	x0, x1
+	b.eq	.L_prepare_rust
+	stp	xzr, xzr, [x0], #16
+	b	.L_bss_init_loop
+
+	// Prepare the jump to Rust code.
+.L_prepare_rust:
+	// Set the stack pointer.
+	ADR_REL	x0, __boot_core_stack_end_exclusive
+	mov	sp, x0
+
+	// Jump to Rust code.
+	b	_start_rust
+
 	// Infinitely wait for events (aka "park the core").
 .L_parking_loop:
 	wfe

diff -uNr 01_wait_forever/src/_arch/aarch64/cpu.rs 02_runtime_init/src/_arch/aarch64/cpu.rs
--- 01_wait_forever/src/_arch/aarch64/cpu.rs
+++ 02_runtime_init/src/_arch/aarch64/cpu.rs
@@ -0,0 +1,26 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
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
+use aarch64_cpu::asm;
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+/// Pause execution on the core.
+#[inline(always)]
+pub fn wait_forever() -> ! {
+    loop {
+        asm::wfe()
+    }
+}

diff -uNr 01_wait_forever/src/bsp/raspberrypi/cpu.rs 02_runtime_init/src/bsp/raspberrypi/cpu.rs
--- 01_wait_forever/src/bsp/raspberrypi/cpu.rs
+++ 02_runtime_init/src/bsp/raspberrypi/cpu.rs
@@ -0,0 +1,14 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
+
+//! BSP Processor code.
+
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
+/// Used by `arch` code to find the early boot core.
+#[no_mangle]
+#[link_section = ".text._start_arguments"]
+pub static BOOT_CORE_ID: u64 = 0;

diff -uNr 01_wait_forever/src/bsp/raspberrypi/kernel.ld 02_runtime_init/src/bsp/raspberrypi/kernel.ld
--- 01_wait_forever/src/bsp/raspberrypi/kernel.ld
+++ 02_runtime_init/src/bsp/raspberrypi/kernel.ld
@@ -3,6 +3,8 @@
  * Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
  */

+__rpi_phys_dram_start_addr = 0;
+
 /* The physical address at which the the kernel binary will be loaded by the Raspberry's firmware */
 __rpi_phys_binary_load_addr = 0x80000;

@@ -13,21 +15,65 @@
  *     4 == R
  *     5 == RX
  *     6 == RW
+ *
+ * Segments are marked PT_LOAD below so that the ELF file provides virtual and physical addresses.
+ * It doesn't mean all of them need actually be loaded.
  */
 PHDRS
 {
-    segment_code PT_LOAD FLAGS(5);
+    segment_boot_core_stack PT_LOAD FLAGS(6);
+    segment_code            PT_LOAD FLAGS(5);
+    segment_data            PT_LOAD FLAGS(6);
 }

 SECTIONS
 {
-    . =  __rpi_phys_binary_load_addr;
+    . =  __rpi_phys_dram_start_addr;
+
+    /***********************************************************************************************
+    * Boot Core Stack
+    ***********************************************************************************************/
+    .boot_core_stack (NOLOAD) :
+    {
+                                             /*   ^             */
+                                             /*   | stack       */
+        . += __rpi_phys_binary_load_addr;    /*   | growth      */
+                                             /*   | direction   */
+        __boot_core_stack_end_exclusive = .; /*   |             */
+    } :segment_boot_core_stack

     /***********************************************************************************************
-    * Code
+    * Code + RO Data + Global Offset Table
     ***********************************************************************************************/
     .text :
     {
         KEEP(*(.text._start))
+        *(.text._start_arguments) /* Constants (or statics in Rust speak) read by _start(). */
+        *(.text._start_rust)      /* The Rust entry point */
+        *(.text*)                 /* Everything else */
     } :segment_code
+
+    .rodata : ALIGN(8) { *(.rodata*) } :segment_code
+
+    /***********************************************************************************************
+    * Data + BSS
+    ***********************************************************************************************/
+    .data : { *(.data*) } :segment_data
+
+    /* Section is zeroed in pairs of u64. Align start and end to 16 bytes */
+    .bss (NOLOAD) : ALIGN(16)
+    {
+        __bss_start = .;
+        *(.bss*);
+        . = ALIGN(16);
+        __bss_end_exclusive = .;
+    } :segment_data
+
+    /***********************************************************************************************
+    * Misc
+    ***********************************************************************************************/
+    .got : { *(.got*) }
+    ASSERT(SIZEOF(.got) == 0, "Relocation support not expected")
+
+    /DISCARD/ : { *(.comment*) }
 }

diff -uNr 01_wait_forever/src/bsp/raspberrypi.rs 02_runtime_init/src/bsp/raspberrypi.rs
--- 01_wait_forever/src/bsp/raspberrypi.rs
+++ 02_runtime_init/src/bsp/raspberrypi.rs
@@ -4,4 +4,4 @@

 //! Top-level BSP file for the Raspberry Pi 3 and 4.

-// Coming soon.
+pub mod cpu;

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
@@ -104,7 +104,9 @@
 //!
 //! 1. The kernel's entry point is the function `cpu::boot::arch_boot::_start()`.
 //!     - It is implemented in `src/_arch/__arch_name__/cpu/boot.s`.
+//! 2. Once finished with architectural setup, the arch code calls `kernel_init()`.

+#![feature(asm_const)]
 #![no_main]
 #![no_std]

@@ -112,4 +114,11 @@
 mod cpu;
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

diff -uNr 01_wait_forever/src/panic_wait.rs 02_runtime_init/src/panic_wait.rs
--- 01_wait_forever/src/panic_wait.rs
+++ 02_runtime_init/src/panic_wait.rs
@@ -4,6 +4,7 @@

 //! A panic handler that infinitely waits.

+use crate::cpu;
 use core::panic::PanicInfo;

 //--------------------------------------------------------------------------------------------------
@@ -12,5 +13,5 @@

 #[panic_handler]
 fn panic(_info: &PanicInfo) -> ! {
-    unimplemented!()
+    cpu::wait_forever()
 }

```
