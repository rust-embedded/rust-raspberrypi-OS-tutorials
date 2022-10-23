# Tutorial 16 - Virtual Memory Part 4: Higher-Half Kernel

## tl;dr

- The time has come: We map and run the kernel from the top of the 64 bit virtual address space! 🥳

## Table of Contents

- [Introduction](#introduction)
- [Implementation](#implementation)
  - [Linking Changes](#linking-changes)
  - [Position-Independent Boot Code](#position-independent-boot-code)
- [Test it](#test-it)
- [Diff to previous](#diff-to-previous)

## Introduction

A long time in the making, in this tutorial we finally map the kernel to the most significant area
(alternatively: higher-half) of the 64 bit virtual address space. This makes room for future
applications to use the whole of the least significant area of the virtual memory space.

As has been teased since `tutorial 14`, we will make use of the `AArch64`'s `TTBR1`. Since the
kernel's virtual address space size currently is `1 GiB` (defined in
`bsp/__board_name__/memory/mmu.rs`), `TTBR1` will cover the range from `0xffff_ffff_ffff_ffff` down
to `ffff_ffff_c000_0000` (both inclusive).

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

Thanks to this infrastructure, `BSP` Rust code in `bsp/__board_name__/memory/mmu.rs` only needs to
change to this newly introduced type in order to switch from lower half to higher half translation
tables for the kernel:

```rust
type KernelTranslationTable =
    <KernelVirtAddrSpace as AssociatedTranslationTable>::TableStartFromTop;
```

### Linking Changes

In the `kernel.ld` linker script, we define a new symbol `__kernel_virt_start_addr` now, which is
the start address of the kernel's virtual address space, calculated as `(u64::MAX -
__kernel_virt_addr_space_size) + 1`. Before the first section definition, we set the linker script's
location counter to this address:

```ld.s
SECTIONS
{
    . =  __kernel_virt_start_addr;

    ASSERT((. & PAGE_MASK) == 0, "Start of address space is not page aligned")

    /***********************************************************************************************
    * Code + RO Data + Global Offset Table
    ***********************************************************************************************/
```

Since we are not identity mapping anymore, we start to make use of the `AT` keyword in the output
section specification:

```ld.s
/* The physical address at which the the kernel binary will be loaded by the Raspberry's firmware */
__rpi_phys_binary_load_addr = 0x80000;

/* omitted */

SECTIONS
{
    . =  __kernel_virt_start_addr;

    /* omitted */

    __code_start = .;
    .text : AT(__rpi_phys_binary_load_addr)
```

This will manifest in the kernel ELF `segment` attributes, as can be inspected using the `make
readelf` command:

```console
$ make readelf

Program Headers:
  Type           Offset             VirtAddr           PhysAddr
                 FileSiz            MemSiz              Flags  Align
  LOAD           0x0000000000010000 0xffffffffc0000000 0x0000000000080000
                 0x000000000000cb08 0x000000000000cb08  R E    0x10000
  LOAD           0x0000000000020000 0xffffffffc0010000 0x0000000000090000
                 0x0000000000030dc0 0x0000000000030de0  RW     0x10000
  LOAD           0x0000000000060000 0xffffffffc0860000 0x0000000000000000
                 0x0000000000000000 0x0000000000080000  RW     0x10000

 Section to Segment mapping:
  Segment Sections...
   00     .text .rodata
   01     .data .bss
   02     .boot_core_stack

```

As you can see, `VirtAddr` and `PhysAddr` are different now, as compared to all the previous
tutorials where they were identical. This information from the `ELF` file will eventually be parsed
by the `translation table tool` and incorporated when compiling the precomputed translation tables.

You might have noticed that `.text .rodata` and `.boot_core_stack` exchanged places as compared to
previous tutorials. The reason this was done is that with a remapped kernel, this is trivial to do
without affecting the physical layout. This allows us to place an unmapped `guard page` between the
`boot core stack` and the `mmio remap region` in the VA space, which nicely protects the kernel from
stack overflows now:

```ld.s
/***********************************************************************************************
* MMIO Remap Reserved
***********************************************************************************************/
__mmio_remap_start = .;
. += 8 * 1024 * 1024;
__mmio_remap_end_exclusive = .;

ASSERT((. & PAGE_MASK) == 0, "MMIO remap reservation is not page aligned")

/***********************************************************************************************
* Guard Page
***********************************************************************************************/
. += PAGE_SIZE;

/***********************************************************************************************
* Boot Core Stack
***********************************************************************************************/
.boot_core_stack (NOLOAD) : AT(__rpi_phys_dram_start_addr)
{
    __boot_core_stack_start = .;         /*   ^             */
                                         /*   | stack       */
    . += __rpi_phys_binary_load_addr;    /*   | growth      */
                                         /*   | direction   */
    __boot_core_stack_end_exclusive = .; /*   |             */
} :segment_boot_core_stack

ASSERT((. & PAGE_MASK) == 0, "End of boot core stack is not page aligned")
```

Changes in the `_arch` `MMU` driver are minimal, and mostly concerned with configuring `TCR_EL1` for
use with `TTBR1_EL1` now. And of course, setting `TTBR1_EL1` in `fn enable_mmu_and_caching(...)`.

### Position-Independent Boot Code

Remember all the fuss that we made about `position-independent code` that will be needed until the
`MMU` is enabled. Let's quickly check what it means for us in reality now:

In `_arch/aarch64/cpu/boot.rs`, we turn on the `MMU` just before returning from `EL2` to `EL1`. So
by the time the CPU enters `EL1`, virtual memory will be active, and the CPU must therefore use the
new higher-half `virtual addresses` for everything it does.

Specifically, this means the address from which the CPU should execute upon entering `EL1` (function
`kernel_init()`) must be a valid _virtual address_, same as the stack pointer's address. Both of
them are programmed in function `fn prepare_el2_to_el1_transition(...)`, so we must ensure now that
_link-time_ addresses are used here. For this reason, retrieval of these addresses happens in
`assembly` in `boot.s`, where we can explicitly enforce generation of **absolute** addresses:

```asm
// Load the _absolute_ addresses of the following symbols. Since the kernel is linked at
// the top of the 64 bit address space, these are effectively virtual addresses.
ADR_ABS	x1, __boot_core_stack_end_exclusive
ADR_ABS	x2, kernel_init
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
             ------------------------------------------------------------------------------------
                 Sections          Virt Start Addr         Phys Start Addr       Size      Attr
             ------------------------------------------------------------------------------------
  Generating .text .rodata    | 0xffff_ffff_c000_0000 | 0x0000_0000_0008_0000 |  64 KiB | C RO X
  Generating .data .bss       | 0xffff_ffff_c001_0000 | 0x0000_0000_0009_0000 | 256 KiB | C RW XN
  Generating .boot_core_stack | 0xffff_ffff_c086_0000 | 0x0000_0000_0000_0000 | 512 KiB | C RW XN
             ------------------------------------------------------------------------------------
    Patching Kernel table struct at ELF file offset 0x2_0000
    Patching Kernel tables physical base address start argument to value 0xb_0000 at ELF file offset 0x1_0088
    Finished in 0.14s

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
[MP] ⏩ Pushing 257 KiB ======================================🦀 100% 128 KiB/s Time: 00:00:02
[ML] Loaded! Executing the payload now

[    2.870248] mingo version 0.16.0
[    2.870456] Booting on: Raspberry Pi 3
[    2.870911] MMU online:
[    2.871203]       -------------------------------------------------------------------------------------------------------------------------------------------
[    2.872947]                         Virtual                                   Physical               Size       Attr                    Entity
[    2.874691]       -------------------------------------------------------------------------------------------------------------------------------------------
[    2.876436]       0xffff_ffff_c000_0000..0xffff_ffff_c000_ffff --> 0x00_0008_0000..0x00_0008_ffff |  64 KiB | C   RO X  | Kernel code and RO data
[    2.878050]       0xffff_ffff_c001_0000..0xffff_ffff_c004_ffff --> 0x00_0009_0000..0x00_000c_ffff | 256 KiB | C   RW XN | Kernel data and bss
[    2.879621]       0xffff_ffff_c005_0000..0xffff_ffff_c005_ffff --> 0x00_3f20_0000..0x00_3f20_ffff |  64 KiB | Dev RW XN | BCM PL011 UART
[    2.881137]                                                                                                             | BCM GPIO
[    2.882589]       0xffff_ffff_c006_0000..0xffff_ffff_c006_ffff --> 0x00_3f00_0000..0x00_3f00_ffff |  64 KiB | Dev RW XN | BCM Interrupt Controller
[    2.884214]       0xffff_ffff_c086_0000..0xffff_ffff_c08d_ffff --> 0x00_0000_0000..0x00_0007_ffff | 512 KiB | C   RW XN | Kernel boot-core stack
[    2.885818]       -------------------------------------------------------------------------------------------------------------------------------------------
```

Raspberry Pi 4:

```console
$ BSP=rpi4 make chainboot
[...]

Precomputing kernel translation tables and patching kernel ELF
             ------------------------------------------------------------------------------------
                 Sections          Virt Start Addr         Phys Start Addr       Size      Attr
             ------------------------------------------------------------------------------------
  Generating .text .rodata    | 0xffff_ffff_c000_0000 | 0x0000_0000_0008_0000 |  64 KiB | C RO X
  Generating .data .bss       | 0xffff_ffff_c001_0000 | 0x0000_0000_0009_0000 | 256 KiB | C RW XN
  Generating .boot_core_stack | 0xffff_ffff_c086_0000 | 0x0000_0000_0000_0000 | 512 KiB | C RW XN
             ------------------------------------------------------------------------------------
    Patching Kernel table struct at ELF file offset 0x2_0000
    Patching Kernel tables physical base address start argument to value 0xb_0000 at ELF file offset 0x1_0080
    Finished in 0.13s

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
[MP] ⏩ Pushing 257 KiB ======================================🦀 100% 128 KiB/s Time: 00:00:02
[ML] Loaded! Executing the payload now

[    2.871960] mingo version 0.16.0
[    2.871994] Booting on: Raspberry Pi 4
[    2.872449] MMU online:
[    2.872742]       -------------------------------------------------------------------------------------------------------------------------------------------
[    2.874486]                         Virtual                                   Physical               Size       Attr                    Entity
[    2.876230]       -------------------------------------------------------------------------------------------------------------------------------------------
[    2.877975]       0xffff_ffff_c000_0000..0xffff_ffff_c000_ffff --> 0x00_0008_0000..0x00_0008_ffff |  64 KiB | C   RO X  | Kernel code and RO data
[    2.879589]       0xffff_ffff_c001_0000..0xffff_ffff_c004_ffff --> 0x00_0009_0000..0x00_000c_ffff | 256 KiB | C   RW XN | Kernel data and bss
[    2.881159]       0xffff_ffff_c005_0000..0xffff_ffff_c005_ffff --> 0x00_fe20_0000..0x00_fe20_ffff |  64 KiB | Dev RW XN | BCM PL011 UART
[    2.882676]                                                                                                             | BCM GPIO
[    2.884128]       0xffff_ffff_c006_0000..0xffff_ffff_c006_ffff --> 0x00_ff84_0000..0x00_ff84_ffff |  64 KiB | Dev RW XN | GICv2 GICD
[    2.885601]                                                                                                             | GICV2 GICC
[    2.887074]       0xffff_ffff_c086_0000..0xffff_ffff_c08d_ffff --> 0x00_0000_0000..0x00_0007_ffff | 512 KiB | C   RW XN | Kernel boot-core stack
[    2.888678]       -------------------------------------------------------------------------------------------------------------------------------------------
```

## Diff to previous
```diff

diff -uNr 15_virtual_mem_part3_precomputed_tables/kernel/Cargo.toml 16_virtual_mem_part4_higher_half_kernel/kernel/Cargo.toml
--- 15_virtual_mem_part3_precomputed_tables/kernel/Cargo.toml
+++ 16_virtual_mem_part4_higher_half_kernel/kernel/Cargo.toml
@@ -1,6 +1,6 @@
 [package]
 name = "mingo"
-version = "0.15.0"
+version = "0.16.0"
 authors = ["Andre Richter <andre.o.richter@gmail.com>"]
 edition = "2021"


diff -uNr 15_virtual_mem_part3_precomputed_tables/kernel/src/_arch/aarch64/cpu/boot.rs 16_virtual_mem_part4_higher_half_kernel/kernel/src/_arch/aarch64/cpu/boot.rs
--- 15_virtual_mem_part3_precomputed_tables/kernel/src/_arch/aarch64/cpu/boot.rs
+++ 16_virtual_mem_part4_higher_half_kernel/kernel/src/_arch/aarch64/cpu/boot.rs
@@ -34,7 +34,10 @@
 /// - The `bss` section is not initialized yet. The code must not use or reference it in any way.
 /// - The HW state of EL1 must be prepared in a sound way.
 #[inline(always)]
-unsafe fn prepare_el2_to_el1_transition(phys_boot_core_stack_end_exclusive_addr: u64) {
+unsafe fn prepare_el2_to_el1_transition(
+    virt_boot_core_stack_end_exclusive_addr: u64,
+    virt_kernel_init_addr: u64,
+) {
     // Enable timer counter registers for EL1.
     CNTHCTL_EL2.write(CNTHCTL_EL2::EL1PCEN::SET + CNTHCTL_EL2::EL1PCTEN::SET);

@@ -57,11 +60,11 @@
     );

     // Second, let the link register point to kernel_init().
-    ELR_EL2.set(crate::kernel_init as *const () as u64);
+    ELR_EL2.set(virt_kernel_init_addr);

     // Set up SP_EL1 (stack pointer), which will be used by EL1 once we "return" to it. Since there
     // are no plans to ever return to EL2, just re-use the same stack.
-    SP_EL1.set(phys_boot_core_stack_end_exclusive_addr);
+    SP_EL1.set(virt_boot_core_stack_end_exclusive_addr);
 }

 //--------------------------------------------------------------------------------------------------
@@ -78,14 +81,19 @@
 #[no_mangle]
 pub unsafe extern "C" fn _start_rust(
     phys_kernel_tables_base_addr: u64,
-    phys_boot_core_stack_end_exclusive_addr: u64,
+    virt_boot_core_stack_end_exclusive_addr: u64,
+    virt_kernel_init_addr: u64,
 ) -> ! {
-    prepare_el2_to_el1_transition(phys_boot_core_stack_end_exclusive_addr);
+    prepare_el2_to_el1_transition(
+        virt_boot_core_stack_end_exclusive_addr,
+        virt_kernel_init_addr,
+    );

     // Turn on the MMU for EL1.
     let addr = Address::new(phys_kernel_tables_base_addr as usize);
     memory::mmu::enable_mmu_and_caching(addr).unwrap();

-    // Use `eret` to "return" to EL1. This results in execution of kernel_init() in EL1.
+    // Use `eret` to "return" to EL1. Since virtual memory will already be enabled, this results in
+    // execution of kernel_init() in EL1 from its _virtual address_.
     asm::eret()
 }

diff -uNr 15_virtual_mem_part3_precomputed_tables/kernel/src/_arch/aarch64/cpu/boot.s 16_virtual_mem_part4_higher_half_kernel/kernel/src/_arch/aarch64/cpu/boot.s
--- 15_virtual_mem_part3_precomputed_tables/kernel/src/_arch/aarch64/cpu/boot.s
+++ 16_virtual_mem_part4_higher_half_kernel/kernel/src/_arch/aarch64/cpu/boot.s
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
 //--------------------------------------------------------------------------------------------------
 // Public Code
 //--------------------------------------------------------------------------------------------------
@@ -56,19 +68,31 @@
 	// Load the base address of the kernel's translation tables.
 	ldr	x0, PHYS_KERNEL_TABLES_BASE_ADDR // provided by bsp/__board_name__/memory/mmu.rs

-	// Set the stack pointer. This ensures that any code in EL2 that needs the stack will work.
-	ADR_REL	x1, __boot_core_stack_end_exclusive
-	mov	sp, x1
+	// Load the _absolute_ addresses of the following symbols. Since the kernel is linked at
+	// the top of the 64 bit address space, these are effectively virtual addresses.
+	ADR_ABS	x1, __boot_core_stack_end_exclusive
+	ADR_ABS	x2, kernel_init
+
+	// Load the PC-relative address of the stack and set the stack pointer.
+	//
+	// Since _start() is the first function that runs after the firmware has loaded the kernel
+	// into memory, retrieving this symbol PC-relative returns the "physical" address.
+	//
+	// Setting the stack pointer to this value ensures that anything that still runs in EL2,
+	// until the kernel returns to EL1 with the MMU enabled, works as well. After the return to
+	// EL1, the virtual address of the stack retrieved above will be used.
+	ADR_REL	x3, __boot_core_stack_end_exclusive
+	mov	sp, x3

 	// Read the CPU's timer counter frequency and store it in ARCH_TIMER_COUNTER_FREQUENCY.
 	// Abort if the frequency read back as 0.
-	ADR_REL	x2, ARCH_TIMER_COUNTER_FREQUENCY // provided by aarch64/time.rs
-	mrs	x3, CNTFRQ_EL0
-	cmp	x3, xzr
+	ADR_REL	x4, ARCH_TIMER_COUNTER_FREQUENCY // provided by aarch64/time.rs
+	mrs	x5, CNTFRQ_EL0
+	cmp	x5, xzr
 	b.eq	.L_parking_loop
-	str	w3, [x2]
+	str	w5, [x4]

-	// Jump to Rust code. x0 and x1 hold the function arguments provided to _start_rust().
+	// Jump to Rust code. x0, x1 and x2 hold the function arguments provided to _start_rust().
 	b	_start_rust

 	// Infinitely wait for events (aka "park the core").

diff -uNr 15_virtual_mem_part3_precomputed_tables/kernel/src/_arch/aarch64/memory/mmu/translation_table.rs 16_virtual_mem_part4_higher_half_kernel/kernel/src/_arch/aarch64/memory/mmu/translation_table.rs
--- 15_virtual_mem_part3_precomputed_tables/kernel/src/_arch/aarch64/memory/mmu/translation_table.rs
+++ 16_virtual_mem_part4_higher_half_kernel/kernel/src/_arch/aarch64/memory/mmu/translation_table.rs
@@ -136,7 +136,7 @@
 /// aligned, so the lvl3 is put first.
 #[repr(C)]
 #[repr(align(65536))]
-pub struct FixedSizeTranslationTable<const NUM_TABLES: usize> {
+pub struct FixedSizeTranslationTable<const NUM_TABLES: usize, const START_FROM_TOP: bool> {
     /// Page descriptors, covering 64 KiB windows per entry.
     lvl3: [[PageDescriptor; 8192]; NUM_TABLES],

@@ -302,10 +302,19 @@
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
+    const START_FROM_TOP_OFFSET: Address<Virtual> =
+        Address::new((usize::MAX - (Granule512MiB::SIZE * NUM_TABLES)) + 1);
+
     /// Create an instance.
     #[allow(clippy::assertions_on_constants)]
     const fn _new(for_precompute: bool) -> Self {
@@ -336,9 +345,14 @@
         &self,
         virt_page_addr: PageAddress<Virtual>,
     ) -> Result<(usize, usize), &'static str> {
-        let addr = virt_page_addr.into_inner().as_usize();
-        let lvl2_index = addr >> Granule512MiB::SHIFT;
-        let lvl3_index = (addr & Granule512MiB::MASK) >> Granule64KiB::SHIFT;
+        let mut addr = virt_page_addr.into_inner();
+
+        if START_FROM_TOP {
+            addr = addr - Self::START_FROM_TOP_OFFSET;
+        }
+
+        let lvl2_index = addr.as_usize() >> Granule512MiB::SHIFT;
+        let lvl3_index = (addr.as_usize() & Granule512MiB::MASK) >> Granule64KiB::SHIFT;

         if lvl2_index > (NUM_TABLES - 1) {
             return Err("Virtual page is out of bounds of translation table");
@@ -384,8 +398,9 @@
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
@@ -479,7 +494,7 @@
 //--------------------------------------------------------------------------------------------------

 #[cfg(test)]
-pub type MinSizeTranslationTable = FixedSizeTranslationTable<1>;
+pub type MinSizeTranslationTable = FixedSizeTranslationTable<1, true>;

 #[cfg(test)]
 mod tests {

diff -uNr 15_virtual_mem_part3_precomputed_tables/kernel/src/_arch/aarch64/memory/mmu.rs 16_virtual_mem_part4_higher_half_kernel/kernel/src/_arch/aarch64/memory/mmu.rs
--- 15_virtual_mem_part3_precomputed_tables/kernel/src/_arch/aarch64/memory/mmu.rs
+++ 16_virtual_mem_part4_higher_half_kernel/kernel/src/_arch/aarch64/memory/mmu.rs
@@ -66,6 +66,7 @@

 impl MemoryManagementUnit {
     /// Setup function for the MAIR_EL1 register.
+    #[inline(always)]
     fn set_up_mair(&self) {
         // Define the memory types being mapped.
         MAIR_EL1.write(
@@ -79,20 +80,21 @@
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
@@ -131,7 +133,7 @@
         self.set_up_mair();

         // Set the "Translation Table Base Register".
-        TTBR0_EL1.set_baddr(phys_tables_base_addr.as_usize() as u64);
+        TTBR1_EL1.set_baddr(phys_tables_base_addr.as_usize() as u64);

         self.configure_translation_control();


diff -uNr 15_virtual_mem_part3_precomputed_tables/kernel/src/bsp/raspberrypi/kernel.ld 16_virtual_mem_part4_higher_half_kernel/kernel/src/bsp/raspberrypi/kernel.ld
--- 15_virtual_mem_part3_precomputed_tables/kernel/src/bsp/raspberrypi/kernel.ld
+++ 16_virtual_mem_part4_higher_half_kernel/kernel/src/bsp/raspberrypi/kernel.ld
@@ -8,6 +8,13 @@
 PAGE_SIZE = 64K;
 PAGE_MASK = PAGE_SIZE - 1;

+/* The kernel's virtual address range will be:
+ *
+ * [END_ADDRESS_INCLUSIVE, START_ADDRESS]
+ * [u64::MAX             , (u64::MAX - __kernel_virt_addr_space_size) + 1]
+ */
+__kernel_virt_start_addr = ((0xffffffffffffffff - __kernel_virt_addr_space_size) + 1);
+
 __rpi_phys_dram_start_addr = 0;

 /* The physical address at which the the kernel binary will be loaded by the Raspberry's firmware */
@@ -26,34 +33,22 @@
  */
 PHDRS
 {
-    segment_boot_core_stack PT_LOAD FLAGS(6);
     segment_code            PT_LOAD FLAGS(5);
     segment_data            PT_LOAD FLAGS(6);
+    segment_boot_core_stack PT_LOAD FLAGS(6);
 }

 SECTIONS
 {
-    . =  __rpi_phys_dram_start_addr;
-
-    /***********************************************************************************************
-    * Boot Core Stack
-    ***********************************************************************************************/
-    .boot_core_stack (NOLOAD) :
-    {
-        __boot_core_stack_start = .;         /*   ^             */
-                                             /*   | stack       */
-        . += __rpi_phys_binary_load_addr;    /*   | growth      */
-                                             /*   | direction   */
-        __boot_core_stack_end_exclusive = .; /*   |             */
-    } :segment_boot_core_stack
+    . =  __kernel_virt_start_addr;

-    ASSERT((. & PAGE_MASK) == 0, "End of boot core stack is not page aligned")
+    ASSERT((. & PAGE_MASK) == 0, "Start of address space is not page aligned")

     /***********************************************************************************************
     * Code + RO Data + Global Offset Table
     ***********************************************************************************************/
     __code_start = .;
-    .text :
+    .text : AT(__rpi_phys_binary_load_addr)
     {
         KEEP(*(.text._start))
         *(.text._start_arguments) /* Constants (or statics in Rust speak) read by _start(). */
@@ -91,6 +86,25 @@
     . += 8 * 1024 * 1024;
     __mmio_remap_end_exclusive = .;

+    ASSERT((. & PAGE_MASK) == 0, "MMIO remap reservation is not page aligned")
+
+    /***********************************************************************************************
+    * Guard Page
+    ***********************************************************************************************/
+    . += PAGE_SIZE;
+
+    /***********************************************************************************************
+    * Boot Core Stack
+    ***********************************************************************************************/
+    .boot_core_stack (NOLOAD) : AT(__rpi_phys_dram_start_addr)
+    {
+        __boot_core_stack_start = .;         /*   ^             */
+                                             /*   | stack       */
+        . += __rpi_phys_binary_load_addr;    /*   | growth      */
+                                             /*   | direction   */
+        __boot_core_stack_end_exclusive = .; /*   |             */
+    } :segment_boot_core_stack
+
     ASSERT((. & PAGE_MASK) == 0, "End of boot core stack is not page aligned")

     /***********************************************************************************************

diff -uNr 15_virtual_mem_part3_precomputed_tables/kernel/src/bsp/raspberrypi/memory/mmu.rs 16_virtual_mem_part4_higher_half_kernel/kernel/src/bsp/raspberrypi/memory/mmu.rs
--- 15_virtual_mem_part3_precomputed_tables/kernel/src/bsp/raspberrypi/memory/mmu.rs
+++ 16_virtual_mem_part4_higher_half_kernel/kernel/src/bsp/raspberrypi/memory/mmu.rs
@@ -20,7 +20,7 @@
 //--------------------------------------------------------------------------------------------------

 type KernelTranslationTable =
-    <KernelVirtAddrSpace as AssociatedTranslationTable>::TableStartFromBottom;
+    <KernelVirtAddrSpace as AssociatedTranslationTable>::TableStartFromTop;

 //--------------------------------------------------------------------------------------------------
 // Public Definitions
@@ -153,14 +153,6 @@
 /// `translation table tool` and patched into the kernel binary. This function just adds the mapping
 /// record entries.
 pub fn kernel_add_mapping_records_for_precomputed() {
-    let virt_boot_core_stack_region = virt_boot_core_stack_region();
-    generic_mmu::kernel_add_mapping_record(
-        "Kernel boot-core stack",
-        &virt_boot_core_stack_region,
-        &kernel_virt_to_phys_region(virt_boot_core_stack_region),
-        &kernel_page_attributes(virt_boot_core_stack_region.start_page_addr()),
-    );
-
     let virt_code_region = virt_code_region();
     generic_mmu::kernel_add_mapping_record(
         "Kernel code and RO data",
@@ -176,4 +168,12 @@
         &kernel_virt_to_phys_region(virt_data_region),
         &kernel_page_attributes(virt_data_region.start_page_addr()),
     );
+
+    let virt_boot_core_stack_region = virt_boot_core_stack_region();
+    generic_mmu::kernel_add_mapping_record(
+        "Kernel boot-core stack",
+        &virt_boot_core_stack_region,
+        &kernel_virt_to_phys_region(virt_boot_core_stack_region),
+        &kernel_page_attributes(virt_boot_core_stack_region.start_page_addr()),
+    );
 }

diff -uNr 15_virtual_mem_part3_precomputed_tables/kernel/src/bsp/raspberrypi/memory.rs 16_virtual_mem_part4_higher_half_kernel/kernel/src/bsp/raspberrypi/memory.rs
--- 15_virtual_mem_part3_precomputed_tables/kernel/src/bsp/raspberrypi/memory.rs
+++ 16_virtual_mem_part4_higher_half_kernel/kernel/src/bsp/raspberrypi/memory.rs
@@ -37,13 +37,7 @@
 //! The virtual memory layout is as follows:
 //!
 //! +---------------------------------------+
-//! |                                       | boot_core_stack_start @ 0x0
-//! |                                       |                                ^
-//! | Boot-core Stack                       |                                | stack
-//! |                                       |                                | growth
-//! |                                       |                                | direction
-//! +---------------------------------------+
-//! |                                       | code_start @ 0x8_0000 == boot_core_stack_end_exclusive
+//! |                                       | code_start @ __kernel_virt_start_addr
 //! | .text                                 |
 //! | .rodata                               |
 //! | .got                                  |
@@ -59,6 +53,16 @@
 //! |                                       |
 //! +---------------------------------------+
 //! |                                       |  mmio_remap_end_exclusive
+//! | Unmapped guard page                   |
+//! |                                       |
+//! +---------------------------------------+
+//! |                                       | boot_core_stack_start
+//! |                                       |                                ^
+//! | Boot-core Stack                       |                                | stack
+//! |                                       |                                | growth
+//! |                                       |                                | direction
+//! +---------------------------------------+
+//! |                                       | boot_core_stack_end_exclusive
 //! |                                       |
 pub mod mmu;


diff -uNr 15_virtual_mem_part3_precomputed_tables/kernel/src/lib.rs 16_virtual_mem_part4_higher_half_kernel/kernel/src/lib.rs
--- 15_virtual_mem_part3_precomputed_tables/kernel/src/lib.rs
+++ 16_virtual_mem_part4_higher_half_kernel/kernel/src/lib.rs
@@ -157,11 +157,6 @@
     )
 }

-#[cfg(not(test))]
-extern "Rust" {
-    fn kernel_init() -> !;
-}
-
 //--------------------------------------------------------------------------------------------------
 // Testing
 //--------------------------------------------------------------------------------------------------

diff -uNr 15_virtual_mem_part3_precomputed_tables/kernel/src/memory/mmu/translation_table.rs 16_virtual_mem_part4_higher_half_kernel/kernel/src/memory/mmu/translation_table.rs
--- 15_virtual_mem_part3_precomputed_tables/kernel/src/memory/mmu/translation_table.rs
+++ 16_virtual_mem_part4_higher_half_kernel/kernel/src/memory/mmu/translation_table.rs
@@ -99,9 +99,9 @@

         assert_eq!(tables.init(), Ok(()));

-        let virt_start_page_addr: PageAddress<Virtual> = PageAddress::from(0);
-        let virt_end_exclusive_page_addr: PageAddress<Virtual> =
-            virt_start_page_addr.checked_offset(5).unwrap();
+        let virt_end_exclusive_page_addr: PageAddress<Virtual> = PageAddress::MAX;
+        let virt_start_page_addr: PageAddress<Virtual> =
+            virt_end_exclusive_page_addr.checked_offset(-5).unwrap();

         let phys_start_page_addr: PageAddress<Physical> = PageAddress::from(0);
         let phys_end_exclusive_page_addr: PageAddress<Physical> =
@@ -124,7 +124,7 @@
         );

         assert_eq!(
-            tables.try_page_attributes(virt_start_page_addr.checked_offset(6).unwrap()),
+            tables.try_page_attributes(virt_start_page_addr.checked_offset(-1).unwrap()),
             Err("Page marked invalid")
         );


diff -uNr 15_virtual_mem_part3_precomputed_tables/kernel/src/memory/mmu/types.rs 16_virtual_mem_part4_higher_half_kernel/kernel/src/memory/mmu/types.rs
--- 15_virtual_mem_part3_precomputed_tables/kernel/src/memory/mmu/types.rs
+++ 16_virtual_mem_part4_higher_half_kernel/kernel/src/memory/mmu/types.rs
@@ -67,6 +67,11 @@
 // PageAddress
 //------------------------------------------------------------------------------
 impl<ATYPE: AddressType> PageAddress<ATYPE> {
+    /// The largest value that can be represented by this type.
+    pub const MAX: Self = PageAddress {
+        inner: Address::new(usize::MAX).align_down_page(),
+    };
+
     /// Unwraps the value.
     pub fn into_inner(self) -> Address<ATYPE> {
         self.inner

diff -uNr 15_virtual_mem_part3_precomputed_tables/kernel/src/memory/mmu.rs 16_virtual_mem_part4_higher_half_kernel/kernel/src/memory/mmu.rs
--- 15_virtual_mem_part3_precomputed_tables/kernel/src/memory/mmu.rs
+++ 16_virtual_mem_part4_higher_half_kernel/kernel/src/memory/mmu.rs
@@ -66,6 +66,11 @@
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

diff -uNr 15_virtual_mem_part3_precomputed_tables/kernel/tests/02_exception_sync_page_fault.rs 16_virtual_mem_part4_higher_half_kernel/kernel/tests/02_exception_sync_page_fault.rs
--- 15_virtual_mem_part3_precomputed_tables/kernel/tests/02_exception_sync_page_fault.rs
+++ 16_virtual_mem_part4_higher_half_kernel/kernel/tests/02_exception_sync_page_fault.rs
@@ -28,8 +28,8 @@
     // This line will be printed as the test header.
     println!("Testing synchronous exception handling by causing a page fault");

-    info!("Writing beyond mapped area to address 9 GiB...");
-    let big_addr: u64 = 9 * 1024 * 1024 * 1024;
+    info!("Writing to bottom of address space to address 1 GiB...");
+    let big_addr: u64 = 1024 * 1024 * 1024;
     core::ptr::read_volatile(big_addr as *mut u64);

     // If execution reaches here, the memory access above did not cause a page fault exception.

diff -uNr 15_virtual_mem_part3_precomputed_tables/tools/translation_table_tool/arch.rb 16_virtual_mem_part4_higher_half_kernel/tools/translation_table_tool/arch.rb
--- 15_virtual_mem_part3_precomputed_tables/tools/translation_table_tool/arch.rb
+++ 16_virtual_mem_part4_higher_half_kernel/tools/translation_table_tool/arch.rb
@@ -255,6 +255,8 @@
     end

     def lvl2_lvl3_index_from(addr)
+        addr -= BSP.kernel_virt_start_addr
+
         lvl2_index = addr >> Granule512MiB::SHIFT
         lvl3_index = (addr & Granule512MiB::MASK) >> Granule64KiB::SHIFT


diff -uNr 15_virtual_mem_part3_precomputed_tables/tools/translation_table_tool/bsp.rb 16_virtual_mem_part4_higher_half_kernel/tools/translation_table_tool/bsp.rb
--- 15_virtual_mem_part3_precomputed_tables/tools/translation_table_tool/bsp.rb
+++ 16_virtual_mem_part4_higher_half_kernel/tools/translation_table_tool/bsp.rb
@@ -6,7 +6,7 @@

 # Raspberry Pi 3 + 4
 class RaspberryPi
-    attr_reader :kernel_granule, :kernel_virt_addr_space_size
+    attr_reader :kernel_granule, :kernel_virt_addr_space_size, :kernel_virt_start_addr

     MEMORY_SRC = File.read('kernel/src/bsp/raspberrypi/memory.rs').split("\n")

@@ -14,6 +14,7 @@
         @kernel_granule = Granule64KiB

         @kernel_virt_addr_space_size = KERNEL_ELF.symbol_value('__kernel_virt_addr_space_size')
+        @kernel_virt_start_addr = KERNEL_ELF.symbol_value('__kernel_virt_start_addr')

         @virt_addr_of_kernel_tables = KERNEL_ELF.symbol_value('KERNEL_TABLES')
         @virt_addr_of_phys_kernel_tables_base_addr = KERNEL_ELF.symbol_value(

```
