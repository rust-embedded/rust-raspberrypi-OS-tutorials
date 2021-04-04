# Tutorial 02 - Runtime Init

## tl;dr

- We extend `boot.s` to call into Rust code for the first time. There, we zero the [bss] section
  before execution is halted with a call to `panic()`.
- Check out `make qemu` again to see the additional code run.

## Notable additions

- More additions to the linker script:
     - New sections: `.rodata`, `.got`, `.data`, `.bss`.
     - A dedicated place for linking boot-time arguments that need to be read by `_start()`.
- `_start()` in `_arch/__arch_name__/cpu/boot.s`:
     1. Halt core if core != core0.
     1. Set up the `stack pointer`.
     1. Jump to the `_start_rust()` function, defined in `arch/__arch_name__/cpu/boot.rs`.
- `runtime_init()` in `runtime_init.rs`:
     - Zeros the `.bss` section.
     - Calls `kernel_init()`, which calls `panic!()`, which eventually halts core0 as well.
- The library now uses the [cortex-a] crate, which provides zero-overhead abstractions and wraps
  `unsafe` parts when dealing with the CPU's resources.
    - See it in action in `_arch/__arch_name__/cpu.rs`.

[bss]: https://en.wikipedia.org/wiki/.bss
[cortex-a]: https://github.com/rust-embedded/cortex-a

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
 edition = "2018"

@@ -21,3 +21,7 @@
 ##--------------------------------------------------------------------------------------------------

 [dependencies]
+
+# Platform specific dependencies
+[target.'cfg(target_arch = "aarch64")'.dependencies]
+cortex-a = { version = "5.x.x" }

diff -uNr 01_wait_forever/Makefile 02_runtime_init/Makefile
--- 01_wait_forever/Makefile
+++ 02_runtime_init/Makefile
@@ -102,6 +102,8 @@
 	$(call colorecho, "\nLaunching objdump")
 	@$(DOCKER_TOOLS) $(OBJDUMP_BINARY) --disassemble --demangle \
                 --section .text   \
+                --section .rodata \
+                --section .got    \
                 $(KERNEL_ELF) | rustfilt

 nm: $(KERNEL_ELF)

diff -uNr 01_wait_forever/src/_arch/aarch64/cpu/boot.rs 02_runtime_init/src/_arch/aarch64/cpu/boot.rs
--- 01_wait_forever/src/_arch/aarch64/cpu/boot.rs
+++ 02_runtime_init/src/_arch/aarch64/cpu/boot.rs
@@ -11,5 +11,23 @@
 //!
 //! crate::cpu::boot::arch_boot

+use crate::runtime_init;
+
 // Assembly counterpart to this file.
 global_asm!(include_str!("boot.s"));
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+/// The Rust entry of the `kernel` binary.
+///
+/// The function is called from the assembly `_start` function.
+///
+/// # Safety
+///
+/// - The `bss` section is not initialized yet. The code must not use or reference it in any way.
+#[no_mangle]
+pub unsafe fn _start_rust() -> ! {
+    runtime_init::runtime_init()
+}

diff -uNr 01_wait_forever/src/_arch/aarch64/cpu/boot.s 02_runtime_init/src/_arch/aarch64/cpu/boot.s
--- 01_wait_forever/src/_arch/aarch64/cpu/boot.s
+++ 02_runtime_init/src/_arch/aarch64/cpu/boot.s
@@ -3,6 +3,24 @@
 // Copyright (c) 2021 Andre Richter <andre.o.richter@gmail.com>

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
+.equ _core_id_mask, 0b11
+
+//--------------------------------------------------------------------------------------------------
 // Public Code
 //--------------------------------------------------------------------------------------------------
 .section .text._start
@@ -11,6 +29,22 @@
 // fn _start()
 //------------------------------------------------------------------------------
 _start:
+	// Only proceed on the boot core. Park it otherwise.
+	mrs	x1, MPIDR_EL1
+	and	x1, x1, _core_id_mask
+	ldr	x2, BOOT_CORE_ID      // provided by bsp/__board_name__/cpu.rs
+	cmp	x1, x2
+	b.ne	1f
+
+	// If execution reaches here, it is the boot core. Now, prepare the jump to Rust code.
+
+	// Set the stack pointer.
+	ADR_REL	x0, __boot_core_stack_end_exclusive
+	mov	sp, x0
+
+	// Jump to Rust code.
+	b	_start_rust
+
 	// Infinitely wait for events (aka "park the core").
 1:	wfe
 	b	1b

diff -uNr 01_wait_forever/src/_arch/aarch64/cpu.rs 02_runtime_init/src/_arch/aarch64/cpu.rs
--- 01_wait_forever/src/_arch/aarch64/cpu.rs
+++ 02_runtime_init/src/_arch/aarch64/cpu.rs
@@ -0,0 +1,26 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
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
+use cortex_a::asm;
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
+// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
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

diff -uNr 01_wait_forever/src/bsp/raspberrypi/link.ld 02_runtime_init/src/bsp/raspberrypi/link.ld
--- 01_wait_forever/src/bsp/raspberrypi/link.ld
+++ 02_runtime_init/src/bsp/raspberrypi/link.ld
@@ -11,17 +11,45 @@
 PHDRS
 {
     segment_rx PT_LOAD FLAGS(5); /* 5 == RX */
+    segment_rw PT_LOAD FLAGS(6); /* 6 == RW */
 }

 SECTIONS
 {
     . =  __rpi_load_addr;
+                                        /*   ^             */
+                                        /*   | stack       */
+                                        /*   | growth      */
+                                        /*   | direction   */
+   __boot_core_stack_end_exclusive = .; /*   |             */

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
     } :segment_rx
+
+    .rodata : ALIGN(8) { *(.rodata*) } :segment_rx
+    .got    : ALIGN(8) { *(.got)     } :segment_rx
+
+    /***********************************************************************************************
+    * Data + BSS
+    ***********************************************************************************************/
+    .data : { *(.data*) } :segment_rw
+
+    /* Section is zeroed in u64 chunks, align start and end to 8 bytes */
+    .bss : ALIGN(8)
+    {
+        __bss_start = .;
+        *(.bss*);
+        . = ALIGN(8);
+
+        . += 8; /* Fill for the bss == 0 case, so that __bss_start <= __bss_end_inclusive holds */
+        __bss_end_inclusive = . - 8;
+    } :NONE
 }

diff -uNr 01_wait_forever/src/bsp/raspberrypi/memory.rs 02_runtime_init/src/bsp/raspberrypi/memory.rs
--- 01_wait_forever/src/bsp/raspberrypi/memory.rs
+++ 02_runtime_init/src/bsp/raspberrypi/memory.rs
@@ -0,0 +1,37 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
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
@@ -4,4 +4,5 @@

 //! Top-level BSP file for the Raspberry Pi 3 and 4.

-// Coming soon.
+pub mod cpu;
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
@@ -102,14 +102,25 @@
 //!
 //! 1. The kernel's entry point is the function `cpu::boot::arch_boot::_start()`.
 //!     - It is implemented in `src/_arch/__arch_name__/cpu/boot.s`.
+//! 2. Once finished with architectural setup, the arch code calls [`runtime_init::runtime_init()`].
+//!
+//! [`runtime_init::runtime_init()`]: runtime_init/fn.runtime_init.html

-#![feature(asm)]
 #![feature(global_asm)]
 #![no_main]
 #![no_std]

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
+// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
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
@@ -0,0 +1,37 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
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
+pub unsafe fn runtime_init() -> ! {
+    zero_bss();
+
+    crate::kernel_init()
+}

```
