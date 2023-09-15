# Tutorial 17 - Kernel Symbols

## tl;dr

- To enrich and augment existing and future debugging code, we add support for `kernel symbol`
  lookup.

## Table of Contents

- [Introduction](#introduction)
- [Implementation](#implementation)
  - [Linking Changes](#linking-changes)
  - [Kernel Symbols Tool](#kernel-symbols-tool)
  - [Lookup Code](#lookup-code)
- [Test it](#test-it)
- [Diff to previous](#diff-to-previous)

## Introduction

Ever since the first tutorial, it was possible to execute the `make nm` target in order to view all
`kernel symbols`. The kernel itself, however, does not have any means yet to correlate a virtual
address to a symbol during runtime. Gaining this capability would be useful for augmenting
debug-related prints. For example, when the kernel is handling an `exception`, it prints the content
of the `exception link register`, which is the program address where the CPU was executing from when
the exception happened.

Until now, in order to understand to which function or code such an address belongs to, a manual
lookup by the person debugging the issue was necessary. In this tutorial, we are adding a `data
structure` to the kernel which contains _all the symbol names and corresponding address ranges_.
This enables the kernel to print symbol names in existing and future debug-related code, which
improves triaging of issues by humans, because it does away with the manual lookup.

This tutorial is mostly is an enabler for the upcoming tutorial that will add [`backtracing`]
support.

[`backtracing`]: https://en.wikipedia.org/wiki/Stack_trace

## Implementation

First of all, a new support crate is added under `$ROOT/libraries/debug-symbol-types`. It contains
the definition for `struct Symbol`:

```rust
/// A symbol containing a size.
#[repr(C)]
pub struct Symbol {
    addr_range: Range<usize>,
    name: &'static str,
}
```

To enable the kernel to lookup symbol names, we will add an `array` to the kernel binary that
contains all the kernel symbols. Because we can query the final symbol names and addresses only
_after_ the kernel has been `linked`, the same approach as for the `translation tables` will be
used: The symbols array will be patched into a `placeholder section` of the final kernel `ELF`.

### Linking Changes

In the `kernel.ld` linker script, we define a new section named `kernel_symbols` and give it a size
of `32 KiB`:

```ld.s
    .rodata         : ALIGN(8) { *(.rodata*) } :segment_code
    .got            : ALIGN(8) { *(.got)     } :segment_code
    .kernel_symbols : ALIGN(8) {
        __kernel_symbols_start = .;
        . += 32 * 1024;
    } :segment_code
```

Also, we are providing the start address of the section through the symbol `__kernel_symbols_start`,
which will be used by our `Rust` code later on.

### Kernel Symbols Tool

Under `$ROOT/tools/kernel_symbols_tool`, we are adding a helper tool that is able to dynamically
generate an `array` of all the kernel symbols and patch it into the final kernel `ELF`. In our main
`Makefile`, we are invoking the tool after the translation table generation. In the first step, the
tool generates a temporary `Rust` file that instantiates the symbols array. Here is an example of
how this can look like:

```console
$ head ./target/aarch64-unknown-none-softfloat/release/kernel+ttables_symbols_demangled.rs
```
```rust
use debug_symbol_types::Symbol;

# [no_mangle]
# [link_section = ".rodata.symbol_desc"]
static KERNEL_SYMBOLS: [Symbol; 139] = [
    Symbol::new(18446744072635809792, 124, "_start"),
    Symbol::new(18446744072635809920, 8, "BOOT_CORE_ID"),
    Symbol::new(18446744072635809928, 8, "PHYS_KERNEL_TABLES_BASE_ADDR"),
    Symbol::new(18446744072635809936, 80, "_start_rust"),
    Symbol::new(18446744072635813888, 84, "__exception_restore_context"),
    // Many more
```

Next, the _helper crate_ `$ROOT/kernel_symbols` is compiled. This crate contains a single `main.rs`
that just includes the temporary symbols file shown above.

```rust
//! Generation of kernel symbols.

#![no_std]
#![no_main]

#[cfg(feature = "generated_symbols_available")]
include!(env!("KERNEL_SYMBOLS_DEMANGLED_RS"));
```

`KERNEL_SYMBOLS_DEMANGLED_RS` is set by the corresponding `build.rs` file. The helper crate has its
own `linker file`, which ensures that that just the array and the corresponding strings that it
references are kept:

```ld.s
SECTIONS
{
    .rodata : {
        ASSERT(. > 0xffffffff00000000, "Expected higher half address")

        KEEP(*(.rodata.symbol_desc*))
        . = ALIGN(8);
        *(.rodata*)
    }
}
```

Afterwards, `objcopy` is used to strip the produced helper crate ELF. What remains is a small
`binary blob` that just contains the symbols array and the `names` that are referenced. To ensure
that these references are valid kernel addresses (remember that those are defined as `name: &'static
str`, so basically a pointer to a kernel address), the sub-makefile compiling this helper crate
(`$ROOT/kernel_symbols.mk`) did the following:

It used the `kernel_symbols_tool` to query the virtual address of the `kernel_symbols` **section**
(of the final kernel ELF). This address was then supplied to the linker when the helper crate was
linked (emphasis on the `--section-start=.rodata=` part):

```Makefile
GET_SYMBOLS_SECTION_VIRT_ADDR = $(DOCKER_TOOLS) $(EXEC_SYMBOLS_TOOL) \
    --get_symbols_section_virt_addr $(KERNEL_SYMBOLS_OUTPUT_ELF)

RUSTFLAGS = -C link-arg=--script=$(KERNEL_SYMBOLS_LINKER_SCRIPT) \
    -C link-arg=--section-start=.rodata=$$($(GET_SYMBOLS_SECTION_VIRT_ADDR))
```

This might be a bit convoluted, but the main take away is: This ensures that the start address of
the `.rodata` section of the `kernel_symbols` helper crate is exactly the same address as the
`placeholder section` of the final kernel ELF where the symbols `binary blob` will be patched into.
The latter is the last step done by the tool.

### Lookup Code

In the kernel, we add the file `src/symbols.rs`. It makes the linker-provided symbol
`__kernel_symbols_start` that we saw earlier accesible, and also defines `NUM_KERNEL_SYMBOLS`:

```rust
#[no_mangle]
static NUM_KERNEL_SYMBOLS: u64 = 0;
```

When the `kernel_symbols_tool` patches the symbols blob into the kernel ELF, it also updates this
value to reflect the number of symbols that are available. This is needed for the code that
internally crafts the slice of symbols that the kernel uses for lookup:

```rust
fn kernel_symbol_section_virt_start_addr() -> Address<Virtual> {
    Address::new(unsafe { __kernel_symbols_start.get() as usize })
}

fn num_kernel_symbols() -> usize {
    unsafe {
        // Read volatile is needed here to prevent the compiler from optimizing NUM_KERNEL_SYMBOLS
        // away.
        core::ptr::read_volatile(&NUM_KERNEL_SYMBOLS as *const u64) as usize
    }
}

fn kernel_symbols_slice() -> &'static [Symbol] {
    let ptr = kernel_symbol_section_virt_start_addr().as_usize() as *const Symbol;

    unsafe { slice::from_raw_parts(ptr, num_kernel_symbols()) }
}
```

Lookup is done by just iterating over the slice:

```rust
/// Retrieve the symbol corresponding to a virtual address, if any.
pub fn lookup_symbol(addr: Address<Virtual>) -> Option<&'static Symbol> {
    kernel_symbols_slice()
        .iter()
        .find(|&i| i.contains(addr.as_usize()))
}
```

And that's it for this tutorial. The upcoming tutorial on `backtracing` will put this code to more
prominent use.

## Test it

For now, symbol lookup can be observed in the integration test for synchronous exception handling.
Here, the kernel now also prints the symbol name that corresponds to the value of `ELR_EL1`. In the
following case, this is `kernel_init()`, which is where the the exception is generated in the test:

```console
$ TEST=02_exception_sync_page_fault make test_integration
[...]
         -------------------------------------------------------------------
         🦀 Testing synchronous exception handling by causing a page fault
         -------------------------------------------------------------------

         [    0.002640] Writing to bottom of address space to address 1 GiB...
         [    0.004549] Kernel panic!

         Panic location:
               File 'kernel/src/_arch/aarch64/exception.rs', line 59, column 5

         CPU Exception!

         ESR_EL1: 0x96000004

         ...

         ELR_EL1: 0xffffffffc0001118
               Symbol: kernel_init
```

## Diff to previous
```diff

diff -uNr 16_virtual_mem_part4_higher_half_kernel/Cargo.toml 17_kernel_symbols/Cargo.toml
--- 16_virtual_mem_part4_higher_half_kernel/Cargo.toml
+++ 17_kernel_symbols/Cargo.toml
@@ -2,7 +2,8 @@

 members = [
         "libraries/*",
-        "kernel"
+        "kernel",
+        "kernel_symbols"
 ]

 [profile.release]

diff -uNr 16_virtual_mem_part4_higher_half_kernel/kernel/Cargo.toml 17_kernel_symbols/kernel/Cargo.toml
--- 16_virtual_mem_part4_higher_half_kernel/kernel/Cargo.toml
+++ 17_kernel_symbols/kernel/Cargo.toml
@@ -1,6 +1,6 @@
 [package]
 name = "mingo"
-version = "0.16.0"
+version = "0.17.0"
 authors = ["Andre Richter <andre.o.richter@gmail.com>"]
 edition = "2021"

@@ -16,6 +16,7 @@

 [dependencies]
 test-types = { path = "../libraries/test-types" }
+debug-symbol-types = { path = "../libraries/debug-symbol-types" }

 # Optional dependencies
 tock-registers = { version = "0.8.x", default-features = false, features = ["register_types"], optional = true }

diff -uNr 16_virtual_mem_part4_higher_half_kernel/kernel/src/_arch/aarch64/exception.rs 17_kernel_symbols/kernel/src/_arch/aarch64/exception.rs
--- 16_virtual_mem_part4_higher_half_kernel/kernel/src/_arch/aarch64/exception.rs
+++ 17_kernel_symbols/kernel/src/_arch/aarch64/exception.rs
@@ -11,7 +11,7 @@
 //!
 //! crate::exception::arch_exception

-use crate::exception;
+use crate::{exception, memory, symbols};
 use aarch64_cpu::{asm::barrier, registers::*};
 use core::{arch::global_asm, cell::UnsafeCell, fmt};
 use tock_registers::{
@@ -260,6 +260,14 @@

         writeln!(f, "{}", self.spsr_el1)?;
         writeln!(f, "ELR_EL1: {:#018x}", self.elr_el1)?;
+        writeln!(
+            f,
+            "      Symbol: {}",
+            match symbols::lookup_symbol(memory::Address::new(self.elr_el1 as usize)) {
+                Some(sym) => sym.name(),
+                _ => "Symbol not found",
+            }
+        )?;
         writeln!(f)?;
         writeln!(f, "General purpose register:")?;


diff -uNr 16_virtual_mem_part4_higher_half_kernel/kernel/src/bsp/raspberrypi/kernel.ld 17_kernel_symbols/kernel/src/bsp/raspberrypi/kernel.ld
--- 16_virtual_mem_part4_higher_half_kernel/kernel/src/bsp/raspberrypi/kernel.ld
+++ 17_kernel_symbols/kernel/src/bsp/raspberrypi/kernel.ld
@@ -56,7 +56,11 @@
         *(.text*)                 /* Everything else */
     } :segment_code

-    .rodata : ALIGN(8) { *(.rodata*) } :segment_code
+    .rodata         : ALIGN(8) { *(.rodata*) } :segment_code
+    .kernel_symbols : ALIGN(8) {
+        __kernel_symbols_start = .;
+        . += 32 * 1024;
+    } :segment_code

     . = ALIGN(PAGE_SIZE);
     __code_end_exclusive = .;

diff -uNr 16_virtual_mem_part4_higher_half_kernel/kernel/src/bsp/raspberrypi/memory.rs 17_kernel_symbols/kernel/src/bsp/raspberrypi/memory.rs
--- 16_virtual_mem_part4_higher_half_kernel/kernel/src/bsp/raspberrypi/memory.rs
+++ 17_kernel_symbols/kernel/src/bsp/raspberrypi/memory.rs
@@ -20,6 +20,7 @@
 //! | .text                                 |
 //! | .rodata                               |
 //! | .got                                  |
+//! | .kernel_symbols                       |
 //! |                                       |
 //! +---------------------------------------+
 //! |                                       | data_start == code_end_exclusive
@@ -41,6 +42,7 @@
 //! | .text                                 |
 //! | .rodata                               |
 //! | .got                                  |
+//! | .kernel_symbols                       |
 //! |                                       |
 //! +---------------------------------------+
 //! |                                       | data_start == code_end_exclusive

diff -uNr 16_virtual_mem_part4_higher_half_kernel/kernel/src/lib.rs 17_kernel_symbols/kernel/src/lib.rs
--- 16_virtual_mem_part4_higher_half_kernel/kernel/src/lib.rs
+++ 17_kernel_symbols/kernel/src/lib.rs
@@ -142,6 +142,7 @@
 pub mod memory;
 pub mod print;
 pub mod state;
+pub mod symbols;
 pub mod time;

 //--------------------------------------------------------------------------------------------------

diff -uNr 16_virtual_mem_part4_higher_half_kernel/kernel/src/symbols.rs 17_kernel_symbols/kernel/src/symbols.rs
--- 16_virtual_mem_part4_higher_half_kernel/kernel/src/symbols.rs
+++ 17_kernel_symbols/kernel/src/symbols.rs
@@ -0,0 +1,88 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2022-2023 Andre Richter <andre.o.richter@gmail.com>
+
+//! Debug symbol support.
+
+use crate::memory::{Address, Virtual};
+use core::{cell::UnsafeCell, slice};
+use debug_symbol_types::Symbol;
+
+//--------------------------------------------------------------------------------------------------
+// Private Definitions
+//--------------------------------------------------------------------------------------------------
+
+// Symbol from the linker script.
+extern "Rust" {
+    static __kernel_symbols_start: UnsafeCell<()>;
+}
+
+//--------------------------------------------------------------------------------------------------
+// Global instances
+//--------------------------------------------------------------------------------------------------
+
+/// This will be patched to the correct value by the "kernel symbols tool" after linking. This given
+/// value here is just a (safe) dummy.
+#[no_mangle]
+static NUM_KERNEL_SYMBOLS: u64 = 0;
+
+//--------------------------------------------------------------------------------------------------
+// Private Code
+//--------------------------------------------------------------------------------------------------
+
+fn kernel_symbol_section_virt_start_addr() -> Address<Virtual> {
+    Address::new(unsafe { __kernel_symbols_start.get() as usize })
+}
+
+fn num_kernel_symbols() -> usize {
+    unsafe {
+        // Read volatile is needed here to prevent the compiler from optimizing NUM_KERNEL_SYMBOLS
+        // away.
+        core::ptr::read_volatile(&NUM_KERNEL_SYMBOLS as *const u64) as usize
+    }
+}
+
+fn kernel_symbols_slice() -> &'static [Symbol] {
+    let ptr = kernel_symbol_section_virt_start_addr().as_usize() as *const Symbol;
+
+    unsafe { slice::from_raw_parts(ptr, num_kernel_symbols()) }
+}
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+/// Retrieve the symbol corresponding to a virtual address, if any.
+pub fn lookup_symbol(addr: Address<Virtual>) -> Option<&'static Symbol> {
+    kernel_symbols_slice()
+        .iter()
+        .find(|&i| i.contains(addr.as_usize()))
+}
+
+//--------------------------------------------------------------------------------------------------
+// Testing
+//--------------------------------------------------------------------------------------------------
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use test_macros::kernel_test;
+
+    /// Sanity of symbols module.
+    #[kernel_test]
+    fn symbols_sanity() {
+        let first_sym = lookup_symbol(Address::new(
+            crate::common::is_aligned as *const usize as usize,
+        ))
+        .unwrap()
+        .name();
+
+        assert_eq!(first_sym, "libkernel::common::is_aligned");
+
+        let second_sym = lookup_symbol(Address::new(crate::version as *const usize as usize))
+            .unwrap()
+            .name();
+
+        assert_eq!(second_sym, "libkernel::version");
+    }
+}

diff -uNr 16_virtual_mem_part4_higher_half_kernel/kernel_symbols/build.rs 17_kernel_symbols/kernel_symbols/build.rs
--- 16_virtual_mem_part4_higher_half_kernel/kernel_symbols/build.rs
+++ 17_kernel_symbols/kernel_symbols/build.rs
@@ -0,0 +1,14 @@
+use std::{env, path::Path};
+
+fn main() {
+    if let Ok(path) = env::var("KERNEL_SYMBOLS_DEMANGLED_RS") {
+        if Path::new(&path).exists() {
+            println!("cargo:rustc-cfg=feature=\"generated_symbols_available\"")
+        }
+    }
+
+    println!(
+        "cargo:rerun-if-changed={}",
+        Path::new("kernel_symbols.ld").display()
+    );
+}

diff -uNr 16_virtual_mem_part4_higher_half_kernel/kernel_symbols/Cargo.toml 17_kernel_symbols/kernel_symbols/Cargo.toml
--- 16_virtual_mem_part4_higher_half_kernel/kernel_symbols/Cargo.toml
+++ 17_kernel_symbols/kernel_symbols/Cargo.toml
@@ -0,0 +1,15 @@
+[package]
+name = "kernel_symbols"
+version = "0.1.0"
+edition = "2021"
+
+[features]
+default = []
+generated_symbols_available = []
+
+##--------------------------------------------------------------------------------------------------
+## Dependencies
+##--------------------------------------------------------------------------------------------------
+
+[dependencies]
+debug-symbol-types = { path = "../libraries/debug-symbol-types" }

diff -uNr 16_virtual_mem_part4_higher_half_kernel/kernel_symbols/kernel_symbols.ld 17_kernel_symbols/kernel_symbols/kernel_symbols.ld
--- 16_virtual_mem_part4_higher_half_kernel/kernel_symbols/kernel_symbols.ld
+++ 17_kernel_symbols/kernel_symbols/kernel_symbols.ld
@@ -0,0 +1,15 @@
+/* SPDX-License-Identifier: MIT OR Apache-2.0
+ *
+ * Copyright (c) 2022-2023 Andre Richter <andre.o.richter@gmail.com>
+ */
+
+SECTIONS
+{
+    .rodata : {
+        ASSERT(. > 0xffffffff00000000, "Expected higher half address")
+
+        KEEP(*(.rodata.symbol_desc*))
+        . = ALIGN(8);
+        *(.rodata*)
+    }
+}

diff -uNr 16_virtual_mem_part4_higher_half_kernel/kernel_symbols/src/main.rs 17_kernel_symbols/kernel_symbols/src/main.rs
--- 16_virtual_mem_part4_higher_half_kernel/kernel_symbols/src/main.rs
+++ 17_kernel_symbols/kernel_symbols/src/main.rs
@@ -0,0 +1,16 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2022-2023 Andre Richter <andre.o.richter@gmail.com>
+
+//! Generation of kernel symbols.
+
+#![no_std]
+#![no_main]
+
+#[cfg(feature = "generated_symbols_available")]
+include!(env!("KERNEL_SYMBOLS_DEMANGLED_RS"));
+
+#[panic_handler]
+fn panic(_info: &core::panic::PanicInfo) -> ! {
+    unimplemented!()
+}

diff -uNr 16_virtual_mem_part4_higher_half_kernel/kernel_symbols.mk 17_kernel_symbols/kernel_symbols.mk
--- 16_virtual_mem_part4_higher_half_kernel/kernel_symbols.mk
+++ 17_kernel_symbols/kernel_symbols.mk
@@ -0,0 +1,117 @@
+## SPDX-License-Identifier: MIT OR Apache-2.0
+##
+## Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
+
+include ../common/format.mk
+include ../common/docker.mk
+
+##--------------------------------------------------------------------------------------------------
+## Check for input variables that need be exported by the calling Makefile
+##--------------------------------------------------------------------------------------------------
+ifndef KERNEL_SYMBOLS_TOOL_PATH
+$(error KERNEL_SYMBOLS_TOOL_PATH is not set)
+endif
+
+ifndef TARGET
+$(error TARGET is not set)
+endif
+
+ifndef KERNEL_SYMBOLS_INPUT_ELF
+$(error KERNEL_SYMBOLS_INPUT_ELF is not set)
+endif
+
+ifndef KERNEL_SYMBOLS_OUTPUT_ELF
+$(error KERNEL_SYMBOLS_OUTPUT_ELF is not set)
+endif
+
+
+
+##--------------------------------------------------------------------------------------------------
+## Targets and Prerequisites
+##--------------------------------------------------------------------------------------------------
+KERNEL_SYMBOLS_MANIFEST      = kernel_symbols/Cargo.toml
+KERNEL_SYMBOLS_LINKER_SCRIPT = kernel_symbols/kernel_symbols.ld
+
+KERNEL_SYMBOLS_RS           = $(KERNEL_SYMBOLS_INPUT_ELF)_symbols.rs
+KERNEL_SYMBOLS_DEMANGLED_RS = $(shell pwd)/$(KERNEL_SYMBOLS_INPUT_ELF)_symbols_demangled.rs
+
+KERNEL_SYMBOLS_ELF      = target/$(TARGET)/release/kernel_symbols
+KERNEL_SYMBOLS_STRIPPED = target/$(TARGET)/release/kernel_symbols_stripped
+
+# Export for build.rs of kernel_symbols crate.
+export KERNEL_SYMBOLS_DEMANGLED_RS
+
+
+
+##--------------------------------------------------------------------------------------------------
+## Command building blocks
+##--------------------------------------------------------------------------------------------------
+GET_SYMBOLS_SECTION_VIRT_ADDR = $(DOCKER_TOOLS) $(EXEC_SYMBOLS_TOOL) \
+    --get_symbols_section_virt_addr $(KERNEL_SYMBOLS_OUTPUT_ELF)
+
+RUSTFLAGS = -C link-arg=--script=$(KERNEL_SYMBOLS_LINKER_SCRIPT) \
+    -C link-arg=--section-start=.rodata=$$($(GET_SYMBOLS_SECTION_VIRT_ADDR))
+
+RUSTFLAGS_PEDANTIC = $(RUSTFLAGS) \
+    -D warnings                   \
+    -D missing_docs
+
+COMPILER_ARGS = --target=$(TARGET) \
+    --release
+
+RUSTC_CMD   = cargo rustc $(COMPILER_ARGS) --manifest-path $(KERNEL_SYMBOLS_MANIFEST)
+OBJCOPY_CMD = rust-objcopy \
+    --strip-all            \
+    -O binary
+
+EXEC_SYMBOLS_TOOL  = ruby $(KERNEL_SYMBOLS_TOOL_PATH)/main.rb
+
+##------------------------------------------------------------------------------
+## Dockerization
+##------------------------------------------------------------------------------
+DOCKER_CMD = docker run -t --rm -v $(shell pwd):/work/tutorial -w /work/tutorial
+
+# DOCKER_IMAGE defined in include file (see top of this file).
+DOCKER_TOOLS = $(DOCKER_CMD) $(DOCKER_IMAGE)
+
+
+
+##--------------------------------------------------------------------------------------------------
+## Targets
+##--------------------------------------------------------------------------------------------------
+.PHONY: all symbols measure_time_start measure_time_finish
+
+all: measure_time_start symbols measure_time_finish
+
+symbols:
+	@cp $(KERNEL_SYMBOLS_INPUT_ELF) $(KERNEL_SYMBOLS_OUTPUT_ELF)
+
+	@$(DOCKER_TOOLS) $(EXEC_SYMBOLS_TOOL) --gen_symbols $(KERNEL_SYMBOLS_OUTPUT_ELF) \
+                $(KERNEL_SYMBOLS_RS)
+
+	$(call color_progress_prefix, "Demangling")
+	@echo Symbol names
+	@cat $(KERNEL_SYMBOLS_RS) | rustfilt > $(KERNEL_SYMBOLS_DEMANGLED_RS)
+
+	@RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(RUSTC_CMD)
+
+	$(call color_progress_prefix, "Stripping")
+	@echo Symbols ELF file
+	@$(OBJCOPY_CMD) $(KERNEL_SYMBOLS_ELF) $(KERNEL_SYMBOLS_STRIPPED)
+
+	@$(DOCKER_TOOLS) $(EXEC_SYMBOLS_TOOL) --patch_data $(KERNEL_SYMBOLS_OUTPUT_ELF) \
+                $(KERNEL_SYMBOLS_STRIPPED)
+
+# Note: The following is the only _trivial_ way I could think of that works out of the box on both
+# Linux and macOS. Since macOS does not have the moduloN nanosecond format string option, the
+# resolution is restricted to whole seconds.
+measure_time_start:
+	@date +modulos > /tmp/kernel_symbols_start.date
+
+measure_time_finish:
+	@date +modulos > /tmp/kernel_symbols_end.date
+
+	$(call color_progress_prefix, "Finished")
+	@echo "in $$((`cat /tmp/kernel_symbols_end.date` - `cat /tmp/kernel_symbols_start.date`)).0s"
+
+	@rm /tmp/kernel_symbols_end.date /tmp/kernel_symbols_start.date

diff -uNr 16_virtual_mem_part4_higher_half_kernel/libraries/debug-symbol-types/Cargo.toml 17_kernel_symbols/libraries/debug-symbol-types/Cargo.toml
--- 16_virtual_mem_part4_higher_half_kernel/libraries/debug-symbol-types/Cargo.toml
+++ 17_kernel_symbols/libraries/debug-symbol-types/Cargo.toml
@@ -0,0 +1,4 @@
+[package]
+name = "debug-symbol-types"
+version = "0.1.0"
+edition = "2021"

diff -uNr 16_virtual_mem_part4_higher_half_kernel/libraries/debug-symbol-types/src/lib.rs 17_kernel_symbols/libraries/debug-symbol-types/src/lib.rs
--- 16_virtual_mem_part4_higher_half_kernel/libraries/debug-symbol-types/src/lib.rs
+++ 17_kernel_symbols/libraries/debug-symbol-types/src/lib.rs
@@ -0,0 +1,45 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2022-2023 Andre Richter <andre.o.richter@gmail.com>
+
+//! Types for implementing debug symbol support.
+
+#![no_std]
+
+use core::ops::Range;
+
+/// A symbol containing a size.
+#[repr(C)]
+#[derive(Clone)]
+pub struct Symbol {
+    addr_range: Range<usize>,
+    name: &'static str,
+}
+
+impl Symbol {
+    /// Create an instance.
+    pub const fn new(start: usize, size: usize, name: &'static str) -> Symbol {
+        Symbol {
+            addr_range: Range {
+                start,
+                end: start + size,
+            },
+            name,
+        }
+    }
+
+    /// Returns true if addr is contained in the range.
+    pub fn contains(&self, addr: usize) -> bool {
+        self.addr_range.contains(&addr)
+    }
+
+    /// Returns the symbol's name.
+    pub fn name(&self) -> &'static str {
+        self.name
+    }
+
+    /// Returns the symbol's size.
+    pub fn size(&self) -> usize {
+        self.addr_range.end - self.addr_range.start
+    }
+}

diff -uNr 16_virtual_mem_part4_higher_half_kernel/Makefile 17_kernel_symbols/Makefile
--- 16_virtual_mem_part4_higher_half_kernel/Makefile
+++ 17_kernel_symbols/Makefile
@@ -85,7 +85,24 @@
 KERNEL_ELF_TTABLES      = target/$(TARGET)/release/kernel+ttables
 KERNEL_ELF_TTABLES_DEPS = $(KERNEL_ELF_RAW) $(wildcard $(TT_TOOL_PATH)/*)

-KERNEL_ELF = $(KERNEL_ELF_TTABLES)
+##------------------------------------------------------------------------------
+## Kernel symbols
+##------------------------------------------------------------------------------
+export KERNEL_SYMBOLS_TOOL_PATH = tools/kernel_symbols_tool
+
+KERNEL_ELF_TTABLES_SYMS = target/$(TARGET)/release/kernel+ttables+symbols
+
+# Unlike with KERNEL_ELF_RAW, we are not relying on dep-info here. One of the reasons being that the
+# name of the generated symbols file varies between runs, which can cause confusion.
+KERNEL_ELF_TTABLES_SYMS_DEPS = $(KERNEL_ELF_TTABLES) \
+    $(wildcard kernel_symbols/*)                     \
+    $(wildcard $(KERNEL_SYMBOLS_TOOL_PATH)/*)
+
+export TARGET
+export KERNEL_SYMBOLS_INPUT_ELF  = $(KERNEL_ELF_TTABLES)
+export KERNEL_SYMBOLS_OUTPUT_ELF = $(KERNEL_ELF_TTABLES_SYMS)
+
+KERNEL_ELF = $(KERNEL_ELF_TTABLES_SYMS)



@@ -178,11 +195,18 @@
 	@$(DOCKER_TOOLS) $(EXEC_TT_TOOL) $(BSP) $(KERNEL_ELF_TTABLES)

 ##------------------------------------------------------------------------------
+## Generate kernel symbols and patch them into the kernel ELF
+##------------------------------------------------------------------------------
+$(KERNEL_ELF_TTABLES_SYMS): $(KERNEL_ELF_TTABLES_SYMS_DEPS)
+	$(call color_header, "Generating kernel symbols and patching kernel ELF")
+	@$(MAKE) --no-print-directory -f kernel_symbols.mk
+
+##------------------------------------------------------------------------------
 ## Generate the stripped kernel binary
 ##------------------------------------------------------------------------------
-$(KERNEL_BIN): $(KERNEL_ELF_TTABLES)
+$(KERNEL_BIN): $(KERNEL_ELF_TTABLES_SYMS)
 	$(call color_header, "Generating stripped binary")
-	@$(OBJCOPY_CMD) $(KERNEL_ELF_TTABLES) $(KERNEL_BIN)
+	@$(OBJCOPY_CMD) $(KERNEL_ELF_TTABLES_SYMS) $(KERNEL_BIN)
 	$(call color_progress_prefix, "Name")
 	@echo $(KERNEL_BIN)
 	$(call color_progress_prefix, "Size")
@@ -191,7 +215,7 @@
 ##------------------------------------------------------------------------------
 ## Generate the documentation
 ##------------------------------------------------------------------------------
-doc:
+doc: clean
 	$(call color_header, "Generating docs")
 	@$(DOC_CMD) --document-private-items --open

@@ -318,10 +342,19 @@
     cd $(shell pwd)

     TEST_ELF=$$(echo $$1 | sed -e 's/.*target/target/g')
+    TEST_ELF_SYMS="$${TEST_ELF}_syms"
     TEST_BINARY=$$(echo $$1.img | sed -e 's/.*target/target/g')

     $(DOCKER_TOOLS) $(EXEC_TT_TOOL) $(BSP) $$TEST_ELF > /dev/null
-    $(OBJCOPY_CMD) $$TEST_ELF $$TEST_BINARY
+
+    # This overrides the two ENV variables. The other ENV variables that are required as input for
+    # the .mk file are set already because they are exported by this Makefile and this script is
+    # started by the same.
+    KERNEL_SYMBOLS_INPUT_ELF=$$TEST_ELF           \
+        KERNEL_SYMBOLS_OUTPUT_ELF=$$TEST_ELF_SYMS \
+        $(MAKE) --no-print-directory -f kernel_symbols.mk > /dev/null 2>&1
+
+    $(OBJCOPY_CMD) $$TEST_ELF_SYMS $$TEST_BINARY
     $(DOCKER_TEST) $(EXEC_TEST_DISPATCH) $(EXEC_QEMU) $(QEMU_TEST_ARGS) -kernel $$TEST_BINARY
 endef


diff -uNr 16_virtual_mem_part4_higher_half_kernel/tools/kernel_symbols_tool/cmds.rb 17_kernel_symbols/tools/kernel_symbols_tool/cmds.rb
--- 16_virtual_mem_part4_higher_half_kernel/tools/kernel_symbols_tool/cmds.rb
+++ 17_kernel_symbols/tools/kernel_symbols_tool/cmds.rb
@@ -0,0 +1,45 @@
+# frozen_string_literal: true
+
+# SPDX-License-Identifier: MIT OR Apache-2.0
+#
+# Copyright (c) 2022-2023 Andre Richter <andre.o.richter@gmail.com>
+
+def generate_symbols(kernel_elf, output_file)
+    File.open(output_file, 'w') do |file|
+        header = <<~HEREDOC
+            use debug_symbol_types::Symbol;
+
+            # [no_mangle]
+            # [link_section = ".rodata.symbol_desc"]
+            static KERNEL_SYMBOLS: [Symbol; #{kernel_elf.num_symbols}] = [
+        HEREDOC
+
+        file.write(header)
+        kernel_elf.symbols.each do |sym|
+            value = sym.header.st_value
+            size = sym.header.st_size
+            name = sym.name
+
+            file.write("    Symbol::new(#{value}, #{size}, \"#{name}\"),\n")
+        end
+        file.write("];\n")
+    end
+end
+
+def get_symbols_section_virt_addr(kernel_elf)
+    kernel_elf.kernel_symbols_section_virt_addr
+end
+
+def patch_symbol_data(kernel_elf, symbols_blob_path)
+    symbols_blob = File.binread(symbols_blob_path)
+
+    raise if symbols_blob.size > kernel_elf.kernel_symbols_section_size
+
+    File.binwrite(kernel_elf.path, File.binread(symbols_blob_path),
+                  kernel_elf.kernel_symbols_section_offset_in_file)
+end
+
+def patch_num_symbols(kernel_elf)
+    num_packed = [kernel_elf.num_symbols].pack('Q<*') # "Q" == uint64_t, "<" == little endian
+    File.binwrite(kernel_elf.path, num_packed, kernel_elf.num_kernel_symbols_offset_in_file)
+end

diff -uNr 16_virtual_mem_part4_higher_half_kernel/tools/kernel_symbols_tool/kernel_elf.rb 17_kernel_symbols/tools/kernel_symbols_tool/kernel_elf.rb
--- 16_virtual_mem_part4_higher_half_kernel/tools/kernel_symbols_tool/kernel_elf.rb
+++ 17_kernel_symbols/tools/kernel_symbols_tool/kernel_elf.rb
@@ -0,0 +1,74 @@
+# frozen_string_literal: true
+
+# SPDX-License-Identifier: MIT OR Apache-2.0
+#
+# Copyright (c) 2021-2023 Andre Richter <andre.o.richter@gmail.com>
+
+# KernelELF
+class KernelELF
+    attr_reader :path
+
+    def initialize(kernel_elf_path, kernel_symbols_section, num_kernel_symbols)
+        @elf = ELFTools::ELFFile.new(File.open(kernel_elf_path))
+        @symtab_section = @elf.section_by_name('.symtab')
+
+        @path = kernel_elf_path
+        fetch_values(kernel_symbols_section, num_kernel_symbols)
+    end
+
+    private
+
+    def fetch_values(kernel_symbols_section, num_kernel_symbols)
+        sym = @symtab_section.symbol_by_name(num_kernel_symbols)
+        raise "Symbol \"#{num_kernel_symbols}\" not found" if sym.nil?
+
+        @num_kernel_symbols = sym
+
+        section = @elf.section_by_name(kernel_symbols_section)
+        raise "Section \"#{kernel_symbols_section}\" not found" if section.nil?
+
+        @kernel_symbols_section = section
+    end
+
+    def num_kernel_symbols_virt_addr
+        @num_kernel_symbols.header.st_value
+    end
+
+    def segment_containing_virt_addr(virt_addr)
+        @elf.each_segments do |segment|
+            return segment if segment.vma_in?(virt_addr)
+        end
+    end
+
+    def virt_addr_to_file_offset(virt_addr)
+        segment = segment_containing_virt_addr(virt_addr)
+        segment.vma_to_offset(virt_addr)
+    end
+
+    public
+
+    def symbols
+        non_zero_symbols = @symtab_section.symbols.reject { |sym| sym.header.st_size.zero? }
+        non_zero_symbols.sort_by { |sym| sym.header.st_value }
+    end
+
+    def num_symbols
+        symbols.size
+    end
+
+    def kernel_symbols_section_virt_addr
+        @kernel_symbols_section.header.sh_addr.to_i
+    end
+
+    def kernel_symbols_section_size
+        @kernel_symbols_section.header.sh_size.to_i
+    end
+
+    def kernel_symbols_section_offset_in_file
+        virt_addr_to_file_offset(kernel_symbols_section_virt_addr)
+    end
+
+    def num_kernel_symbols_offset_in_file
+        virt_addr_to_file_offset(num_kernel_symbols_virt_addr)
+    end
+end

diff -uNr 16_virtual_mem_part4_higher_half_kernel/tools/kernel_symbols_tool/main.rb 17_kernel_symbols/tools/kernel_symbols_tool/main.rb
--- 16_virtual_mem_part4_higher_half_kernel/tools/kernel_symbols_tool/main.rb
+++ 17_kernel_symbols/tools/kernel_symbols_tool/main.rb
@@ -0,0 +1,47 @@
+#!/usr/bin/env ruby
+# frozen_string_literal: true
+
+# SPDX-License-Identifier: MIT OR Apache-2.0
+#
+# Copyright (c) 2022-2023 Andre Richter <andre.o.richter@gmail.com>
+
+require 'rubygems'
+require 'bundler/setup'
+require 'colorize'
+require 'elftools'
+
+require_relative 'kernel_elf'
+require_relative 'cmds'
+
+KERNEL_SYMBOLS_SECTION = '.kernel_symbols'
+NUM_KERNEL_SYMBOLS = 'NUM_KERNEL_SYMBOLS'
+
+cmd = ARGV[0]
+
+kernel_elf_path = ARGV[1]
+kernel_elf = KernelELF.new(kernel_elf_path, KERNEL_SYMBOLS_SECTION, NUM_KERNEL_SYMBOLS)
+
+case cmd
+when '--gen_symbols'
+    output_file = ARGV[2]
+
+    print 'Generating'.rjust(12).green.bold
+    puts ' Symbols source file'
+
+    generate_symbols(kernel_elf, output_file)
+when '--get_symbols_section_virt_addr'
+    addr = get_symbols_section_virt_addr(kernel_elf)
+
+    puts "0x#{addr.to_s(16)}"
+when '--patch_data'
+    symbols_blob_path = ARGV[2]
+    num_symbols = kernel_elf.num_symbols
+
+    print 'Patching'.rjust(12).green.bold
+    puts " Symbols blob and number of symbols (#{num_symbols}) into ELF"
+
+    patch_symbol_data(kernel_elf, symbols_blob_path)
+    patch_num_symbols(kernel_elf)
+else
+    raise
+end

```
