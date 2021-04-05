# Tutorial 16 - Virtual Memory Part 4: Higher-Half Kernel

## tl;dr

- The time has come: We map and run the kernel from the top of the 64 bit virtual address space! 🥳

## Table of Contents

- [Introduction](#introduction)
- [Implementation](#implementation)
  - [Position-Independent Boot Code](#position-independent-boot-code)
- [Test it](#test-it)
- [Diff to previous](#diff-to-previous)

## Introduction

A long time in the making, in this tutorial we finally map the kernel to the most significant area
(alternatively: higher-half) of the 64 bit virtual address space. This makes room for future
applications to use the whole of the least significant area of the virtual memory space.

As has been teased since `tutorial 14`, we will make use of the `AArch64`'s `TTBR1`. Since the
kernel's virtual address space size is `2 GiB` since the last tutorial, `TTBR1` will cover the range
from `0xffff_ffff_ffff_ffff` down to `ffff_ffff_8000_0000` (both inclusive).

## Implementation

In `src/memory/mmu.rs`, we extend the `AssociatedTranslationTable` trait with a `TableStartFromTop`
associated type:

```rust
/// Intended to be implemented for [`AddressSpace`].
pub trait AssociatedTranslationTable {
    /// A translation table whose address range is:
    ///
    /// [u64::MAX, (u64::MAX - AS_SIZE) + 1]
    type TableStartFromTop;

    /// A translation table whose address range is:
    ///
    /// [AS_SIZE - 1, 0]
    type TableStartFromBottom;
}
```

Architecture specific code in `_arch/aarch64/memory/mmu/translation_table.rs` populates both types
now by making use of a new generic that is added to `FixedSizeTranslationTable`, which defines
whether the covered address space starts from the top or the bottom:

```rust
pub struct FixedSizeTranslationTable<const NUM_TABLES: usize, const START_FROM_TOP: bool> {
    ...
```

```rust
impl<const AS_SIZE: usize> memory::mmu::AssociatedTranslationTable
    for memory::mmu::AddressSpace<AS_SIZE>
where
    [u8; Self::SIZE >> Granule512MiB::SHIFT]: Sized,
{
    type TableStartFromTop =
        FixedSizeTranslationTable<{ Self::SIZE >> Granule512MiB::SHIFT }, true>;

    type TableStartFromBottom =
        FixedSizeTranslationTable<{ Self::SIZE >> Granule512MiB::SHIFT }, false>;
}
```

Thanks to this infrastructure, `BSP` Rust code in `bsp/raspberrypi/memory/mmu.rs` only needs to
change to this newly introduced type in order to switch from lower half to higher half translation
tables for the kernel:

```rust
type KernelTranslationTable =
    <KernelVirtAddrSpace as AssociatedTranslationTable>::TableStartFromTop;
```

In the `link.ld` linker script, we define a new symbol `__kernel_virt_start_addr` now, which is the
start address of the kernel's virtual address space, calculated as `(u64::MAX -
__kernel_virt_addr_space_size) + 1`. In order to make virtual-to-physical address translation easier
for the human eye (and mind), we link the kernel itself at `__kernel_virt_start_addr +
__rpi_load_addr`.

Before these tutorials, the first mapped address of the kernel binary was always located at
`__rpi_load_addr == 0x8_0000`. Starting with this tutorial, due to the `2 GiB` virtual address space
size, the new first mapped address is `ffff_ffff_8008_0000`. So by ignoring the upper bits of the
address, you can easily derive the physical address.

The changes in the `_arch` `MMU` driver are minimal, and mostly concerned with configuring `TCR_EL1`
for use with `TTBR1_EL1` now. And of course, setting `TTBR1_EL1` in `fn
enable_mmu_and_caching(...)`.

### Position-Independent Boot Code

Remember all the fuss that we made about `position-independent code` that will be needed until the
`MMU` is enabled. Let's quickly check what it means for us in reality now:

In `_arch/aarch64/cpu/boot.rs`, we turn on the `MMU` just before returning from `EL2` to `EL1`. So
by the time the CPU enters `EL1`, virtual memory will be active, and the CPU must therefore use the
new higher-half `virtual addresses` for everything it does.

Specifically, this means the address from which the CPU should execute upon entering `EL1` (function
`runtime_init()`) must be a valid _virtual address_, same as the stack pointer's address. Both of
them are programmed in function `fn prepare_el2_to_el1_transition(...)`, so we must ensure now that
_link-time_ addresses are used here. For this reason, retrieval of these addresses happens in
`assembly` in `boot.s`, where we can explicitly enforce generation of **absolute** addresses:

```asm
// Load the _absolute_ addresses of the following symbols. Since the kernel is linked at
// the top of the 64 bit address space, these are effectively virtual addresses.
ADR_ABS	x1, __boot_core_stack_end_exclusive
ADR_ABS	x2, runtime_init
```

Both values are forwarded to the Rust entry point function `_start_rust()`, which in turn forwards
them to `fn prepare_el2_to_el1_transition(...)`.

One more thing to consider is that we keep on programming the boot core's stack address for `EL2`
using an address that is calculated `PC-relative`, because all the `EL2` code will still run while
virtual memory _is disabled_. As such, we need the "physical" address of the stack, so to speak.

The previous tutorial also explained that it is not easily possible to compile select files using
`-fpic` in `Rust`. Still, we are doing some function calls in `Rust` before virtual memory is
enabled, so _theoretically_, there is room for failure. However, branches to local code in `AArch64`
are usually generated PC-relative. So it is a small risk worth taking. Should it still fail someday,
at least our automated CI pipeline would give notice when the tests start to fail.

## Test it

That's it! We are ready for a higher-half kernel now. Power up your Raspberrys and marvel at those
beautiful (virtual) addresses:

Raspberry Pi 3:

```console
$ make chainboot
[...]

Precomputing kernel translation tables and patching kernel ELF
             --------------------------------------------------
                 Section           Start Virt Addr       Size
             --------------------------------------------------
  Generating Code and RO data | 0xffff_ffff_8008_0000 |  64 KiB
  Generating Data and bss     | 0xffff_ffff_8009_0000 | 384 KiB
  Generating Boot-core stack  | 0xffff_ffff_8010_0000 | 512 KiB
             --------------------------------------------------
    Patching Kernel table struct at physical 0x9_0000
    Patching Value of kernel table physical base address (0xd_0000) at physical 0x8_0060
    Finished in 0.03s

Minipush 1.0

[MP] ⏳ Waiting for /dev/ttyUSB0
[MP] ✅ Serial connected
[MP] 🔌 Please power the target now
 __  __ _      _ _                 _
|  \/  (_)_ _ (_) |   ___  __ _ __| |
| |\/| | | ' \| | |__/ _ \/ _` / _` |
|_|  |_|_|_||_|_|____\___/\__,_\__,_|

           Raspberry Pi 3

[ML] Requesting binary
[MP] ⏩ Pushing 387 KiB =======================================🦀 100% 96 KiB/s Time: 00:00:04
[ML] Loaded! Executing the payload now

[    4.316420] mingo version 0.16.0
[    4.316627] Booting on: Raspberry Pi 3
[    4.317082] MMU online:
[    4.317375]       -------------------------------------------------------------------------------------------------------------------------------------------
[    4.319119]                         Virtual                                   Physical               Size       Attr                    Entity
[    4.320863]       -------------------------------------------------------------------------------------------------------------------------------------------
[    4.322610]       0xffff_ffff_8008_0000..0xffff_ffff_8008_ffff --> 0x00_0008_0000..0x00_0008_ffff |  64 KiB | C   RO X  | Kernel code and RO data
[    4.324223]       0xffff_ffff_8009_0000..0xffff_ffff_800e_ffff --> 0x00_0009_0000..0x00_000e_ffff | 384 KiB | C   RW XN | Kernel data and bss
[    4.325793]       0xffff_ffff_8010_0000..0xffff_ffff_8017_ffff --> 0x00_0010_0000..0x00_0017_ffff | 512 KiB | C   RW XN | Kernel boot-core stack
[    4.327397]       0xffff_ffff_f000_0000..0xffff_ffff_f000_ffff --> 0x00_3f20_0000..0x00_3f20_ffff |  64 KiB | Dev RW XN | BCM GPIO
[    4.328847]                                                                                                             | BCM PL011 UART
[    4.330365]       0xffff_ffff_f001_0000..0xffff_ffff_f001_ffff --> 0x00_3f00_0000..0x00_3f00_ffff |  64 KiB | Dev RW XN | BCM Peripheral Interrupt Controller
[    4.332108]       -------------------------------------------------------------------------------------------------------------------------------------------
```

Raspberry Pi 4:

```console
$ BSP=rpi4 make chainboot
[...]

Precomputing kernel translation tables and patching kernel ELF
             --------------------------------------------------
                 Section           Start Virt Addr       Size
             --------------------------------------------------
  Generating Code and RO data | 0xffff_ffff_8008_0000 |  64 KiB
  Generating Data and bss     | 0xffff_ffff_8009_0000 | 448 KiB
  Generating Boot-core stack  | 0xffff_ffff_8011_0000 | 512 KiB
             --------------------------------------------------
    Patching Kernel table struct at physical 0xa_0000
    Patching Value of kernel table physical base address (0xe_0000) at physical 0x8_0068
    Finished in 0.03s

Minipush 1.0

[MP] ⏳ Waiting for /dev/ttyUSB0
[MP] ✅ Serial connected
[MP] 🔌 Please power the target now
 __  __ _      _ _                 _
|  \/  (_)_ _ (_) |   ___  __ _ __| |
| |\/| | | ' \| | |__/ _ \/ _` / _` |
|_|  |_|_|_||_|_|____\___/\__,_\__,_|

           Raspberry Pi 4

[ML] Requesting binary
[MP] ⏩ Pushing 449 KiB ======================================🦀 100% 112 KiB/s Time: 00:00:04
[ML] Loaded! Executing the payload now

[    5.009551] mingo version 0.16.0
[    5.009585] Booting on: Raspberry Pi 4
[    5.010040] MMU online:
[    5.010332]       -------------------------------------------------------------------------------------------------------------------------------------------
[    5.012077]                         Virtual                                   Physical               Size       Attr                    Entity
[    5.013821]       -------------------------------------------------------------------------------------------------------------------------------------------
[    5.015566]       0xffff_ffff_8008_0000..0xffff_ffff_8008_ffff --> 0x00_0008_0000..0x00_0008_ffff |  64 KiB | C   RO X  | Kernel code and RO data
[    5.017181]       0xffff_ffff_8009_0000..0xffff_ffff_800f_ffff --> 0x00_0009_0000..0x00_000f_ffff | 448 KiB | C   RW XN | Kernel data and bss
[    5.018751]       0xffff_ffff_8011_0000..0xffff_ffff_8018_ffff --> 0x00_0011_0000..0x00_0018_ffff | 512 KiB | C   RW XN | Kernel boot-core stack
[    5.020354]       0xffff_ffff_f000_0000..0xffff_ffff_f000_ffff --> 0x00_fe20_0000..0x00_fe20_ffff |  64 KiB | Dev RW XN | BCM GPIO
[    5.021805]                                                                                                             | BCM PL011 UART
[    5.023322]       0xffff_ffff_f001_0000..0xffff_ffff_f001_ffff --> 0x00_ff84_0000..0x00_ff84_ffff |  64 KiB | Dev RW XN | GICD
[    5.024730]                                                                                                             | GICC
[    5.026138]       -------------------------------------------------------------------------------------------------------------------------------------------
```

## Diff to previous
```diff

diff -uNr 15_virtual_mem_part3_precomputed_tables/Cargo.toml 16_virtual_mem_part4_higher_half_kernel/Cargo.toml
--- 15_virtual_mem_part3_precomputed_tables/Cargo.toml
+++ 16_virtual_mem_part4_higher_half_kernel/Cargo.toml
@@ -1,6 +1,6 @@
 [package]
 name = "mingo"
-version = "0.15.0"
+version = "0.16.0"
 authors = ["Andre Richter <andre.o.richter@gmail.com>"]
 edition = "2018"


diff -uNr 15_virtual_mem_part3_precomputed_tables/src/_arch/aarch64/cpu/boot.rs 16_virtual_mem_part4_higher_half_kernel/src/_arch/aarch64/cpu/boot.rs
--- 15_virtual_mem_part3_precomputed_tables/src/_arch/aarch64/cpu/boot.rs
+++ 16_virtual_mem_part4_higher_half_kernel/src/_arch/aarch64/cpu/boot.rs
@@ -11,7 +11,7 @@
 //!
 //! crate::cpu::boot::arch_boot

-use crate::{cpu, memory, memory::Address, runtime_init};
+use crate::{cpu, memory, memory::Address};
 use core::intrinsics::unlikely;
 use cortex_a::{asm, regs::*};

@@ -29,7 +29,10 @@
 /// - The `bss` section is not initialized yet. The code must not use or reference it in any way.
 /// - The HW state of EL1 must be prepared in a sound way.
 #[inline(always)]
-unsafe fn prepare_el2_to_el1_transition(phys_boot_core_stack_end_exclusive_addr: u64) {
+unsafe fn prepare_el2_to_el1_transition(
+    virt_boot_core_stack_end_exclusive_addr: u64,
+    virt_runtime_init_addr: u64,
+) {
     // Enable timer counter registers for EL1.
     CNTHCTL_EL2.write(CNTHCTL_EL2::EL1PCEN::SET + CNTHCTL_EL2::EL1PCTEN::SET);

@@ -52,11 +55,11 @@
     );

     // Second, let the link register point to runtime_init().
-    ELR_EL2.set(runtime_init::runtime_init as *const () as u64);
+    ELR_EL2.set(virt_runtime_init_addr);

     // Set up SP_EL1 (stack pointer), which will be used by EL1 once we "return" to it. Since there
     // are no plans to ever return to EL2, just re-use the same stack.
-    SP_EL1.set(phys_boot_core_stack_end_exclusive_addr);
+    SP_EL1.set(virt_boot_core_stack_end_exclusive_addr);
 }

 //--------------------------------------------------------------------------------------------------
@@ -74,9 +77,13 @@
 #[no_mangle]
 pub unsafe extern "C" fn _start_rust(
     phys_kernel_tables_base_addr: u64,
-    phys_boot_core_stack_end_exclusive_addr: u64,
+    virt_boot_core_stack_end_exclusive_addr: u64,
+    virt_runtime_init_addr: u64,
 ) -> ! {
-    prepare_el2_to_el1_transition(phys_boot_core_stack_end_exclusive_addr);
+    prepare_el2_to_el1_transition(
+        virt_boot_core_stack_end_exclusive_addr,
+        virt_runtime_init_addr,
+    );

     // Turn on the MMU for EL1.
     let addr = Address::new(phys_kernel_tables_base_addr as usize);
@@ -84,6 +91,7 @@
         cpu::wait_forever();
     }

-    // Use `eret` to "return" to EL1. This results in execution of runtime_init() in EL1.
+    // Use `eret` to "return" to EL1. Since virtual memory will already be enabled, this results in
+    // execution of runtime_init() in EL1 from its _virtual address_.
     asm::eret()
 }

diff -uNr 15_virtual_mem_part3_precomputed_tables/src/_arch/aarch64/cpu/boot.s 16_virtual_mem_part4_higher_half_kernel/src/_arch/aarch64/cpu/boot.s
--- 15_virtual_mem_part3_precomputed_tables/src/_arch/aarch64/cpu/boot.s
+++ 16_virtual_mem_part4_higher_half_kernel/src/_arch/aarch64/cpu/boot.s
@@ -18,6 +18,18 @@
 	add	\register, \register, #:lo12:\symbol
 .endm

+// Load the address of a symbol into a register, absolute.
+//
+// # Resources
+//
+// - https://sourceware.org/binutils/docs-2.36/as/AArch64_002dRelocations.html
+.macro ADR_ABS register, symbol
+	movz	\register, #:abs_g3:\symbol
+	movk	\register, #:abs_g2_nc:\symbol
+	movk	\register, #:abs_g1_nc:\symbol
+	movk	\register, #:abs_g0_nc:\symbol
+.endm
+
 .equ _EL2, 0x8
 .equ _core_id_mask, 0b11

@@ -47,11 +59,23 @@
 	// Load the base address of the kernel's translation tables.
 	ldr	x0, PHYS_KERNEL_TABLES_BASE_ADDR // provided by bsp/__board_name__/memory/mmu.rs

-	// Set the stack pointer. This ensures that any code in EL2 that needs the stack will work.
-	ADR_REL	x1, __boot_core_stack_end_exclusive
-	mov	sp, x1
+	// Load the _absolute_ addresses of the following symbols. Since the kernel is linked at
+	// the top of the 64 bit address space, these are effectively virtual addresses.
+	ADR_ABS	x1, __boot_core_stack_end_exclusive
+	ADR_ABS	x2, runtime_init
+
+	// Load the PC-relative address of the stack and set the stack pointer.
+	//
+	// Since _start() is the first function that runs after the firmware has loaded the kernel
+	// into memory, retrieving this symbol PC-relative returns the "physical" address.
+	//
+	// Setting the stack pointer to this value ensures that anything that still runs in EL2,
+	// until the kernel returns to EL1 with the MMU enabled, works as well. After the return to
+	// EL1, the virtual address of the stack retrieved above will be used.
+	ADR_REL	x4, __boot_core_stack_end_exclusive
+	mov	sp, x4

-	// Jump to Rust code. x0 and x1 hold the function arguments provided to _start_rust().
+	// Jump to Rust code. x0, x1 and x2 hold the function arguments provided to _start_rust().
 	b	_start_rust

 	// Infinitely wait for events (aka "park the core").

diff -uNr 15_virtual_mem_part3_precomputed_tables/src/_arch/aarch64/memory/mmu/translation_table.rs 16_virtual_mem_part4_higher_half_kernel/src/_arch/aarch64/memory/mmu/translation_table.rs
--- 15_virtual_mem_part3_precomputed_tables/src/_arch/aarch64/memory/mmu/translation_table.rs
+++ 16_virtual_mem_part4_higher_half_kernel/src/_arch/aarch64/memory/mmu/translation_table.rs
@@ -131,7 +131,7 @@
 /// aligned, so the lvl3 is put first.
 #[repr(C)]
 #[repr(align(65536))]
-pub struct FixedSizeTranslationTable<const NUM_TABLES: usize> {
+pub struct FixedSizeTranslationTable<const NUM_TABLES: usize, const START_FROM_TOP: bool> {
     /// Page descriptors, covering 64 KiB windows per entry.
     lvl3: [[PageDescriptor; 8192]; NUM_TABLES],

@@ -258,14 +258,23 @@
 where
     [u8; Self::SIZE >> Granule512MiB::SHIFT]: Sized,
 {
-    type TableStartFromBottom = FixedSizeTranslationTable<{ Self::SIZE >> Granule512MiB::SHIFT }>;
+    type TableStartFromTop =
+        FixedSizeTranslationTable<{ Self::SIZE >> Granule512MiB::SHIFT }, true>;
+
+    type TableStartFromBottom =
+        FixedSizeTranslationTable<{ Self::SIZE >> Granule512MiB::SHIFT }, false>;
 }

-impl<const NUM_TABLES: usize> FixedSizeTranslationTable<NUM_TABLES> {
+impl<const NUM_TABLES: usize, const START_FROM_TOP: bool>
+    FixedSizeTranslationTable<NUM_TABLES, START_FROM_TOP>
+{
     // Reserve the last 256 MiB of the address space for MMIO mappings.
     const L2_MMIO_START_INDEX: usize = NUM_TABLES - 1;
     const L3_MMIO_START_INDEX: usize = 8192 / 2;

+    const START_FROM_TOP_OFFSET: Address<Virtual> =
+        Address::new((usize::MAX - (Granule512MiB::SIZE * NUM_TABLES)) + 1);
+
     /// Create an instance.
     #[allow(clippy::assertions_on_constants)]
     const fn _new(for_precompute: bool) -> Self {
@@ -294,20 +303,32 @@
     /// The start address of the table's MMIO range.
     #[inline(always)]
     fn mmio_start_addr(&self) -> Address<Virtual> {
-        Address::new(
+        let mut addr = Address::new(
             (Self::L2_MMIO_START_INDEX << Granule512MiB::SHIFT)
                 | (Self::L3_MMIO_START_INDEX << Granule64KiB::SHIFT),
-        )
+        );
+
+        if START_FROM_TOP {
+            addr += Self::START_FROM_TOP_OFFSET;
+        }
+
+        addr
     }

     /// The inclusive end address of the table's MMIO range.
     #[inline(always)]
     fn mmio_end_addr_inclusive(&self) -> Address<Virtual> {
-        Address::new(
+        let mut addr = Address::new(
             (Self::L2_MMIO_START_INDEX << Granule512MiB::SHIFT)
                 | (8191 << Granule64KiB::SHIFT)
                 | (Granule64KiB::SIZE - 1),
-        )
+        );
+
+        if START_FROM_TOP {
+            addr += Self::START_FROM_TOP_OFFSET;
+        }
+
+        addr
     }

     /// Helper to calculate the lvl2 and lvl3 indices from an address.
@@ -316,7 +337,12 @@
         &self,
         addr: *const Page<Virtual>,
     ) -> Result<(usize, usize), &'static str> {
-        let addr = addr as usize;
+        let mut addr = addr as usize;
+
+        if START_FROM_TOP {
+            addr -= Self::START_FROM_TOP_OFFSET.into_usize()
+        }
+
         let lvl2_index = addr >> Granule512MiB::SHIFT;
         let lvl3_index = (addr & Granule512MiB::MASK) >> Granule64KiB::SHIFT;

@@ -343,8 +369,9 @@
 // OS Interface Code
 //------------------------------------------------------------------------------

-impl<const NUM_TABLES: usize> memory::mmu::translation_table::interface::TranslationTable
-    for FixedSizeTranslationTable<NUM_TABLES>
+impl<const NUM_TABLES: usize, const START_FROM_TOP: bool>
+    memory::mmu::translation_table::interface::TranslationTable
+    for FixedSizeTranslationTable<NUM_TABLES, START_FROM_TOP>
 {
     fn init(&mut self) -> Result<(), &'static str> {
         if self.initialized {
@@ -419,12 +446,16 @@
             return Err("Not enough MMIO space left");
         }

-        let addr = Address::new(
+        let mut addr = Address::new(
             (Self::L2_MMIO_START_INDEX << Granule512MiB::SHIFT)
                 | (self.cur_l3_mmio_index << Granule64KiB::SHIFT),
         );
         self.cur_l3_mmio_index += num_pages;

+        if START_FROM_TOP {
+            addr += Self::START_FROM_TOP_OFFSET;
+        }
+
         Ok(PageSliceDescriptor::from_addr(addr, num_pages))
     }

@@ -447,7 +478,7 @@
 //--------------------------------------------------------------------------------------------------

 #[cfg(test)]
-pub type MinSizeTranslationTable = FixedSizeTranslationTable<1>;
+pub type MinSizeTranslationTable = FixedSizeTranslationTable<1, false>;

 #[cfg(test)]
 mod tests {

diff -uNr 15_virtual_mem_part3_precomputed_tables/src/_arch/aarch64/memory/mmu.rs 16_virtual_mem_part4_higher_half_kernel/src/_arch/aarch64/memory/mmu.rs
--- 15_virtual_mem_part3_precomputed_tables/src/_arch/aarch64/memory/mmu.rs
+++ 16_virtual_mem_part4_higher_half_kernel/src/_arch/aarch64/memory/mmu.rs
@@ -65,6 +65,7 @@

 impl MemoryManagementUnit {
     /// Setup function for the MAIR_EL1 register.
+    #[inline(always)]
     fn set_up_mair(&self) {
         // Define the memory types being mapped.
         MAIR_EL1.write(
@@ -78,20 +79,21 @@
     }

     /// Configure various settings of stage 1 of the EL1 translation regime.
+    #[inline(always)]
     fn configure_translation_control(&self) {
-        let t0sz = (64 - bsp::memory::mmu::KernelVirtAddrSpace::SIZE_SHIFT) as u64;
+        let t1sz = (64 - bsp::memory::mmu::KernelVirtAddrSpace::SIZE_SHIFT) as u64;

         TCR_EL1.write(
-            TCR_EL1::TBI0::Used
+            TCR_EL1::TBI1::Used
                 + TCR_EL1::IPS::Bits_40
-                + TCR_EL1::TG0::KiB_64
-                + TCR_EL1::SH0::Inner
-                + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
-                + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
-                + TCR_EL1::EPD0::EnableTTBR0Walks
-                + TCR_EL1::A1::TTBR0
-                + TCR_EL1::T0SZ.val(t0sz)
-                + TCR_EL1::EPD1::DisableTTBR1Walks,
+                + TCR_EL1::TG1::KiB_64
+                + TCR_EL1::SH1::Inner
+                + TCR_EL1::ORGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
+                + TCR_EL1::IRGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
+                + TCR_EL1::EPD1::EnableTTBR1Walks
+                + TCR_EL1::A1::TTBR1
+                + TCR_EL1::T1SZ.val(t1sz)
+                + TCR_EL1::EPD0::DisableTTBR0Walks,
         );
     }
 }
@@ -130,7 +132,7 @@
         self.set_up_mair();

         // Set the "Translation Table Base Register".
-        TTBR0_EL1.set_baddr(phys_tables_base_addr.into_usize() as u64);
+        TTBR1_EL1.set_baddr(phys_tables_base_addr.into_usize() as u64);

         self.configure_translation_control();


diff -uNr 15_virtual_mem_part3_precomputed_tables/src/bsp/raspberrypi/link.ld 16_virtual_mem_part4_higher_half_kernel/src/bsp/raspberrypi/link.ld
--- 15_virtual_mem_part3_precomputed_tables/src/bsp/raspberrypi/link.ld
+++ 16_virtual_mem_part4_higher_half_kernel/src/bsp/raspberrypi/link.ld
@@ -6,6 +6,15 @@
 /* This file provides __kernel_virt_addr_space_size */
 INCLUDE src/bsp/raspberrypi/kernel_virt_addr_space_size.ld;

+/* The kernel's virtual address range will be:
+ *
+ * [END_ADDRESS_INCLUSIVE, START_ADDRESS]
+ * [u64::MAX             , (u64::MAX - __kernel_virt_addr_space_size) + 1]
+ *
+ * Since the start address is needed to set the linker address below, calculate it now.
+ */
+__kernel_virt_start_addr = ((0xffffffffffffffff - __kernel_virt_addr_space_size) + 1);
+
 /* The address at which the the kernel binary will be loaded by the Raspberry's firmware */
 __rpi_load_addr = 0x80000;

@@ -19,13 +28,14 @@

 SECTIONS
 {
-    . =  __rpi_load_addr;
+    /* Add the load address as an offset. Makes virt-to-phys translation easier for the human eye */
+    . =  __kernel_virt_start_addr + __rpi_load_addr;

     /***********************************************************************************************
     * Code + RO Data + Global Offset Table
     ***********************************************************************************************/
     __rx_start = .;
-    .text :
+    .text : AT(__rpi_load_addr)
     {
         KEEP(*(.text._start))
         *(.text._start_arguments) /* Constants (or statics in Rust speak) read by _start(). */

diff -uNr 15_virtual_mem_part3_precomputed_tables/src/bsp/raspberrypi/memory/mmu.rs 16_virtual_mem_part4_higher_half_kernel/src/bsp/raspberrypi/memory/mmu.rs
--- 15_virtual_mem_part3_precomputed_tables/src/bsp/raspberrypi/memory/mmu.rs
+++ 16_virtual_mem_part4_higher_half_kernel/src/bsp/raspberrypi/memory/mmu.rs
@@ -23,7 +23,7 @@
 //--------------------------------------------------------------------------------------------------

 type KernelTranslationTable =
-    <KernelVirtAddrSpace as AssociatedTranslationTable>::TableStartFromBottom;
+    <KernelVirtAddrSpace as AssociatedTranslationTable>::TableStartFromTop;

 //--------------------------------------------------------------------------------------------------
 // Public Definitions

diff -uNr 15_virtual_mem_part3_precomputed_tables/src/memory/mmu.rs 16_virtual_mem_part4_higher_half_kernel/src/memory/mmu.rs
--- 15_virtual_mem_part3_precomputed_tables/src/memory/mmu.rs
+++ 16_virtual_mem_part4_higher_half_kernel/src/memory/mmu.rs
@@ -80,6 +80,11 @@
 pub trait AssociatedTranslationTable {
     /// A translation table whose address range is:
     ///
+    /// [u64::MAX, (u64::MAX - AS_SIZE) + 1]
+    type TableStartFromTop;
+
+    /// A translation table whose address range is:
+    ///
     /// [AS_SIZE - 1, 0]
     type TableStartFromBottom;
 }

diff -uNr 15_virtual_mem_part3_precomputed_tables/src/runtime_init.rs 16_virtual_mem_part4_higher_half_kernel/src/runtime_init.rs
--- 15_virtual_mem_part3_precomputed_tables/src/runtime_init.rs
+++ 16_virtual_mem_part4_higher_half_kernel/src/runtime_init.rs
@@ -30,6 +30,7 @@
 /// # Safety
 ///
 /// - Only a single core must be active and running this function.
+#[no_mangle]
 pub unsafe fn runtime_init() -> ! {
     extern "Rust" {
         fn kernel_init() -> !;

diff -uNr 15_virtual_mem_part3_precomputed_tables/tests/02_exception_sync_page_fault.rs 16_virtual_mem_part4_higher_half_kernel/tests/02_exception_sync_page_fault.rs
--- 15_virtual_mem_part3_precomputed_tables/tests/02_exception_sync_page_fault.rs
+++ 16_virtual_mem_part4_higher_half_kernel/tests/02_exception_sync_page_fault.rs
@@ -27,8 +27,8 @@
     println!("Testing synchronous exception handling by causing a page fault");
     println!("-------------------------------------------------------------------\n");

-    println!("Writing beyond mapped area to address 9 GiB...");
-    let big_addr: u64 = 9 * 1024 * 1024 * 1024;
+    println!("Writing to bottom of address space to address 1 GiB...");
+    let big_addr: u64 = 1 * 1024 * 1024 * 1024;
     core::ptr::read_volatile(big_addr as *mut u64);

     // If execution reaches here, the memory access above did not cause a page fault exception.

diff -uNr 15_virtual_mem_part3_precomputed_tables/translation_table_tool/bsp.rb 16_virtual_mem_part4_higher_half_kernel/translation_table_tool/bsp.rb
--- 15_virtual_mem_part3_precomputed_tables/translation_table_tool/bsp.rb
+++ 16_virtual_mem_part4_higher_half_kernel/translation_table_tool/bsp.rb
@@ -31,7 +31,7 @@

         symbols = `#{NM_BINARY} --demangle #{kernel_elf}`.split("\n")
         @kernel_virt_addr_space_size = parse_from_symbols(symbols, /__kernel_virt_addr_space_size/)
-        @kernel_virt_start_addr = 0
+        @kernel_virt_start_addr = parse_from_symbols(symbols, /__kernel_virt_start_addr/)
         @virt_addresses = parse_from_symbols(symbols, @virt_addresses)
         @phys_addresses = virt_to_phys(@virt_addresses)

```
