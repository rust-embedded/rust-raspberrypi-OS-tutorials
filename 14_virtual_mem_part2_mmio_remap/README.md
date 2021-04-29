# Tutorial 14 - Virtual Memory Part 2: MMIO Remap

## tl;dr

- We introduce a first set of changes which is eventually needed for separating `kernel` and `user`
  address spaces.
- The memory mapping strategy gets more sophisticated as we do away with `identity mapping` the
  whole of the board's address space.
- Instead, only ranges that are actually needed are mapped:
    - The `kernel binary` stays `identity mapped` for now.
    - Device `MMIO regions` are remapped lazily to a special virtual address region at the top of
      the virtual address space during the device driver's `init()`.

## Table of Contents

- [Introduction](#introduction)
- [Implementation](#implementation)
  - [A New Mapping API in `src/memory/mmu.rs`](#a-new-mapping-api-in-srcmemorymmutranslationtablers)
  - [The new APIs in action](#the-new-apis-in-action)
  - [Supporting Changes](#supporting-changes)
- [Test it](#test-it)
- [Diff to previous](#diff-to-previous)

## Introduction

This tutorial is a first step of many needed for enabling `userspace applications` (which we
hopefully will have some day in the very distant future).

For this, one of the features we want is a clean separation of `kernel` and `user` address spaces.
Fortunately, `ARMv8` has convenient architecture support to realize this. The following text and
pictue gives some more motivation and technical information. It is quoted from the _[ARM Cortex-A
Series Programmer’s Guide for ARMv8-A], Chapter 12.2, Separation of kernel and application Virtual
Address spaces_:

> Operating systems typically have a number of applications or tasks running concurrently. Each of
> these has its own unique set of translation tables and the kernel switches from one to another as
> part of the process of switching context between one task and another. However, much of the memory
> system is used only by the kernel and has fixed virtual to Physical Address mappings where the
> translation table entries rarely change. The ARMv8 architecture provides a number of features to
> efficiently handle this requirement.
>
> The table base addresses are specified in the Translation Table Base Registers `TTBR0_EL1` and
> `TTBR1_EL1`. The translation table pointed to by `TTBR0` is selected when the upper bits of the VA
> are all 0. `TTBR1` is selected when the upper bits of the VA are all set to 1. [...]
>
> Figure 12-4 shows how the kernel space can be mapped to the most significant area of memory and
> the Virtual Address space associated with each application mapped to the least significant area of
> memory. However, both of these are mapped to a much smaller Physical Address space.

<p align="center">
    <img src="../doc/15_kernel_user_address_space_partitioning.png" height="500" align="center">
</p>

This approach is also sometimes called a "[higher half kernel]". To eventually achieve this
separation, this tutorial makes a start by changing the following things:

1. Instead of bulk-`identity mapping` the whole of the board's address space, only the particular
   parts that are needed will be mapped.
1. For now, the `kernel binary` stays identity mapped. This will be changed in the in the coming
   tutorials as it is a quite difficult and peculiar exercise to remap the kernel.
1. Device `MMIO regions` are lazily remapped during the device driver's `init()`.
   1. The remappings will populate the top of the virtual address space. In the `AArch64 MMU
      Driver`, we provide the top `256 MiB` for it.
   1. It is possible to define the size of the virtual address space at compile time. We chose `8
      GiB` for now, which means remapped MMIO virtual addresses will start at `7936 MiB`
      (`0x1_f000_0000`).
1. We keep using `TTBR0` for the kernel translation tables for now. This will be changed when we
   remap the `kernel binary` in the coming tutorials.

[ARM Cortex-A Series Programmer’s Guide for ARMv8-A]: https://developer.arm.com/documentation/den0024/latest/
[higher half kernel]: https://wiki.osdev.org/Higher_Half_Kernel

## Implementation

Until now, the whole address space of the board was identity mapped at once. The **architecture**
(`src/_arch/_/memory/**`) and **bsp** (`src/bsp/_/memory/**`) parts of the kernel worked
together directly while setting up the translation tables, without any indirection through **generic
kernel code** (`src/memory/**`).

The way it worked was that the `architectural MMU code` would query the `bsp code` about the start
and end of the physical address space, and any special regions in this space that need a mapping
that _is not_ normal chacheable DRAM. It would then go ahead and map the whole address space at once
and never touch the translation tables again during runtime.

Changing in this tutorial, **architecture** and **bsp** code will no longer autonomously create the
virtual memory mappings. Instead, this is now orchestrated by the kernel's **generic MMU subsystem
code**.

### A New Mapping API in `src/memory/mmu/translation_table.rs`

First, we define an interface for operating on `translation tables`:

```rust
/// Translation table operations.
pub trait TranslationTable {
    /// Anything that needs to run before any of the other provided functions can be used.
    fn init(&mut self);

    /// The translation table's base address to be used for programming the MMU.
    fn phys_base_address(&self) -> Address<Physical>;

    /// Map the given virtual pages to the given physical pages.
    unsafe fn map_pages_at(
        &mut self,
        virt_pages: &PageSliceDescriptor<Virtual>,
        phys_pages: &PageSliceDescriptor<Physical>,
        attr: &AttributeFields,
    ) -> Result<(), &'static str>;

    /// Obtain a free virtual page slice in the MMIO region.
    fn next_mmio_virt_page_slice(
        &mut self,
        num_pages: usize,
    ) -> Result<PageSliceDescriptor<Virtual>, &'static str>;

    /// Check if a virtual page splice is in the "MMIO region".
    fn is_virt_page_slice_mmio(&self, virt_pages: &PageSliceDescriptor<Virtual>) -> bool;
}
```

In order to enable the generic kernel code to manipulate the kernel's translation tables, they must
first be made accessible. Until now, they were just a "hidden" struct in the `architectural` MMU
driver (`src/arch/.../memory/mmu.rs`). This made sense because the MMU driver code was the only code
that needed to be concerned with the table data structure, so having it accessible locally
simplified things.

Since the tables need to be exposed to the rest of the kernel code now, it makes sense to move them
to `BSP` code. Because ultimately, it is the `BSP` that is defining the translation table's
properties, such as the size of the virtual address space that the tables need to cover.

They are now defined in the global instances region of `src/bsp/.../memory/mmu.rs`. To control
access, they are  guarded by an `InitStateLock`.

```rust
//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

/// The kernel translation tables.
static KERNEL_TABLES: InitStateLock<KernelTranslationTable> =
    InitStateLock::new(KernelTranslationTable::new());
```

The struct `KernelTranslationTable` is a type alias defined in the same file, which in turn gets its
definition from an associated type of type `KernelVirtAddrSpace`, which itself is a type alias of
`memory::mmu::AddressSpace`. I know this sounds horribly complicated, but in the end this is just
some layers of `const generics` whose implementation is scattered between `generic` and `arch` code.
This is done to (1) ensure a sane compile-time definition of the translation table struct (by doing
various bounds checks), and (2) to separate concerns between generic `MMU` code and specializations
that come from the `architectural` part.

In the end, these tables can be accessed by calling `bsp::memory::mmu::kernel_translation_tables()`:

```rust
/// Return a reference to the kernel's translation tables.
pub fn kernel_translation_tables() -> &'static InitStateLock<KernelTranslationTable> {
    &KERNEL_TABLES
}
```

Finally, the generic kernel code (`src/memory/mmu.rs`) now provides a couple of memory mapping
functions that access and manipulate this instance. They  are exported for the rest of the kernel to
use:

```rust
/// Raw mapping of virtual to physical pages in the kernel translation tables.
///
/// Prevents mapping into the MMIO range of the tables.
pub unsafe fn kernel_map_pages_at(
    name: &'static str,
    virt_pages: &PageSliceDescriptor<Virtual>,
    phys_pages: &PageSliceDescriptor<Physical>,
    attr: &AttributeFields,
) -> Result<(), &'static str>;

/// MMIO remapping in the kernel translation tables.
///
/// Typically used by device drivers.
pub unsafe fn kernel_map_mmio(
    name: &'static str,
    phys_mmio_descriptor: &MMIODescriptor<Physical>,
) -> Result<Address<Virtual>, &'static str>;

/// Map the kernel's binary. Returns the translation table's base address.
pub unsafe fn kernel_map_binary() -> Result<Address<Physical>, &'static str>;

/// Enable the MMU and data + instruction caching.
pub unsafe fn enable_mmu_and_caching(
    phys_tables_base_addr: Address<Physical>,
) -> Result<(), MMUEnableError>;
```

### The new APIs in action

`kernel_map_binary()` and `enable_mmu_and_caching()` are used early in `kernel_init()` to set up
virtual memory:

```rust
let phys_kernel_tables_base_addr = match memory::mmu::kernel_map_binary() {
    Err(string) => panic!("Error mapping kernel binary: {}", string),
    Ok(addr) => addr,
};

if let Err(e) = memory::mmu::enable_mmu_and_caching(phys_kernel_tables_base_addr) {
    panic!("Enabling MMU failed: {}", e);
}
```

Both functions internally use `bsp` and `arch` specific code to achieve their goals. For example,
`memory::mmu::kernel_map_binary()` itself wraps around a `bsp` function of the same name
(`bsp::memory::mmu::kernel_map_binary()`):

```rust
/// Map the kernel binary.
pub unsafe fn kernel_map_binary() -> Result<(), &'static str> {
    generic_mmu::kernel_map_pages_at(
        "Kernel boot-core stack",
        &virt_stack_page_desc(),
        &phys_stack_page_desc(),
        &AttributeFields {
            mem_attributes: MemAttributes::CacheableDRAM,
            acc_perms: AccessPermissions::ReadWrite,
            execute_never: true,
        },
    )?;

    generic_mmu::kernel_map_pages_at(
        "Kernel code and RO data",
        // omitted for brevity.
    )?;

    generic_mmu::kernel_map_pages_at(
        "Kernel data and bss",
        // omitted for brevity.
    )?;

    Ok(())
}
```

Another user of the new APIs are device drivers, which now expect an `MMIODescriptor` type instead
of a raw address. The following is an example for the `UART`:

```rust
impl PL011Uart {
    /// Create an instance.
    pub const unsafe fn new(
        phys_mmio_descriptor: memory::mmu::MMIODescriptor<Physical>,
        irq_number: bsp::device_driver::IRQNumber,
    ) -> Self {
        Self {
             // omitted for brevity.
        }
    }
}
```

When the kernel calls the driver's implementation of `driver::interface::DeviceDriver::init()`
during kernel boot, the MMIO Descriptor is used to remap the MMIO region on demand:

```rust
unsafe fn init(&self) -> Result<(), &'static str> {
    let virt_addr =
        memory::mmu::kernel_map_mmio(self.compatible(), &self.phys_mmio_descriptor)?;

    self.inner.lock(|inner| inner.init(Some(virt_addr.into_usize())))?;

     // omitted for brevity.

     Ok(())
}
```

### Supporting Changes

There's a couple of changes not covered in this tutorial text, but the reader should ideally skim
through them:

- [`src/bsp/raspberrypi/memory.rs`](src/bsp/raspberrypi/memory.rs) and
  [`src/bsp/raspberrypi/link.ld`](src/bsp/raspberrypi/link.ld) changed the location of the boot
  core's stack. It is now located after the data segment, and separated by an unmapped `guard page`.
  There is also supporting code in
  [`src/_arch/aarch64/exception.rs`](src/_arch/aarch64/exception.rs) that runs on data aborts and
  checks if the fault address lies within the `stack guard page`. This can be an indication that a
  kernel stack overflow happened.
- [`src/memory/mmu/types.rs`](src/memory/mmu/types.rs) introduces a couple of supporting types, like
  `Page<ATYPE>`.
- [`src/memory/mmu/mapping_record.rs`](src/memory/mmu/mapping_record.rs) provides the generic kernel
  code's way of tracking previous memory mappings for use cases such as reusing existing mappings
  (in case of drivers that have their MMIO ranges in the same `64 KiB` page) or printing mappings
  statistics.

## Test it

When you load the kernel, you can now see that the driver's MMIO virtual addresses start at
`0x1_f000_0000`:

Raspberry Pi 3:

```console
$ make chainboot
[...]
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
[MP] ⏩ Pushing 67 KiB =========================================🦀 100% 0 KiB/s Time: 00:00:00
[ML] Loaded! Executing the payload now

[    0.785711] mingo version 0.14.0
[    0.785919] Booting on: Raspberry Pi 3
[    0.786374] MMU online:
[    0.786666]       -------------------------------------------------------------------------------------------------------------------------------------------
[    0.788410]                         Virtual                                   Physical               Size       Attr                    Entity
[    0.790154]       -------------------------------------------------------------------------------------------------------------------------------------------
[    0.791902]       0x0000_0000_0008_0000..0x0000_0000_0008_ffff --> 0x00_0008_0000..0x00_0008_ffff |  64 KiB | C   RO X  | Kernel code and RO data
[    0.793515]       0x0000_0000_0009_0000..0x0000_0000_001b_ffff --> 0x00_0009_0000..0x00_001b_ffff |   1 MiB | C   RW XN | Kernel data and bss
[    0.795085]       0x0000_0000_001d_0000..0x0000_0000_0024_ffff --> 0x00_001d_0000..0x00_0024_ffff | 512 KiB | C   RW XN | Kernel boot-core stack
[    0.796688]       0x0000_0001_f000_0000..0x0000_0001_f000_ffff --> 0x00_3f20_0000..0x00_3f20_ffff |  64 KiB | Dev RW XN | BCM GPIO
[    0.798139]                                                                                                             | BCM PL011 UART
[    0.799657]       0x0000_0001_f001_0000..0x0000_0001_f001_ffff --> 0x00_3f00_0000..0x00_3f00_ffff |  64 KiB | Dev RW XN | BCM Peripheral Interrupt Controller
[    0.801400]       -------------------------------------------------------------------------------------------------------------------------------------------
```

Raspberry Pi 4:

```console
$ BSP=rpi4 make chainboot
[...]
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
[MP] ⏩ Pushing 74 KiB =========================================🦀 100% 0 KiB/s Time: 00:00:00
[ML] Loaded! Executing the payload now

[    0.869392] mingo version 0.14.0
[    0.869425] Booting on: Raspberry Pi 4
[    0.869880] MMU online:
[    0.870173]       -------------------------------------------------------------------------------------------------------------------------------------------
[    0.871917]                         Virtual                                   Physical               Size       Attr                    Entity
[    0.873661]       -------------------------------------------------------------------------------------------------------------------------------------------
[    0.875407]       0x0000_0000_0008_0000..0x0000_0000_0008_ffff --> 0x00_0008_0000..0x00_0008_ffff |  64 KiB | C   RO X  | Kernel code and RO data
[    0.877021]       0x0000_0000_0009_0000..0x0000_0000_001b_ffff --> 0x00_0009_0000..0x00_001b_ffff |   1 MiB | C   RW XN | Kernel data and bss
[    0.878591]       0x0000_0000_001d_0000..0x0000_0000_0024_ffff --> 0x00_001d_0000..0x00_0024_ffff | 512 KiB | C   RW XN | Kernel boot-core stack
[    0.880195]       0x0000_0001_f000_0000..0x0000_0001_f000_ffff --> 0x00_fe20_0000..0x00_fe20_ffff |  64 KiB | Dev RW XN | BCM GPIO
[    0.881645]                                                                                                             | BCM PL011 UART
[    0.883163]       0x0000_0001_f001_0000..0x0000_0001_f001_ffff --> 0x00_ff84_0000..0x00_ff84_ffff |  64 KiB | Dev RW XN | GICD
[    0.884570]                                                                                                             | GICC
[    0.885979]       -------------------------------------------------------------------------------------------------------------------------------------------
```

## Diff to previous
```diff

diff -uNr 13_exceptions_part2_peripheral_IRQs/Cargo.toml 14_virtual_mem_part2_mmio_remap/Cargo.toml
--- 13_exceptions_part2_peripheral_IRQs/Cargo.toml
+++ 14_virtual_mem_part2_mmio_remap/Cargo.toml
@@ -1,6 +1,6 @@
 [package]
 name = "mingo"
-version = "0.13.0"
+version = "0.14.0"
 authors = ["Andre Richter <andre.o.richter@gmail.com>"]
 edition = "2018"


diff -uNr 13_exceptions_part2_peripheral_IRQs/src/_arch/aarch64/exception.rs 14_virtual_mem_part2_mmio_remap/src/_arch/aarch64/exception.rs
--- 13_exceptions_part2_peripheral_IRQs/src/_arch/aarch64/exception.rs
+++ 14_virtual_mem_part2_mmio_remap/src/_arch/aarch64/exception.rs
@@ -11,7 +11,11 @@
 //!
 //! crate::exception::arch_exception

-use crate::{bsp, exception};
+use crate::{
+    bsp::{self},
+    exception,
+    memory::Address,
+};
 use core::{cell::UnsafeCell, fmt};
 use cortex_a::{barrier, regs::*};
 use register::InMemoryRegister;
@@ -50,6 +54,20 @@
 // Private Code
 //--------------------------------------------------------------------------------------------------

+/// Check if additional context can be derived from a data abort.
+fn inspect_data_abort(f: &mut fmt::Formatter) -> fmt::Result {
+    let fault_addr = Address::new(FAR_EL1.get() as usize);
+
+    if bsp::memory::mmu::virt_boot_core_stack_guard_page_desc().contains(fault_addr) {
+        writeln!(
+            f,
+            "\n\n      >> Attempted to access the guard page of the kernel's boot core stack <<"
+        )?;
+    }
+
+    Ok(())
+}
+
 /// Prints verbose information about the exception and then panics.
 fn default_exception_handler(e: &ExceptionContext) {
     panic!(
@@ -166,7 +184,9 @@
         writeln!(f, " - {}", ec_translation)?;

         // Raw print of instruction specific syndrome.
-        write!(f, "      Instr Specific Syndrome (ISS): {:#x}", esr_el1.read(ESR_EL1::ISS))
+        write!(f, "      Instr Specific Syndrome (ISS): {:#x}", esr_el1.read(ESR_EL1::ISS))?;
+
+        inspect_data_abort(f)
     }
 }


diff -uNr 13_exceptions_part2_peripheral_IRQs/src/_arch/aarch64/memory/mmu/translation_table.rs 14_virtual_mem_part2_mmio_remap/src/_arch/aarch64/memory/mmu/translation_table.rs
--- 13_exceptions_part2_peripheral_IRQs/src/_arch/aarch64/memory/mmu/translation_table.rs
+++ 14_virtual_mem_part2_mmio_remap/src/_arch/aarch64/memory/mmu/translation_table.rs
@@ -15,9 +15,12 @@

 use crate::{
     bsp, memory,
-    memory::mmu::{
-        arch_mmu::{Granule512MiB, Granule64KiB},
-        AccessPermissions, AttributeFields, MemAttributes,
+    memory::{
+        mmu::{
+            arch_mmu::{Granule512MiB, Granule64KiB},
+            AccessPermissions, AttributeFields, MemAttributes, Page, PageSliceDescriptor,
+        },
+        Address, Physical, Virtual,
     },
 };
 use core::convert;
@@ -117,12 +120,9 @@
 }

 trait StartAddr {
-    fn phys_start_addr_u64(&self) -> u64;
-    fn phys_start_addr_usize(&self) -> usize;
+    fn phys_start_addr(&self) -> Address<Physical>;
 }

-const NUM_LVL2_TABLES: usize = bsp::memory::mmu::KernelAddrSpace::SIZE >> Granule512MiB::SHIFT;
-
 //--------------------------------------------------------------------------------------------------
 // Public Definitions
 //--------------------------------------------------------------------------------------------------
@@ -137,10 +137,13 @@

     /// Table descriptors, covering 512 MiB windows.
     lvl2: [TableDescriptor; NUM_TABLES],
-}

-/// A translation table type for the kernel space.
-pub type KernelTranslationTable = FixedSizeTranslationTable<NUM_LVL2_TABLES>;
+    /// Index of the next free MMIO page.
+    cur_l3_mmio_index: usize,
+
+    /// Have the tables been initialized?
+    initialized: bool,
+}

 //--------------------------------------------------------------------------------------------------
 // Private Code
@@ -148,12 +151,8 @@

 // The binary is still identity mapped, so we don't need to convert here.
 impl<T, const N: usize> StartAddr for [T; N] {
-    fn phys_start_addr_u64(&self) -> u64 {
-        self as *const T as u64
-    }
-
-    fn phys_start_addr_usize(&self) -> usize {
-        self as *const _ as usize
+    fn phys_start_addr(&self) -> Address<Physical> {
+        Address::new(self as *const _ as usize)
     }
 }

@@ -166,10 +165,10 @@
     }

     /// Create an instance pointing to the supplied address.
-    pub fn from_next_lvl_table_addr(phys_next_lvl_table_addr: usize) -> Self {
+    pub fn from_next_lvl_table_addr(phys_next_lvl_table_addr: Address<Physical>) -> Self {
         let val = InMemoryRegister::<u64, STAGE1_TABLE_DESCRIPTOR::Register>::new(0);

-        let shifted = phys_next_lvl_table_addr >> Granule64KiB::SHIFT;
+        let shifted = phys_next_lvl_table_addr.into_usize() >> Granule64KiB::SHIFT;
         val.write(
             STAGE1_TABLE_DESCRIPTOR::NEXT_LEVEL_TABLE_ADDR_64KiB.val(shifted as u64)
                 + STAGE1_TABLE_DESCRIPTOR::TYPE::Table
@@ -226,7 +225,10 @@
     }

     /// Create an instance.
-    pub fn from_output_addr(phys_output_addr: usize, attribute_fields: &AttributeFields) -> Self {
+    pub fn from_output_addr(
+        phys_output_addr: *const Page<Physical>,
+        attribute_fields: &AttributeFields,
+    ) -> Self {
         let val = InMemoryRegister::<u64, STAGE1_PAGE_DESCRIPTOR::Register>::new(0);

         let shifted = phys_output_addr as u64 >> Granule64KiB::SHIFT;
@@ -240,50 +242,193 @@

         Self { value: val.get() }
     }
+
+    /// Returns the valid bit.
+    fn is_valid(&self) -> bool {
+        InMemoryRegister::<u64, STAGE1_PAGE_DESCRIPTOR::Register>::new(self.value)
+            .is_set(STAGE1_PAGE_DESCRIPTOR::VALID)
+    }
 }

 //--------------------------------------------------------------------------------------------------
 // Public Code
 //--------------------------------------------------------------------------------------------------

+impl<const AS_SIZE: usize> memory::mmu::AssociatedTranslationTable
+    for memory::mmu::AddressSpace<AS_SIZE>
+where
+    [u8; Self::SIZE >> Granule512MiB::SHIFT]: Sized,
+{
+    type TableStartFromBottom = FixedSizeTranslationTable<{ Self::SIZE >> Granule512MiB::SHIFT }>;
+}
+
 impl<const NUM_TABLES: usize> FixedSizeTranslationTable<NUM_TABLES> {
+    // Reserve the last 256 MiB of the address space for MMIO mappings.
+    const L2_MMIO_START_INDEX: usize = NUM_TABLES - 1;
+    const L3_MMIO_START_INDEX: usize = 8192 / 2;
+
     /// Create an instance.
+    #[allow(clippy::assertions_on_constants)]
     pub const fn new() -> Self {
+        assert!(bsp::memory::mmu::KernelGranule::SIZE == Granule64KiB::SIZE);
+
         // Can't have a zero-sized address space.
         assert!(NUM_TABLES > 0);

         Self {
             lvl3: [[PageDescriptor::new_zeroed(); 8192]; NUM_TABLES],
             lvl2: [TableDescriptor::new_zeroed(); NUM_TABLES],
+            cur_l3_mmio_index: 0,
+            initialized: false,
         }
     }

-    /// Iterates over all static translation table entries and fills them at once.
-    ///
-    /// # Safety
-    ///
-    /// - Modifies a `static mut`. Ensure it only happens from here.
-    pub unsafe fn populate_tt_entries(&mut self) -> Result<(), &'static str> {
-        for (l2_nr, l2_entry) in self.lvl2.iter_mut().enumerate() {
-            *l2_entry =
-                TableDescriptor::from_next_lvl_table_addr(self.lvl3[l2_nr].phys_start_addr_usize());
+    /// The start address of the table's MMIO range.
+    #[inline(always)]
+    fn mmio_start_addr(&self) -> Address<Virtual> {
+        Address::new(
+            (Self::L2_MMIO_START_INDEX << Granule512MiB::SHIFT)
+                | (Self::L3_MMIO_START_INDEX << Granule64KiB::SHIFT),
+        )
+    }
+
+    /// The inclusive end address of the table's MMIO range.
+    #[inline(always)]
+    fn mmio_end_addr_inclusive(&self) -> Address<Virtual> {
+        Address::new(
+            (Self::L2_MMIO_START_INDEX << Granule512MiB::SHIFT)
+                | (8191 << Granule64KiB::SHIFT)
+                | (Granule64KiB::SIZE - 1),
+        )
+    }
+
+    /// Helper to calculate the lvl2 and lvl3 indices from an address.
+    #[inline(always)]
+    fn lvl2_lvl3_index_from(
+        &self,
+        addr: *const Page<Virtual>,
+    ) -> Result<(usize, usize), &'static str> {
+        let addr = addr as usize;
+        let lvl2_index = addr >> Granule512MiB::SHIFT;
+        let lvl3_index = (addr & Granule512MiB::MASK) >> Granule64KiB::SHIFT;
+
+        if lvl2_index > (NUM_TABLES - 1) {
+            return Err("Virtual page is out of bounds of translation table");
+        }
+
+        Ok((lvl2_index, lvl3_index))
+    }
+
+    /// Returns the PageDescriptor corresponding to the supplied Page.
+    #[inline(always)]
+    fn page_descriptor_from(
+        &mut self,
+        addr: *const Page<Virtual>,
+    ) -> Result<&mut PageDescriptor, &'static str> {
+        let (lvl2_index, lvl3_index) = self.lvl2_lvl3_index_from(addr)?;
+
+        Ok(&mut self.lvl3[lvl2_index][lvl3_index])
+    }
+}
+
+//------------------------------------------------------------------------------
+// OS Interface Code
+//------------------------------------------------------------------------------

-            for (l3_nr, l3_entry) in self.lvl3[l2_nr].iter_mut().enumerate() {
-                let virt_addr = (l2_nr << Granule512MiB::SHIFT) + (l3_nr << Granule64KiB::SHIFT);
+impl<const NUM_TABLES: usize> memory::mmu::translation_table::interface::TranslationTable
+    for FixedSizeTranslationTable<NUM_TABLES>
+{
+    fn init(&mut self) {
+        if self.initialized {
+            return;
+        }
+
+        // Populate the l2 entries.
+        for (lvl2_nr, lvl2_entry) in self.lvl2.iter_mut().enumerate() {
+            let desc =
+                TableDescriptor::from_next_lvl_table_addr(self.lvl3[lvl2_nr].phys_start_addr());
+            *lvl2_entry = desc;
+        }
+
+        self.cur_l3_mmio_index = Self::L3_MMIO_START_INDEX;
+        self.initialized = true;
+    }
+
+    fn phys_base_address(&self) -> Address<Physical> {
+        self.lvl2.phys_start_addr()
+    }
+
+    unsafe fn map_pages_at(
+        &mut self,
+        virt_pages: &PageSliceDescriptor<Virtual>,
+        phys_pages: &PageSliceDescriptor<Physical>,
+        attr: &AttributeFields,
+    ) -> Result<(), &'static str> {
+        assert!(self.initialized, "Translation tables not initialized");
+
+        let p = phys_pages.as_slice();
+        let v = virt_pages.as_slice();
+
+        // No work to do for empty slices.
+        if v.is_empty() {
+            return Ok(());
+        }

-                let (phys_output_addr, attribute_fields) =
-                    bsp::memory::mmu::virt_mem_layout().virt_addr_properties(virt_addr)?;
+        if v.len() != p.len() {
+            return Err("Tried to map page slices with unequal sizes");
+        }

-                *l3_entry = PageDescriptor::from_output_addr(phys_output_addr, &attribute_fields);
+        if p.last().unwrap().as_ptr() >= bsp::memory::mmu::phys_addr_space_end_page() {
+            return Err("Tried to map outside of physical address space");
+        }
+
+        let iter = p.iter().zip(v.iter());
+        for (phys_page, virt_page) in iter {
+            let page_descriptor = self.page_descriptor_from(virt_page.as_ptr())?;
+            if page_descriptor.is_valid() {
+                return Err("Virtual page is already mapped");
             }
+
+            *page_descriptor = PageDescriptor::from_output_addr(phys_page.as_ptr(), &attr);
         }

         Ok(())
     }

-    /// The translation table's base address to be used for programming the MMU.
-    pub fn phys_base_address(&self) -> u64 {
-        self.lvl2.phys_start_addr_u64()
+    fn next_mmio_virt_page_slice(
+        &mut self,
+        num_pages: usize,
+    ) -> Result<PageSliceDescriptor<Virtual>, &'static str> {
+        assert!(self.initialized, "Translation tables not initialized");
+
+        if num_pages == 0 {
+            return Err("num_pages == 0");
+        }
+
+        if (self.cur_l3_mmio_index + num_pages) > 8191 {
+            return Err("Not enough MMIO space left");
+        }
+
+        let addr = Address::new(
+            (Self::L2_MMIO_START_INDEX << Granule512MiB::SHIFT)
+                | (self.cur_l3_mmio_index << Granule64KiB::SHIFT),
+        );
+        self.cur_l3_mmio_index += num_pages;
+
+        Ok(PageSliceDescriptor::from_addr(addr, num_pages))
+    }
+
+    fn is_virt_page_slice_mmio(&self, virt_pages: &PageSliceDescriptor<Virtual>) -> bool {
+        let start_addr = virt_pages.start_addr();
+        let end_addr_inclusive = virt_pages.end_addr_inclusive();
+
+        for i in [start_addr, end_addr_inclusive].iter() {
+            if (*i >= self.mmio_start_addr()) && (*i <= self.mmio_end_addr_inclusive()) {
+                return true;
+            }
+        }
+
+        false
     }
 }

@@ -292,6 +437,9 @@
 //--------------------------------------------------------------------------------------------------

 #[cfg(test)]
+pub type MinSizeTranslationTable = FixedSizeTranslationTable<1>;
+
+#[cfg(test)]
 mod tests {
     use super::*;
     use test_macros::kernel_test;

diff -uNr 13_exceptions_part2_peripheral_IRQs/src/_arch/aarch64/memory/mmu.rs 14_virtual_mem_part2_mmio_remap/src/_arch/aarch64/memory/mmu.rs
--- 13_exceptions_part2_peripheral_IRQs/src/_arch/aarch64/memory/mmu.rs
+++ 14_virtual_mem_part2_mmio_remap/src/_arch/aarch64/memory/mmu.rs
@@ -15,7 +15,7 @@

 use crate::{
     bsp, memory,
-    memory::mmu::{translation_table::KernelTranslationTable, TranslationGranule},
+    memory::{mmu::TranslationGranule, Address, Physical},
 };
 use core::intrinsics::unlikely;
 use cortex_a::{barrier, regs::*};
@@ -45,13 +45,6 @@
 // Global instances
 //--------------------------------------------------------------------------------------------------

-/// The kernel translation tables.
-///
-/// # Safety
-///
-/// - Supposed to land in `.bss`. Therefore, ensure that all initial member values boil down to "0".
-static mut KERNEL_TABLES: KernelTranslationTable = KernelTranslationTable::new();
-
 static MMU: MemoryManagementUnit = MemoryManagementUnit;

 //--------------------------------------------------------------------------------------------------
@@ -86,7 +79,7 @@

     /// Configure various settings of stage 1 of the EL1 translation regime.
     fn configure_translation_control(&self) {
-        let t0sz = (64 - bsp::memory::mmu::KernelAddrSpace::SIZE_SHIFT) as u64;
+        let t0sz = (64 - bsp::memory::mmu::KernelVirtAddrSpace::SIZE_SHIFT) as u64;

         TCR_EL1.write(
             TCR_EL1::TBI0::Used
@@ -118,7 +111,10 @@
 use memory::mmu::MMUEnableError;

 impl memory::mmu::interface::MMU for MemoryManagementUnit {
-    unsafe fn enable_mmu_and_caching(&self) -> Result<(), MMUEnableError> {
+    unsafe fn enable_mmu_and_caching(
+        &self,
+        phys_tables_base_addr: Address<Physical>,
+    ) -> Result<(), MMUEnableError> {
         if unlikely(self.is_enabled()) {
             return Err(MMUEnableError::AlreadyEnabled);
         }
@@ -133,13 +129,8 @@
         // Prepare the memory attribute indirection register.
         self.set_up_mair();

-        // Populate translation tables.
-        KERNEL_TABLES
-            .populate_tt_entries()
-            .map_err(|e| MMUEnableError::Other(e))?;
-
         // Set the "Translation Table Base Register".
-        TTBR0_EL1.set_baddr(KERNEL_TABLES.phys_base_address());
+        TTBR0_EL1.set_baddr(phys_tables_base_addr.into_usize() as u64);

         self.configure_translation_control();

@@ -162,22 +153,3 @@
         SCTLR_EL1.matches_all(SCTLR_EL1::M::Enable)
     }
 }
-
-//--------------------------------------------------------------------------------------------------
-// Testing
-//--------------------------------------------------------------------------------------------------
-
-#[cfg(test)]
-mod tests {
-    use super::*;
-    use test_macros::kernel_test;
-
-    /// Check if KERNEL_TABLES is in .bss.
-    #[kernel_test]
-    fn kernel_tables_in_bss() {
-        let bss_range = bsp::memory::bss_range_inclusive();
-        let kernel_tables_addr = unsafe { &KERNEL_TABLES as *const _ as usize as *mut u64 };
-
-        assert!(bss_range.contains(&kernel_tables_addr));
-    }
-}

diff -uNr 13_exceptions_part2_peripheral_IRQs/src/bsp/device_driver/arm/gicv2/gicc.rs 14_virtual_mem_part2_mmio_remap/src/bsp/device_driver/arm/gicv2/gicc.rs
--- 13_exceptions_part2_peripheral_IRQs/src/bsp/device_driver/arm/gicv2/gicc.rs
+++ 14_virtual_mem_part2_mmio_remap/src/bsp/device_driver/arm/gicv2/gicc.rs
@@ -4,7 +4,9 @@

 //! GICC Driver - GIC CPU interface.

-use crate::{bsp::device_driver::common::MMIODerefWrapper, exception};
+use crate::{
+    bsp::device_driver::common::MMIODerefWrapper, exception, synchronization::InitStateLock,
+};
 use register::{mmio::*, register_bitfields, register_structs};

 //--------------------------------------------------------------------------------------------------
@@ -56,12 +58,13 @@

 /// Representation of the GIC CPU interface.
 pub struct GICC {
-    registers: Registers,
+    registers: InitStateLock<Registers>,
 }

 //--------------------------------------------------------------------------------------------------
 // Public Code
 //--------------------------------------------------------------------------------------------------
+use crate::synchronization::interface::ReadWriteEx;

 impl GICC {
     /// Create an instance.
@@ -71,10 +74,15 @@
     /// - The user must ensure to provide a correct MMIO start address.
     pub const unsafe fn new(mmio_start_addr: usize) -> Self {
         Self {
-            registers: Registers::new(mmio_start_addr),
+            registers: InitStateLock::new(Registers::new(mmio_start_addr)),
         }
     }

+    pub unsafe fn set_mmio(&self, new_mmio_start_addr: usize) {
+        self.registers
+            .write(|regs| *regs = Registers::new(new_mmio_start_addr));
+    }
+
     /// Accept interrupts of any priority.
     ///
     /// Quoting the GICv2 Architecture Specification:
@@ -87,7 +95,9 @@
     /// - GICC MMIO registers are banked per CPU core. It is therefore safe to have `&self` instead
     ///   of `&mut self`.
     pub fn priority_accept_all(&self) {
-        self.registers.PMR.write(PMR::Priority.val(255)); // Comment in arch spec.
+        self.registers.read(|regs| {
+            regs.PMR.write(PMR::Priority.val(255)); // Comment in arch spec.
+        });
     }

     /// Enable the interface - start accepting IRQs.
@@ -97,7 +107,9 @@
     /// - GICC MMIO registers are banked per CPU core. It is therefore safe to have `&self` instead
     ///   of `&mut self`.
     pub fn enable(&self) {
-        self.registers.CTLR.write(CTLR::Enable::SET);
+        self.registers.read(|regs| {
+            regs.CTLR.write(CTLR::Enable::SET);
+        });
     }

     /// Extract the number of the highest-priority pending IRQ.
@@ -113,7 +125,8 @@
         &self,
         _ic: &exception::asynchronous::IRQContext<'irq_context>,
     ) -> usize {
-        self.registers.IAR.read(IAR::InterruptID) as usize
+        self.registers
+            .read(|regs| regs.IAR.read(IAR::InterruptID) as usize)
     }

     /// Complete handling of the currently active IRQ.
@@ -132,6 +145,8 @@
         irq_number: u32,
         _ic: &exception::asynchronous::IRQContext<'irq_context>,
     ) {
-        self.registers.EOIR.write(EOIR::EOIINTID.val(irq_number));
+        self.registers.read(|regs| {
+            regs.EOIR.write(EOIR::EOIINTID.val(irq_number));
+        });
     }
 }

diff -uNr 13_exceptions_part2_peripheral_IRQs/src/bsp/device_driver/arm/gicv2/gicd.rs 14_virtual_mem_part2_mmio_remap/src/bsp/device_driver/arm/gicv2/gicd.rs
--- 13_exceptions_part2_peripheral_IRQs/src/bsp/device_driver/arm/gicv2/gicd.rs
+++ 14_virtual_mem_part2_mmio_remap/src/bsp/device_driver/arm/gicv2/gicd.rs
@@ -8,8 +8,9 @@
 //!   - SPI - Shared Peripheral Interrupt.

 use crate::{
-    bsp::device_driver::common::MMIODerefWrapper, state, synchronization,
-    synchronization::IRQSafeNullLock,
+    bsp::device_driver::common::MMIODerefWrapper,
+    state, synchronization,
+    synchronization::{IRQSafeNullLock, InitStateLock},
 };
 use register::{mmio::*, register_bitfields, register_structs};

@@ -79,7 +80,7 @@
     shared_registers: IRQSafeNullLock<SharedRegisters>,

     /// Access to banked registers is unguarded.
-    banked_registers: BankedRegisters,
+    banked_registers: InitStateLock<BankedRegisters>,
 }

 //--------------------------------------------------------------------------------------------------
@@ -116,6 +117,7 @@
 //--------------------------------------------------------------------------------------------------
 // Public Code
 //--------------------------------------------------------------------------------------------------
+use crate::synchronization::interface::ReadWriteEx;
 use synchronization::interface::Mutex;

 impl GICD {
@@ -127,10 +129,17 @@
     pub const unsafe fn new(mmio_start_addr: usize) -> Self {
         Self {
             shared_registers: IRQSafeNullLock::new(SharedRegisters::new(mmio_start_addr)),
-            banked_registers: BankedRegisters::new(mmio_start_addr),
+            banked_registers: InitStateLock::new(BankedRegisters::new(mmio_start_addr)),
         }
     }

+    pub unsafe fn set_mmio(&self, new_mmio_start_addr: usize) {
+        self.shared_registers
+            .lock(|regs| *regs = SharedRegisters::new(new_mmio_start_addr));
+        self.banked_registers
+            .write(|regs| *regs = BankedRegisters::new(new_mmio_start_addr));
+    }
+
     /// Use a banked ITARGETSR to retrieve the executing core's GIC target mask.
     ///
     /// Quoting the GICv2 Architecture Specification:
@@ -138,7 +147,8 @@
     ///   "GICD_ITARGETSR0 to GICD_ITARGETSR7 are read-only, and each field returns a value that
     ///    corresponds only to the processor reading the register."
     fn local_gic_target_mask(&self) -> u32 {
-        self.banked_registers.ITARGETSR[0].read(ITARGETSR::Offset0)
+        self.banked_registers
+            .read(|regs| regs.ITARGETSR[0].read(ITARGETSR::Offset0))
     }

     /// Route all SPIs to the boot core and enable the distributor.
@@ -177,10 +187,10 @@
         // Check if we are handling a private or shared IRQ.
         match irq_num {
             // Private.
-            0..=31 => {
-                let enable_reg = &self.banked_registers.ISENABLER;
+            0..=31 => self.banked_registers.read(|regs| {
+                let enable_reg = &regs.ISENABLER;
                 enable_reg.set(enable_reg.get() | enable_bit);
-            }
+            }),
             // Shared.
             _ => {
                 let enable_reg_index_shared = enable_reg_index - 1;

diff -uNr 13_exceptions_part2_peripheral_IRQs/src/bsp/device_driver/arm/gicv2.rs 14_virtual_mem_part2_mmio_remap/src/bsp/device_driver/arm/gicv2.rs
--- 13_exceptions_part2_peripheral_IRQs/src/bsp/device_driver/arm/gicv2.rs
+++ 14_virtual_mem_part2_mmio_remap/src/bsp/device_driver/arm/gicv2.rs
@@ -79,7 +79,8 @@
 mod gicc;
 mod gicd;

-use crate::{bsp, cpu, driver, exception, synchronization, synchronization::InitStateLock};
+use crate::{bsp, cpu, driver, exception, memory, synchronization, synchronization::InitStateLock};
+use core::sync::atomic::{AtomicBool, Ordering};

 //--------------------------------------------------------------------------------------------------
 // Private Definitions
@@ -96,12 +97,18 @@

 /// Representation of the GIC.
 pub struct GICv2 {
+    gicd_mmio_descriptor: memory::mmu::MMIODescriptor,
+    gicc_mmio_descriptor: memory::mmu::MMIODescriptor,
+
     /// The Distributor.
     gicd: gicd::GICD,

     /// The CPU Interface.
     gicc: gicc::GICC,

+    /// Have the MMIO regions been remapped yet?
+    is_mmio_remapped: AtomicBool,
+
     /// Stores registered IRQ handlers. Writable only during kernel init. RO afterwards.
     handler_table: InitStateLock<HandlerTable>,
 }
@@ -118,11 +125,17 @@
     ///
     /// # Safety
     ///
-    /// - The user must ensure to provide a correct MMIO start address.
-    pub const unsafe fn new(gicd_mmio_start_addr: usize, gicc_mmio_start_addr: usize) -> Self {
+    /// - The user must ensure to provide correct MMIO descriptors.
+    pub const unsafe fn new(
+        gicd_mmio_descriptor: memory::mmu::MMIODescriptor,
+        gicc_mmio_descriptor: memory::mmu::MMIODescriptor,
+    ) -> Self {
         Self {
-            gicd: gicd::GICD::new(gicd_mmio_start_addr),
-            gicc: gicc::GICC::new(gicc_mmio_start_addr),
+            gicd_mmio_descriptor,
+            gicc_mmio_descriptor,
+            gicd: gicd::GICD::new(gicd_mmio_descriptor.start_addr().into_usize()),
+            gicc: gicc::GICC::new(gicc_mmio_descriptor.start_addr().into_usize()),
+            is_mmio_remapped: AtomicBool::new(false),
             handler_table: InitStateLock::new([None; Self::NUM_IRQS]),
         }
     }
@@ -139,6 +152,22 @@
     }

     unsafe fn init(&self) -> Result<(), &'static str> {
+        let remapped = self.is_mmio_remapped.load(Ordering::Relaxed);
+        if !remapped {
+            let mut virt_addr;
+
+            // GICD
+            virt_addr = memory::mmu::kernel_map_mmio("GICD", &self.gicd_mmio_descriptor)?;
+            self.gicd.set_mmio(virt_addr.into_usize());
+
+            // GICC
+            virt_addr = memory::mmu::kernel_map_mmio("GICC", &self.gicc_mmio_descriptor)?;
+            self.gicc.set_mmio(virt_addr.into_usize());
+
+            // Conclude remapping.
+            self.is_mmio_remapped.store(true, Ordering::Relaxed);
+        }
+
         if bsp::cpu::BOOT_CORE_ID == cpu::smp::core_id() {
             self.gicd.boot_core_init();
         }

diff -uNr 13_exceptions_part2_peripheral_IRQs/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs 14_virtual_mem_part2_mmio_remap/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs
--- 13_exceptions_part2_peripheral_IRQs/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs
+++ 14_virtual_mem_part2_mmio_remap/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs
@@ -5,9 +5,10 @@
 //! GPIO Driver.

 use crate::{
-    bsp::device_driver::common::MMIODerefWrapper, driver, synchronization,
+    bsp::device_driver::common::MMIODerefWrapper, driver, memory, synchronization,
     synchronization::IRQSafeNullLock,
 };
+use core::sync::atomic::{AtomicUsize, Ordering};
 use register::{mmio::*, register_bitfields, register_structs};

 //--------------------------------------------------------------------------------------------------
@@ -117,6 +118,8 @@

 /// Representation of the GPIO HW.
 pub struct GPIO {
+    mmio_descriptor: memory::mmu::MMIODescriptor,
+    virt_mmio_start_addr: AtomicUsize,
     inner: IRQSafeNullLock<GPIOInner>,
 }

@@ -136,6 +139,19 @@
         }
     }

+    /// Init code.
+    ///
+    /// # Safety
+    ///
+    /// - The user must ensure to provide a correct MMIO start address.
+    pub unsafe fn init(&mut self, new_mmio_start_addr: Option<usize>) -> Result<(), &'static str> {
+        if let Some(addr) = new_mmio_start_addr {
+            self.registers = Registers::new(addr);
+        }
+
+        Ok(())
+    }
+
     /// Disable pull-up/down on pins 14 and 15.
     #[cfg(feature = "bsp_rpi3")]
     fn disable_pud_14_15_bcm2837(&mut self) {
@@ -190,10 +206,12 @@
     ///
     /// # Safety
     ///
-    /// - The user must ensure to provide a correct MMIO start address.
-    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
+    /// - The user must ensure to provide correct MMIO descriptors.
+    pub const unsafe fn new(mmio_descriptor: memory::mmu::MMIODescriptor) -> Self {
         Self {
-            inner: IRQSafeNullLock::new(GPIOInner::new(mmio_start_addr)),
+            mmio_descriptor,
+            virt_mmio_start_addr: AtomicUsize::new(0),
+            inner: IRQSafeNullLock::new(GPIOInner::new(mmio_descriptor.start_addr().into_usize())),
         }
     }

@@ -212,4 +230,26 @@
     fn compatible(&self) -> &'static str {
         "BCM GPIO"
     }
+
+    unsafe fn init(&self) -> Result<(), &'static str> {
+        let virt_addr = memory::mmu::kernel_map_mmio(self.compatible(), &self.mmio_descriptor)?;
+
+        self.inner
+            .lock(|inner| inner.init(Some(virt_addr.into_usize())))?;
+
+        self.virt_mmio_start_addr
+            .store(virt_addr.into_usize(), Ordering::Relaxed);
+
+        Ok(())
+    }
+
+    fn virt_mmio_start_addr(&self) -> Option<usize> {
+        let addr = self.virt_mmio_start_addr.load(Ordering::Relaxed);
+
+        if addr == 0 {
+            return None;
+        }
+
+        Some(addr)
+    }
 }

diff -uNr 13_exceptions_part2_peripheral_IRQs/src/bsp/device_driver/bcm/bcm2xxx_interrupt_controller/peripheral_ic.rs 14_virtual_mem_part2_mmio_remap/src/bsp/device_driver/bcm/bcm2xxx_interrupt_controller/peripheral_ic.rs
--- 13_exceptions_part2_peripheral_IRQs/src/bsp/device_driver/bcm/bcm2xxx_interrupt_controller/peripheral_ic.rs
+++ 14_virtual_mem_part2_mmio_remap/src/bsp/device_driver/bcm/bcm2xxx_interrupt_controller/peripheral_ic.rs
@@ -2,12 +2,12 @@
 //
 // Copyright (c) 2020-2021 Andre Richter <andre.o.richter@gmail.com>

-//! Peripheral Interrupt regsler Driver.
+//! Peripheral Interrupt Controller Driver.

 use super::{InterruptController, PendingIRQs, PeripheralIRQ};
 use crate::{
     bsp::device_driver::common::MMIODerefWrapper,
-    exception, synchronization,
+    driver, exception, memory, synchronization,
     synchronization::{IRQSafeNullLock, InitStateLock},
 };
 use register::{mmio::*, register_structs};
@@ -51,11 +51,13 @@

 /// Representation of the peripheral interrupt controller.
 pub struct PeripheralIC {
+    mmio_descriptor: memory::mmu::MMIODescriptor,
+
     /// Access to write registers is guarded with a lock.
     wo_registers: IRQSafeNullLock<WriteOnlyRegisters>,

     /// Register read access is unguarded.
-    ro_registers: ReadOnlyRegisters,
+    ro_registers: InitStateLock<ReadOnlyRegisters>,

     /// Stores registered IRQ handlers. Writable only during kernel init. RO afterwards.
     handler_table: InitStateLock<HandlerTable>,
@@ -70,21 +72,26 @@
     ///
     /// # Safety
     ///
-    /// - The user must ensure to provide a correct MMIO start address.
-    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
+    /// - The user must ensure to provide correct MMIO descriptors.
+    pub const unsafe fn new(mmio_descriptor: memory::mmu::MMIODescriptor) -> Self {
+        let addr = mmio_descriptor.start_addr().into_usize();
+
         Self {
-            wo_registers: IRQSafeNullLock::new(WriteOnlyRegisters::new(mmio_start_addr)),
-            ro_registers: ReadOnlyRegisters::new(mmio_start_addr),
+            mmio_descriptor,
+            wo_registers: IRQSafeNullLock::new(WriteOnlyRegisters::new(addr)),
+            ro_registers: InitStateLock::new(ReadOnlyRegisters::new(addr)),
             handler_table: InitStateLock::new([None; InterruptController::NUM_PERIPHERAL_IRQS]),
         }
     }

     /// Query the list of pending IRQs.
     fn pending_irqs(&self) -> PendingIRQs {
-        let pending_mask: u64 = (u64::from(self.ro_registers.PENDING_2.get()) << 32)
-            | u64::from(self.ro_registers.PENDING_1.get());
+        self.ro_registers.read(|regs| {
+            let pending_mask: u64 =
+                (u64::from(regs.PENDING_2.get()) << 32) | u64::from(regs.PENDING_1.get());

-        PendingIRQs::new(pending_mask)
+            PendingIRQs::new(pending_mask)
+        })
     }
 }

@@ -93,6 +100,24 @@
 //------------------------------------------------------------------------------
 use synchronization::interface::{Mutex, ReadWriteEx};

+impl driver::interface::DeviceDriver for PeripheralIC {
+    fn compatible(&self) -> &'static str {
+        "BCM Peripheral Interrupt Controller"
+    }
+
+    unsafe fn init(&self) -> Result<(), &'static str> {
+        let virt_addr =
+            memory::mmu::kernel_map_mmio(self.compatible(), &self.mmio_descriptor)?.into_usize();
+
+        self.wo_registers
+            .lock(|regs| *regs = WriteOnlyRegisters::new(virt_addr));
+        self.ro_registers
+            .write(|regs| *regs = ReadOnlyRegisters::new(virt_addr));
+
+        Ok(())
+    }
+}
+
 impl exception::asynchronous::interface::IRQManager for PeripheralIC {
     type IRQNumberType = PeripheralIRQ;


diff -uNr 13_exceptions_part2_peripheral_IRQs/src/bsp/device_driver/bcm/bcm2xxx_interrupt_controller.rs 14_virtual_mem_part2_mmio_remap/src/bsp/device_driver/bcm/bcm2xxx_interrupt_controller.rs
--- 13_exceptions_part2_peripheral_IRQs/src/bsp/device_driver/bcm/bcm2xxx_interrupt_controller.rs
+++ 14_virtual_mem_part2_mmio_remap/src/bsp/device_driver/bcm/bcm2xxx_interrupt_controller.rs
@@ -6,7 +6,7 @@

 mod peripheral_ic;

-use crate::{driver, exception};
+use crate::{driver, exception, memory};

 //--------------------------------------------------------------------------------------------------
 // Private Definitions
@@ -78,10 +78,13 @@
     ///
     /// # Safety
     ///
-    /// - The user must ensure to provide a correct MMIO start address.
-    pub const unsafe fn new(_local_mmio_start_addr: usize, periph_mmio_start_addr: usize) -> Self {
+    /// - The user must ensure to provide correct MMIO descriptors.
+    pub const unsafe fn new(
+        _local_mmio_descriptor: memory::mmu::MMIODescriptor,
+        periph_mmio_descriptor: memory::mmu::MMIODescriptor,
+    ) -> Self {
         Self {
-            periph: peripheral_ic::PeripheralIC::new(periph_mmio_start_addr),
+            periph: peripheral_ic::PeripheralIC::new(periph_mmio_descriptor),
         }
     }
 }
@@ -94,6 +97,10 @@
     fn compatible(&self) -> &'static str {
         "BCM Interrupt Controller"
     }
+
+    unsafe fn init(&self) -> Result<(), &'static str> {
+        self.periph.init()
+    }
 }

 impl exception::asynchronous::interface::IRQManager for InterruptController {

diff -uNr 13_exceptions_part2_peripheral_IRQs/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs 14_virtual_mem_part2_mmio_remap/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
--- 13_exceptions_part2_peripheral_IRQs/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
+++ 14_virtual_mem_part2_mmio_remap/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
@@ -10,10 +10,13 @@
 //! - <https://developer.arm.com/documentation/ddi0183/latest>

 use crate::{
-    bsp, bsp::device_driver::common::MMIODerefWrapper, console, cpu, driver, exception,
+    bsp, bsp::device_driver::common::MMIODerefWrapper, console, cpu, driver, exception, memory,
     synchronization, synchronization::IRQSafeNullLock,
 };
-use core::fmt;
+use core::{
+    fmt,
+    sync::atomic::{AtomicUsize, Ordering},
+};
 use register::{mmio::*, register_bitfields, register_structs};

 //--------------------------------------------------------------------------------------------------
@@ -232,6 +235,8 @@

 /// Representation of the UART.
 pub struct PL011Uart {
+    mmio_descriptor: memory::mmu::MMIODescriptor,
+    virt_mmio_start_addr: AtomicUsize,
     inner: IRQSafeNullLock<PL011UartInner>,
     irq_number: bsp::device_driver::IRQNumber,
 }
@@ -271,7 +276,15 @@
     /// genrated baud rate of `48_000_000 / (16 * 3.25) = 923_077`.
     ///
     /// Error = `((923_077 - 921_600) / 921_600) * 100 = 0.16modulo`.
-    pub fn init(&mut self) {
+    ///
+    /// # Safety
+    ///
+    /// - The user must ensure to provide a correct MMIO start address.
+    pub unsafe fn init(&mut self, new_mmio_start_addr: Option<usize>) -> Result<(), &'static str> {
+        if let Some(addr) = new_mmio_start_addr {
+            self.registers = Registers::new(addr);
+        }
+
         // Execution can arrive here while there are still characters queued in the TX FIFO and
         // actively being sent out by the UART hardware. If the UART is turned off in this case,
         // those queued characters would be lost.
@@ -313,6 +326,8 @@
         self.registers
             .CR
             .write(CR::UARTEN::Enabled + CR::TXE::Enabled + CR::RXE::Enabled);
+
+        Ok(())
     }

     /// Send a character.
@@ -390,13 +405,18 @@
     ///
     /// # Safety
     ///
-    /// - The user must ensure to provide a correct MMIO start address.
+    /// - The user must ensure to provide correct MMIO descriptors.
+    /// - The user must ensure to provide correct IRQ numbers.
     pub const unsafe fn new(
-        mmio_start_addr: usize,
+        mmio_descriptor: memory::mmu::MMIODescriptor,
         irq_number: bsp::device_driver::IRQNumber,
     ) -> Self {
         Self {
-            inner: IRQSafeNullLock::new(PL011UartInner::new(mmio_start_addr)),
+            mmio_descriptor,
+            virt_mmio_start_addr: AtomicUsize::new(0),
+            inner: IRQSafeNullLock::new(PL011UartInner::new(
+                mmio_descriptor.start_addr().into_usize(),
+            )),
             irq_number,
         }
     }
@@ -413,7 +433,13 @@
     }

     unsafe fn init(&self) -> Result<(), &'static str> {
-        self.inner.lock(|inner| inner.init());
+        let virt_addr = memory::mmu::kernel_map_mmio(self.compatible(), &self.mmio_descriptor)?;
+
+        self.inner
+            .lock(|inner| inner.init(Some(virt_addr.into_usize())))?;
+
+        self.virt_mmio_start_addr
+            .store(virt_addr.into_usize(), Ordering::Relaxed);

         Ok(())
     }
@@ -432,6 +458,16 @@

         Ok(())
     }
+
+    fn virt_mmio_start_addr(&self) -> Option<usize> {
+        let addr = self.virt_mmio_start_addr.load(Ordering::Relaxed);
+
+        if addr == 0 {
+            return None;
+        }
+
+        Some(addr)
+    }
 }

 impl console::interface::Write for PL011Uart {

diff -uNr 13_exceptions_part2_peripheral_IRQs/src/bsp/raspberrypi/console.rs 14_virtual_mem_part2_mmio_remap/src/bsp/raspberrypi/console.rs
--- 13_exceptions_part2_peripheral_IRQs/src/bsp/raspberrypi/console.rs
+++ 14_virtual_mem_part2_mmio_remap/src/bsp/raspberrypi/console.rs
@@ -5,7 +5,7 @@
 //! BSP console facilities.

 use super::memory;
-use crate::{bsp::device_driver, console};
+use crate::{bsp::device_driver, console, cpu, driver};
 use core::fmt;

 //--------------------------------------------------------------------------------------------------
@@ -23,11 +23,25 @@
 ///
 /// - Use only for printing during a panic.
 pub unsafe fn panic_console_out() -> impl fmt::Write {
-    let mut panic_gpio = device_driver::PanicGPIO::new(memory::map::mmio::GPIO_START);
-    let mut panic_uart = device_driver::PanicUart::new(memory::map::mmio::PL011_UART_START);
+    use driver::interface::DeviceDriver;

+    let mut panic_gpio = device_driver::PanicGPIO::new(memory::map::mmio::GPIO_START.into_usize());
+    let mut panic_uart =
+        device_driver::PanicUart::new(memory::map::mmio::PL011_UART_START.into_usize());
+
+    // If remapping of the driver's MMIO already happened, take the remapped start address.
+    // Otherwise, take a chance with the default physical address.
+    let maybe_gpio_mmio_start_addr = super::GPIO.virt_mmio_start_addr();
+    let maybe_uart_mmio_start_addr = super::PL011_UART.virt_mmio_start_addr();
+
+    panic_gpio
+        .init(maybe_gpio_mmio_start_addr)
+        .unwrap_or_else(|_| cpu::wait_forever());
     panic_gpio.map_pl011_uart();
-    panic_uart.init();
+    panic_uart
+        .init(maybe_uart_mmio_start_addr)
+        .unwrap_or_else(|_| cpu::wait_forever());
+
     panic_uart
 }


diff -uNr 13_exceptions_part2_peripheral_IRQs/src/bsp/raspberrypi/driver.rs 14_virtual_mem_part2_mmio_remap/src/bsp/raspberrypi/driver.rs
--- 13_exceptions_part2_peripheral_IRQs/src/bsp/raspberrypi/driver.rs
+++ 14_virtual_mem_part2_mmio_remap/src/bsp/raspberrypi/driver.rs
@@ -46,7 +46,15 @@
         &self.device_drivers[..]
     }

-    fn post_device_driver_init(&self) {
+    fn early_print_device_drivers(&self) -> &[&'static (dyn DeviceDriver + Sync)] {
+        &self.device_drivers[0..=1]
+    }
+
+    fn non_early_print_device_drivers(&self) -> &[&'static (dyn DeviceDriver + Sync)] {
+        &self.device_drivers[2..]
+    }
+
+    fn post_early_print_device_driver_init(&self) {
         // Configure PL011Uart's output pins.
         super::GPIO.map_pl011_uart();
     }

diff -uNr 13_exceptions_part2_peripheral_IRQs/src/bsp/raspberrypi/link.ld 14_virtual_mem_part2_mmio_remap/src/bsp/raspberrypi/link.ld
--- 13_exceptions_part2_peripheral_IRQs/src/bsp/raspberrypi/link.ld
+++ 14_virtual_mem_part2_mmio_remap/src/bsp/raspberrypi/link.ld
@@ -17,11 +17,6 @@
 SECTIONS
 {
     . =  __rpi_load_addr;
-                                        /*   ^             */
-                                        /*   | stack       */
-                                        /*   | growth      */
-                                        /*   | direction   */
-   __boot_core_stack_end_exclusive = .; /*   |             */

     /***********************************************************************************************
     * Code + RO Data + Global Offset Table
@@ -44,6 +39,7 @@
     /***********************************************************************************************
     * Data + BSS
     ***********************************************************************************************/
+    __rw_start = .;
     .data : { *(.data*) } :segment_rw

     /* Section is zeroed in u64 chunks, align start and end to 8 bytes */
@@ -56,4 +52,23 @@
         . += 8; /* Fill for the bss == 0 case, so that __bss_start <= __bss_end_inclusive holds */
         __bss_end_inclusive = . - 8;
     } :NONE
+
+    . = ALIGN(64K); /* Align to page boundary */
+    __rw_end_exclusive = .;
+
+    /***********************************************************************************************
+    * Guard Page between boot core stack and data
+    ***********************************************************************************************/
+    __boot_core_stack_guard_page_start = .;
+    . += 64K;
+    __boot_core_stack_guard_page_end_exclusive = .;
+
+    /***********************************************************************************************
+    * Boot Core Stack
+    ***********************************************************************************************/
+    __boot_core_stack_start = .;         /*   ^             */
+                                         /*   | stack       */
+    . += 512K;                           /*   | growth      */
+                                         /*   | direction   */
+    __boot_core_stack_end_exclusive = .; /*   |             */
 }

diff -uNr 13_exceptions_part2_peripheral_IRQs/src/bsp/raspberrypi/memory/mmu.rs 14_virtual_mem_part2_mmio_remap/src/bsp/raspberrypi/memory/mmu.rs
--- 13_exceptions_part2_peripheral_IRQs/src/bsp/raspberrypi/memory/mmu.rs
+++ 14_virtual_mem_part2_mmio_remap/src/bsp/raspberrypi/memory/mmu.rs
@@ -4,70 +4,164 @@

 //! BSP Memory Management Unit.

-use super::map as memory_map;
-use crate::memory::mmu::*;
-use core::ops::RangeInclusive;
+use crate::{
+    common,
+    memory::{
+        mmu as generic_mmu,
+        mmu::{
+            AccessPermissions, AddressSpace, AssociatedTranslationTable, AttributeFields,
+            MemAttributes, Page, PageSliceDescriptor, TranslationGranule,
+        },
+        Physical, Virtual,
+    },
+    synchronization::InitStateLock,
+};
+
+//--------------------------------------------------------------------------------------------------
+// Private Definitions
+//--------------------------------------------------------------------------------------------------
+
+type KernelTranslationTable =
+    <KernelVirtAddrSpace as AssociatedTranslationTable>::TableStartFromBottom;

 //--------------------------------------------------------------------------------------------------
 // Public Definitions
 //--------------------------------------------------------------------------------------------------

-/// The kernel's address space defined by this BSP.
-pub type KernelAddrSpace = AddressSpace<{ memory_map::END_INCLUSIVE + 1 }>;
+/// The translation granule chosen by this BSP. This will be used everywhere else in the kernel to
+/// derive respective data structures and their sizes. For example, the `crate::memory::mmu::Page`.
+pub type KernelGranule = TranslationGranule<{ 64 * 1024 }>;
+
+/// The kernel's virtual address space defined by this BSP.
+pub type KernelVirtAddrSpace = AddressSpace<{ 8 * 1024 * 1024 * 1024 }>;

-const NUM_MEM_RANGES: usize = 2;
+//--------------------------------------------------------------------------------------------------
+// Global instances
+//--------------------------------------------------------------------------------------------------

-/// The virtual memory layout.
+/// The kernel translation tables.
 ///
-/// The layout must contain only special ranges, aka anything that is _not_ normal cacheable DRAM.
-/// It is agnostic of the paging granularity that the architecture's MMU will use.
-pub static LAYOUT: KernelVirtualLayout<NUM_MEM_RANGES> = KernelVirtualLayout::new(
-    memory_map::END_INCLUSIVE,
-    [
-        TranslationDescriptor {
-            name: "Kernel code and RO data",
-            virtual_range: rx_range_inclusive,
-            physical_range_translation: Translation::Identity,
-            attribute_fields: AttributeFields {
-                mem_attributes: MemAttributes::CacheableDRAM,
-                acc_perms: AccessPermissions::ReadOnly,
-                execute_never: false,
-            },
-        },
-        TranslationDescriptor {
-            name: "Device MMIO",
-            virtual_range: mmio_range_inclusive,
-            physical_range_translation: Translation::Identity,
-            attribute_fields: AttributeFields {
-                mem_attributes: MemAttributes::Device,
-                acc_perms: AccessPermissions::ReadWrite,
-                execute_never: true,
-            },
-        },
-    ],
-);
+/// It is mandatory that InitStateLock is transparent.
+///
+/// That is, `size_of(InitStateLock<KernelTranslationTable>) == size_of(KernelTranslationTable)`.
+/// There is a unit tests that checks this porperty.
+static KERNEL_TABLES: InitStateLock<KernelTranslationTable> =
+    InitStateLock::new(KernelTranslationTable::new());

 //--------------------------------------------------------------------------------------------------
 // Private Code
 //--------------------------------------------------------------------------------------------------

-fn rx_range_inclusive() -> RangeInclusive<usize> {
-    // Notice the subtraction to turn the exclusive end into an inclusive end.
-    #[allow(clippy::range_minus_one)]
-    RangeInclusive::new(super::rx_start(), super::rx_end_exclusive() - 1)
+/// Helper function for calculating the number of pages the given parameter spans.
+const fn size_to_num_pages(size: usize) -> usize {
+    assert!(size > 0);
+    assert!(size modulo KernelGranule::SIZE == 0);
+
+    size >> KernelGranule::SHIFT
+}
+
+/// The Read+Execute (RX) pages of the kernel binary.
+fn virt_rx_page_desc() -> PageSliceDescriptor<Virtual> {
+    let num_pages = size_to_num_pages(super::rx_size());
+
+    PageSliceDescriptor::from_addr(super::virt_rx_start(), num_pages)
+}
+
+/// The Read+Write (RW) pages of the kernel binary.
+fn virt_rw_page_desc() -> PageSliceDescriptor<Virtual> {
+    let num_pages = size_to_num_pages(super::rw_size());
+
+    PageSliceDescriptor::from_addr(super::virt_rw_start(), num_pages)
+}
+
+/// The boot core's stack.
+fn virt_boot_core_stack_page_desc() -> PageSliceDescriptor<Virtual> {
+    let num_pages = size_to_num_pages(super::boot_core_stack_size());
+
+    PageSliceDescriptor::from_addr(super::virt_boot_core_stack_start(), num_pages)
+}
+
+// The binary is still identity mapped, so we don't need to convert in the following.
+
+/// The Read+Execute (RX) pages of the kernel binary.
+fn phys_rx_page_desc() -> PageSliceDescriptor<Physical> {
+    virt_rx_page_desc().into()
 }

-fn mmio_range_inclusive() -> RangeInclusive<usize> {
-    RangeInclusive::new(memory_map::mmio::START, memory_map::mmio::END_INCLUSIVE)
+/// The Read+Write (RW) pages of the kernel binary.
+fn phys_rw_page_desc() -> PageSliceDescriptor<Physical> {
+    virt_rw_page_desc().into()
+}
+
+/// The boot core's stack.
+fn phys_boot_core_stack_page_desc() -> PageSliceDescriptor<Physical> {
+    virt_boot_core_stack_page_desc().into()
 }

 //--------------------------------------------------------------------------------------------------
 // Public Code
 //--------------------------------------------------------------------------------------------------

-/// Return a reference to the virtual memory layout.
-pub fn virt_mem_layout() -> &'static KernelVirtualLayout<NUM_MEM_RANGES> {
-    &LAYOUT
+/// Return a reference to the kernel's translation tables.
+pub fn kernel_translation_tables() -> &'static InitStateLock<KernelTranslationTable> {
+    &KERNEL_TABLES
+}
+
+/// The boot core's stack guard page.
+pub fn virt_boot_core_stack_guard_page_desc() -> PageSliceDescriptor<Virtual> {
+    let num_pages = size_to_num_pages(super::boot_core_stack_guard_page_size());
+
+    PageSliceDescriptor::from_addr(super::virt_boot_core_stack_guard_page_start(), num_pages)
+}
+
+/// Pointer to the last page of the physical address space.
+pub fn phys_addr_space_end_page() -> *const Page<Physical> {
+    common::align_down(
+        super::phys_addr_space_end().into_usize(),
+        KernelGranule::SIZE,
+    ) as *const Page<_>
+}
+
+/// Map the kernel binary.
+///
+/// # Safety
+///
+/// - Any miscalculation or attribute error will likely be fatal. Needs careful manual checking.
+pub unsafe fn kernel_map_binary() -> Result<(), &'static str> {
+    generic_mmu::kernel_map_pages_at(
+        "Kernel code and RO data",
+        &virt_rx_page_desc(),
+        &phys_rx_page_desc(),
+        &AttributeFields {
+            mem_attributes: MemAttributes::CacheableDRAM,
+            acc_perms: AccessPermissions::ReadOnly,
+            execute_never: false,
+        },
+    )?;
+
+    generic_mmu::kernel_map_pages_at(
+        "Kernel data and bss",
+        &virt_rw_page_desc(),
+        &phys_rw_page_desc(),
+        &AttributeFields {
+            mem_attributes: MemAttributes::CacheableDRAM,
+            acc_perms: AccessPermissions::ReadWrite,
+            execute_never: true,
+        },
+    )?;
+
+    generic_mmu::kernel_map_pages_at(
+        "Kernel boot-core stack",
+        &virt_boot_core_stack_page_desc(),
+        &phys_boot_core_stack_page_desc(),
+        &AttributeFields {
+            mem_attributes: MemAttributes::CacheableDRAM,
+            acc_perms: AccessPermissions::ReadWrite,
+            execute_never: true,
+        },
+    )?;
+
+    Ok(())
 }

 //--------------------------------------------------------------------------------------------------
@@ -82,14 +176,18 @@
     /// Check alignment of the kernel's virtual memory layout sections.
     #[kernel_test]
     fn virt_mem_layout_sections_are_64KiB_aligned() {
-        const SIXTYFOUR_KIB: usize = 65536;
-
-        for i in LAYOUT.inner().iter() {
-            let start: usize = *(i.virtual_range)().start();
-            let end: usize = *(i.virtual_range)().end() + 1;
+        for i in [
+            virt_rx_page_desc,
+            virt_rw_page_desc,
+            virt_boot_core_stack_page_desc,
+        ]
+        .iter()
+        {
+            let start: usize = i().start_addr().into_usize();
+            let end: usize = i().end_addr().into_usize();

-            assert_eq!(start modulo SIXTYFOUR_KIB, 0);
-            assert_eq!(end modulo SIXTYFOUR_KIB, 0);
+            assert_eq!(start modulo KernelGranule::SIZE, 0);
+            assert_eq!(end modulo KernelGranule::SIZE, 0);
             assert!(end >= start);
         }
     }
@@ -97,18 +195,28 @@
     /// Ensure the kernel's virtual memory layout is free of overlaps.
     #[kernel_test]
     fn virt_mem_layout_has_no_overlaps() {
-        let layout = virt_mem_layout().inner();
-
-        for (i, first) in layout.iter().enumerate() {
-            for second in layout.iter().skip(i + 1) {
-                let first_range = first.virtual_range;
-                let second_range = second.virtual_range;
-
-                assert!(!first_range().contains(second_range().start()));
-                assert!(!first_range().contains(second_range().end()));
-                assert!(!second_range().contains(first_range().start()));
-                assert!(!second_range().contains(first_range().end()));
+        let layout = [
+            virt_rx_page_desc(),
+            virt_rw_page_desc(),
+            virt_boot_core_stack_page_desc(),
+        ];
+
+        for (i, first_range) in layout.iter().enumerate() {
+            for second_range in layout.iter().skip(i + 1) {
+                assert!(!first_range.contains(second_range.start_addr()));
+                assert!(!first_range.contains(second_range.end_addr_inclusive()));
+                assert!(!second_range.contains(first_range.start_addr()));
+                assert!(!second_range.contains(first_range.end_addr_inclusive()));
             }
         }
     }
+
+    /// Check if KERNEL_TABLES is in .bss.
+    #[kernel_test]
+    fn kernel_tables_in_bss() {
+        let bss_range = super::super::bss_range_inclusive();
+        let kernel_tables_addr = &KERNEL_TABLES as *const _ as usize as *mut u64;
+
+        assert!(bss_range.contains(&kernel_tables_addr));
+    }
 }

diff -uNr 13_exceptions_part2_peripheral_IRQs/src/bsp/raspberrypi/memory.rs 14_virtual_mem_part2_mmio_remap/src/bsp/raspberrypi/memory.rs
--- 13_exceptions_part2_peripheral_IRQs/src/bsp/raspberrypi/memory.rs
+++ 14_virtual_mem_part2_mmio_remap/src/bsp/raspberrypi/memory.rs
@@ -3,9 +3,40 @@
 // Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

 //! BSP Memory Management.
+//!
+//! The physical memory layout after the kernel has been loaded by the Raspberry's firmware, which
+//! copies the binary to 0x8_0000:
+//!
+//! +---------------------------------------------+
+//! |                                             |
+//! | Unmapped                                    |
+//! |                                             |
+//! +---------------------------------------------+
+//! |                                             | rx_start @ 0x8_0000
+//! | .text                                       |
+//! | .rodata                                     |
+//! | .got                                        |
+//! |                                             | rx_end_inclusive
+//! +---------------------------------------------+
+//! |                                             | rw_start == rx_end
+//! | .data                                       |
+//! | .bss                                        |
+//! |                                             | rw_end_inclusive
+//! +---------------------------------------------+
+//! |                                             | rw_end
+//! | Unmapped Boot-core Stack Guard Page         |
+//! |                                             |
+//! +---------------------------------------------+
+//! |                                             | boot_core_stack_start          ^
+//! |                                             |                                | stack
+//! | Boot-core Stack                             |                                | growth
+//! |                                             |                                | direction
+//! |                                             | boot_core_stack_end_inclusive  |
+//! +---------------------------------------------+

 pub mod mmu;

+use crate::memory::{Address, Physical, Virtual};
 use core::{cell::UnsafeCell, ops::RangeInclusive};

 //--------------------------------------------------------------------------------------------------
@@ -17,8 +48,16 @@
     static __rx_start: UnsafeCell<()>;
     static __rx_end_exclusive: UnsafeCell<()>;

+    static __rw_start: UnsafeCell<()>;
     static __bss_start: UnsafeCell<u64>;
     static __bss_end_inclusive: UnsafeCell<u64>;
+    static __rw_end_exclusive: UnsafeCell<()>;
+
+    static __boot_core_stack_start: UnsafeCell<()>;
+    static __boot_core_stack_end_exclusive: UnsafeCell<()>;
+
+    static __boot_core_stack_guard_page_start: UnsafeCell<()>;
+    static __boot_core_stack_guard_page_end_exclusive: UnsafeCell<()>;
 }

 //--------------------------------------------------------------------------------------------------
@@ -28,35 +67,26 @@
 /// The board's physical memory map.
 #[rustfmt::skip]
 pub(super) mod map {
-    /// The inclusive end address of the memory map.
-    ///
-    /// End address + 1 must be power of two.
-    ///
-    /// # Note
-    ///
-    /// RPi3 and RPi4 boards can have different amounts of RAM. To make our code lean for
-    /// educational purposes, we set the max size of the address space to 4 GiB regardless of board.
-    /// This way, we can map the entire range that we need (end of MMIO for RPi4) in one take.
-    ///
-    /// However, making this trade-off has the downside of making it possible for the CPU to assert a
-    /// physical address that is not backed by any DRAM (e.g. accessing an address close to 4 GiB on
-    /// an RPi3 that comes with 1 GiB of RAM). This would result in a crash or other kind of error.
-    pub const END_INCLUSIVE:       usize = 0xFFFF_FFFF;
-
-    pub const GPIO_OFFSET:         usize = 0x0020_0000;
-    pub const UART_OFFSET:         usize = 0x0020_1000;
+    use super::*;

     /// Physical devices.
     #[cfg(feature = "bsp_rpi3")]
     pub mod mmio {
         use super::*;

-        pub const START:                                 usize =         0x3F00_0000;
-        pub const PERIPHERAL_INTERRUPT_CONTROLLER_START: usize = START + 0x0000_B200;
-        pub const GPIO_START:                            usize = START + GPIO_OFFSET;
-        pub const PL011_UART_START:                      usize = START + UART_OFFSET;
-        pub const LOCAL_INTERRUPT_CONTROLLER_START:      usize =         0x4000_0000;
-        pub const END_INCLUSIVE:                         usize =         0x4000_FFFF;
+        pub const PERIPHERAL_IC_START: Address<Physical> = Address::new(0x3F00_B200);
+        pub const PERIPHERAL_IC_SIZE:  usize             =              0x24;
+
+        pub const GPIO_START:          Address<Physical> = Address::new(0x3F20_0000);
+        pub const GPIO_SIZE:           usize             =              0xA0;
+
+        pub const PL011_UART_START:    Address<Physical> = Address::new(0x3F20_1000);
+        pub const PL011_UART_SIZE:     usize             =              0x48;
+
+        pub const LOCAL_IC_START:      Address<Physical> = Address::new(0x4000_0000);
+        pub const LOCAL_IC_SIZE:       usize             =              0x100;
+
+        pub const END:                 Address<Physical> = Address::new(0x4001_0000);
     }

     /// Physical devices.
@@ -64,13 +94,22 @@
     pub mod mmio {
         use super::*;

-        pub const START:            usize =         0xFE00_0000;
-        pub const GPIO_START:       usize = START + GPIO_OFFSET;
-        pub const PL011_UART_START: usize = START + UART_OFFSET;
-        pub const GICD_START:       usize =         0xFF84_1000;
-        pub const GICC_START:       usize =         0xFF84_2000;
-        pub const END_INCLUSIVE:    usize =         0xFF84_FFFF;
+        pub const GPIO_START:       Address<Physical> = Address::new(0xFE20_0000);
+        pub const GPIO_SIZE:        usize             =              0xA0;
+
+        pub const PL011_UART_START: Address<Physical> = Address::new(0xFE20_1000);
+        pub const PL011_UART_SIZE:  usize             =              0x48;
+
+        pub const GICD_START:       Address<Physical> = Address::new(0xFF84_1000);
+        pub const GICD_SIZE:        usize             =              0x824;
+
+        pub const GICC_START:       Address<Physical> = Address::new(0xFF84_2000);
+        pub const GICC_SIZE:        usize             =              0x14;
+
+        pub const END:              Address<Physical> = Address::new(0xFF85_0000);
     }
+
+    pub const END: Address<Physical> = mmio::END;
 }

 //--------------------------------------------------------------------------------------------------
@@ -83,18 +122,69 @@
 ///
 /// - Value is provided by the linker script and must be trusted as-is.
 #[inline(always)]
-fn rx_start() -> usize {
-    unsafe { __rx_start.get() as usize }
+fn virt_rx_start() -> Address<Virtual> {
+    Address::new(unsafe { __rx_start.get() as usize })
 }

-/// Exclusive end address of the Read+Execute (RX) range.
+/// Size of the Read+Execute (RX) range.
 ///
 /// # Safety
 ///
 /// - Value is provided by the linker script and must be trusted as-is.
 #[inline(always)]
-fn rx_end_exclusive() -> usize {
-    unsafe { __rx_end_exclusive.get() as usize }
+fn rx_size() -> usize {
+    unsafe { (__rx_end_exclusive.get() as usize) - (__rx_start.get() as usize) }
+}
+
+/// Start address of the Read+Write (RW) range.
+#[inline(always)]
+fn virt_rw_start() -> Address<Virtual> {
+    Address::new(unsafe { __rw_start.get() as usize })
+}
+
+/// Size of the Read+Write (RW) range.
+///
+/// # Safety
+///
+/// - Value is provided by the linker script and must be trusted as-is.
+#[inline(always)]
+fn rw_size() -> usize {
+    unsafe { (__rw_end_exclusive.get() as usize) - (__rw_start.get() as usize) }
+}
+
+/// Start address of the boot core's stack.
+#[inline(always)]
+fn virt_boot_core_stack_start() -> Address<Virtual> {
+    Address::new(unsafe { __boot_core_stack_start.get() as usize })
+}
+
+/// Size of the boot core's stack.
+#[inline(always)]
+fn boot_core_stack_size() -> usize {
+    unsafe {
+        (__boot_core_stack_end_exclusive.get() as usize) - (__boot_core_stack_start.get() as usize)
+    }
+}
+
+/// Start address of the boot core's stack guard page.
+#[inline(always)]
+fn virt_boot_core_stack_guard_page_start() -> Address<Virtual> {
+    Address::new(unsafe { __boot_core_stack_guard_page_start.get() as usize })
+}
+
+/// Size of the boot core's stack guard page.
+#[inline(always)]
+fn boot_core_stack_guard_page_size() -> usize {
+    unsafe {
+        (__boot_core_stack_guard_page_end_exclusive.get() as usize)
+            - (__boot_core_stack_guard_page_start.get() as usize)
+    }
+}
+
+/// Exclusive end address of the physical address space.
+#[inline(always)]
+fn phys_addr_space_end() -> Address<Physical> {
+    map::END
 }

 //--------------------------------------------------------------------------------------------------

diff -uNr 13_exceptions_part2_peripheral_IRQs/src/bsp/raspberrypi.rs 14_virtual_mem_part2_mmio_remap/src/bsp/raspberrypi.rs
--- 13_exceptions_part2_peripheral_IRQs/src/bsp/raspberrypi.rs
+++ 14_virtual_mem_part2_mmio_remap/src/bsp/raspberrypi.rs
@@ -10,17 +10,20 @@
 pub mod exception;
 pub mod memory;

+use super::device_driver;
+use crate::memory::mmu::MMIODescriptor;
+use memory::map::mmio;
+
 //--------------------------------------------------------------------------------------------------
 // Global instances
 //--------------------------------------------------------------------------------------------------
-use super::device_driver;

 static GPIO: device_driver::GPIO =
-    unsafe { device_driver::GPIO::new(memory::map::mmio::GPIO_START) };
+    unsafe { device_driver::GPIO::new(MMIODescriptor::new(mmio::GPIO_START, mmio::GPIO_SIZE)) };

 static PL011_UART: device_driver::PL011Uart = unsafe {
     device_driver::PL011Uart::new(
-        memory::map::mmio::PL011_UART_START,
+        MMIODescriptor::new(mmio::PL011_UART_START, mmio::PL011_UART_SIZE),
         exception::asynchronous::irq_map::PL011_UART,
     )
 };
@@ -28,14 +31,17 @@
 #[cfg(feature = "bsp_rpi3")]
 static INTERRUPT_CONTROLLER: device_driver::InterruptController = unsafe {
     device_driver::InterruptController::new(
-        memory::map::mmio::LOCAL_INTERRUPT_CONTROLLER_START,
-        memory::map::mmio::PERIPHERAL_INTERRUPT_CONTROLLER_START,
+        MMIODescriptor::new(mmio::LOCAL_IC_START, mmio::LOCAL_IC_SIZE),
+        MMIODescriptor::new(mmio::PERIPHERAL_IC_START, mmio::PERIPHERAL_IC_SIZE),
     )
 };

 #[cfg(feature = "bsp_rpi4")]
 static INTERRUPT_CONTROLLER: device_driver::GICv2 = unsafe {
-    device_driver::GICv2::new(memory::map::mmio::GICD_START, memory::map::mmio::GICC_START)
+    device_driver::GICv2::new(
+        MMIODescriptor::new(mmio::GICD_START, mmio::GICD_SIZE),
+        MMIODescriptor::new(mmio::GICC_START, mmio::GICC_SIZE),
+    )
 };

 //--------------------------------------------------------------------------------------------------

diff -uNr 13_exceptions_part2_peripheral_IRQs/src/common.rs 14_virtual_mem_part2_mmio_remap/src/common.rs
--- 13_exceptions_part2_peripheral_IRQs/src/common.rs
+++ 14_virtual_mem_part2_mmio_remap/src/common.rs
@@ -0,0 +1,21 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2020-2021 Andre Richter <andre.o.richter@gmail.com>
+
+//! General purpose code.
+
+/// Check if a value is aligned to a given size.
+#[inline(always)]
+pub const fn is_aligned(value: usize, alignment: usize) -> bool {
+    assert!(alignment.is_power_of_two());
+
+    (value & (alignment - 1)) == 0
+}
+
+/// Align down.
+#[inline(always)]
+pub const fn align_down(value: usize, alignment: usize) -> usize {
+    assert!(alignment.is_power_of_two());
+
+    value & !(alignment - 1)
+}

diff -uNr 13_exceptions_part2_peripheral_IRQs/src/driver.rs 14_virtual_mem_part2_mmio_remap/src/driver.rs
--- 13_exceptions_part2_peripheral_IRQs/src/driver.rs
+++ 14_virtual_mem_part2_mmio_remap/src/driver.rs
@@ -31,6 +31,14 @@
         fn register_and_enable_irq_handler(&'static self) -> Result<(), &'static str> {
             Ok(())
         }
+
+        /// After MMIO remapping, returns the new virtual start address.
+        ///
+        /// This API assumes a driver has only a single, contiguous MMIO aperture, which will not be
+        /// the case for more complex devices. This API will likely change in future tutorials.
+        fn virt_mmio_start_addr(&self) -> Option<usize> {
+            None
+        }
     }

     /// Device driver management functions.
@@ -38,15 +46,17 @@
     /// The `BSP` is supposed to supply one global instance.
     pub trait DriverManager {
         /// Return a slice of references to all `BSP`-instantiated drivers.
-        ///
-        /// # Safety
-        ///
-        /// - The order of devices is the order in which `DeviceDriver::init()` is called.
         fn all_device_drivers(&self) -> &[&'static (dyn DeviceDriver + Sync)];

-        /// Initialization code that runs after driver init.
+        /// Return only those drivers needed for the BSP's early printing functionality.
         ///
-        /// For example, device driver code that depends on other drivers already being online.
-        fn post_device_driver_init(&self);
+        /// For example, the default UART.
+        fn early_print_device_drivers(&self) -> &[&'static (dyn DeviceDriver + Sync)];
+
+        /// Return all drivers minus early-print drivers.
+        fn non_early_print_device_drivers(&self) -> &[&'static (dyn DeviceDriver + Sync)];
+
+        /// Initialization code that runs after the early print driver init.
+        fn post_early_print_device_driver_init(&self);
     }
 }

diff -uNr 13_exceptions_part2_peripheral_IRQs/src/lib.rs 14_virtual_mem_part2_mmio_remap/src/lib.rs
--- 13_exceptions_part2_peripheral_IRQs/src/lib.rs
+++ 14_virtual_mem_part2_mmio_remap/src/lib.rs
@@ -111,6 +111,8 @@
 #![allow(clippy::upper_case_acronyms)]
 #![allow(incomplete_features)]
 #![feature(asm)]
+#![feature(const_evaluatable_checked)]
+#![feature(const_fn)]
 #![feature(const_fn_fn_ptr_basics)]
 #![feature(const_generics)]
 #![feature(const_panic)]
@@ -132,6 +134,7 @@
 mod synchronization;

 pub mod bsp;
+pub mod common;
 pub mod console;
 pub mod cpu;
 pub mod driver;

diff -uNr 13_exceptions_part2_peripheral_IRQs/src/main.rs 14_virtual_mem_part2_mmio_remap/src/main.rs
--- 13_exceptions_part2_peripheral_IRQs/src/main.rs
+++ 14_virtual_mem_part2_mmio_remap/src/main.rs
@@ -25,21 +25,39 @@
 #[no_mangle]
 unsafe fn kernel_init() -> ! {
     use driver::interface::DriverManager;
-    use memory::mmu::interface::MMU;

     exception::handling_init();

-    if let Err(string) = memory::mmu::mmu().enable_mmu_and_caching() {
-        panic!("MMU: {}", string);
+    let phys_kernel_tables_base_addr = match memory::mmu::kernel_map_binary() {
+        Err(string) => panic!("Error mapping kernel binary: {}", string),
+        Ok(addr) => addr,
+    };
+
+    if let Err(e) = memory::mmu::enable_mmu_and_caching(phys_kernel_tables_base_addr) {
+        panic!("Enabling MMU failed: {}", e);
+    }
+    // Printing will silently fail from here on, because the driver's MMIO is not remapped yet.
+
+    // Bring up the drivers needed for printing first.
+    for i in bsp::driver::driver_manager()
+        .early_print_device_drivers()
+        .iter()
+    {
+        // Any encountered errors cannot be printed yet, obviously, so just safely park the CPU.
+        i.init().unwrap_or_else(|_| cpu::wait_forever());
     }
+    bsp::driver::driver_manager().post_early_print_device_driver_init();
+    // Printing available again from here on.

-    for i in bsp::driver::driver_manager().all_device_drivers().iter() {
+    // Now bring up the remaining drivers.
+    for i in bsp::driver::driver_manager()
+        .non_early_print_device_drivers()
+        .iter()
+    {
         if let Err(x) = i.init() {
             panic!("Error loading driver: {}: {}", i.compatible(), x);
         }
     }
-    bsp::driver::driver_manager().post_device_driver_init();
-    // println! is usable from here on.

     // Let device drivers register and enable their handlers with the interrupt controller.
     for i in bsp::driver::driver_manager().all_device_drivers() {
@@ -66,8 +84,8 @@
     info!("{}", libkernel::version());
     info!("Booting on: {}", bsp::board_name());

-    info!("MMU online. Special regions:");
-    bsp::memory::mmu::virt_mem_layout().print_layout();
+    info!("MMU online:");
+    memory::mmu::kernel_print_mappings();

     let (_, privilege_level) = exception::current_privilege_level();
     info!("Current privilege level: {}", privilege_level);

diff -uNr 13_exceptions_part2_peripheral_IRQs/src/memory/mmu/mapping_record.rs 14_virtual_mem_part2_mmio_remap/src/memory/mmu/mapping_record.rs
--- 13_exceptions_part2_peripheral_IRQs/src/memory/mmu/mapping_record.rs
+++ 14_virtual_mem_part2_mmio_remap/src/memory/mmu/mapping_record.rs
@@ -0,0 +1,216 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2020-2021 Andre Richter <andre.o.richter@gmail.com>
+
+//! A record of mapped pages.
+
+use super::{
+    AccessPermissions, Address, AttributeFields, MMIODescriptor, MemAttributes,
+    PageSliceDescriptor, Physical, Virtual,
+};
+use crate::{info, synchronization, synchronization::InitStateLock, warn};
+
+//--------------------------------------------------------------------------------------------------
+// Private Definitions
+//--------------------------------------------------------------------------------------------------
+
+/// Type describing a virtual memory mapping.
+#[allow(missing_docs)]
+#[derive(Copy, Clone)]
+struct MappingRecordEntry {
+    pub users: [Option<&'static str>; 5],
+    pub phys_pages: PageSliceDescriptor<Physical>,
+    pub virt_start_addr: Address<Virtual>,
+    pub attribute_fields: AttributeFields,
+}
+
+struct MappingRecord {
+    inner: [Option<MappingRecordEntry>; 12],
+}
+
+//--------------------------------------------------------------------------------------------------
+// Global instances
+//--------------------------------------------------------------------------------------------------
+
+static KERNEL_MAPPING_RECORD: InitStateLock<MappingRecord> =
+    InitStateLock::new(MappingRecord::new());
+
+//--------------------------------------------------------------------------------------------------
+// Private Code
+//--------------------------------------------------------------------------------------------------
+
+impl MappingRecordEntry {
+    pub fn new(
+        name: &'static str,
+        virt_pages: &PageSliceDescriptor<Virtual>,
+        phys_pages: &PageSliceDescriptor<Physical>,
+        attr: &AttributeFields,
+    ) -> Self {
+        Self {
+            users: [Some(name), None, None, None, None],
+            phys_pages: *phys_pages,
+            virt_start_addr: virt_pages.start_addr(),
+            attribute_fields: *attr,
+        }
+    }
+
+    fn find_next_free_user(&mut self) -> Result<&mut Option<&'static str>, &'static str> {
+        if let Some(x) = self.users.iter_mut().find(|x| x.is_none()) {
+            return Ok(x);
+        };
+
+        Err("Storage for user info exhausted")
+    }
+
+    pub fn add_user(&mut self, user: &'static str) -> Result<(), &'static str> {
+        let x = self.find_next_free_user()?;
+        *x = Some(user);
+        Ok(())
+    }
+}
+
+impl MappingRecord {
+    pub const fn new() -> Self {
+        Self { inner: [None; 12] }
+    }
+
+    fn find_next_free(&mut self) -> Result<&mut Option<MappingRecordEntry>, &'static str> {
+        if let Some(x) = self.inner.iter_mut().find(|x| x.is_none()) {
+            return Ok(x);
+        }
+
+        Err("Storage for mapping info exhausted")
+    }
+
+    fn find_duplicate(
+        &mut self,
+        phys_pages: &PageSliceDescriptor<Physical>,
+    ) -> Option<&mut MappingRecordEntry> {
+        self.inner
+            .iter_mut()
+            .filter(|x| x.is_some())
+            .map(|x| x.as_mut().unwrap())
+            .filter(|x| x.attribute_fields.mem_attributes == MemAttributes::Device)
+            .find(|x| x.phys_pages == *phys_pages)
+    }
+
+    pub fn add(
+        &mut self,
+        name: &'static str,
+        virt_pages: &PageSliceDescriptor<Virtual>,
+        phys_pages: &PageSliceDescriptor<Physical>,
+        attr: &AttributeFields,
+    ) -> Result<(), &'static str> {
+        let x = self.find_next_free()?;
+
+        *x = Some(MappingRecordEntry::new(name, virt_pages, phys_pages, attr));
+        Ok(())
+    }
+
+    pub fn print(&self) {
+        const KIB_RSHIFT: u32 = 10; // log2(1024).
+        const MIB_RSHIFT: u32 = 20; // log2(1024 * 1024).
+
+        info!("      -------------------------------------------------------------------------------------------------------------------------------------------");
+        info!(
+            "      {:^44}     {:^30}   {:^7}   {:^9}   {:^35}",
+            "Virtual", "Physical", "Size", "Attr", "Entity"
+        );
+        info!("      -------------------------------------------------------------------------------------------------------------------------------------------");
+
+        for i in self.inner.iter().flatten() {
+            let virt_start = i.virt_start_addr;
+            let virt_end_inclusive = virt_start + i.phys_pages.size() - 1;
+            let phys_start = i.phys_pages.start_addr();
+            let phys_end_inclusive = i.phys_pages.end_addr_inclusive();
+            let size = i.phys_pages.size();
+
+            let (size, unit) = if (size >> MIB_RSHIFT) > 0 {
+                (size >> MIB_RSHIFT, "MiB")
+            } else if (size >> KIB_RSHIFT) > 0 {
+                (size >> KIB_RSHIFT, "KiB")
+            } else {
+                (size, "Byte")
+            };
+
+            let attr = match i.attribute_fields.mem_attributes {
+                MemAttributes::CacheableDRAM => "C",
+                MemAttributes::Device => "Dev",
+            };
+
+            let acc_p = match i.attribute_fields.acc_perms {
+                AccessPermissions::ReadOnly => "RO",
+                AccessPermissions::ReadWrite => "RW",
+            };
+
+            let xn = if i.attribute_fields.execute_never {
+                "XN"
+            } else {
+                "X"
+            };
+
+            info!(
+                "      {}..{} --> {}..{} | \
+                        {: >3} {} | {: <3} {} {: <2} | {}",
+                virt_start,
+                virt_end_inclusive,
+                phys_start,
+                phys_end_inclusive,
+                size,
+                unit,
+                attr,
+                acc_p,
+                xn,
+                i.users[0].unwrap()
+            );
+
+            for k in i.users[1..].iter() {
+                if let Some(additional_user) = *k {
+                    info!(
+                        "                                                                                                            | {}",
+                        additional_user
+                    );
+                }
+            }
+        }
+
+        info!("      -------------------------------------------------------------------------------------------------------------------------------------------");
+    }
+}
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+use synchronization::interface::ReadWriteEx;
+
+/// Add an entry to the mapping info record.
+pub fn kernel_add(
+    name: &'static str,
+    virt_pages: &PageSliceDescriptor<Virtual>,
+    phys_pages: &PageSliceDescriptor<Physical>,
+    attr: &AttributeFields,
+) -> Result<(), &'static str> {
+    KERNEL_MAPPING_RECORD.write(|mr| mr.add(name, virt_pages, phys_pages, attr))
+}
+
+pub fn kernel_find_and_insert_mmio_duplicate(
+    mmio_descriptor: &MMIODescriptor,
+    new_user: &'static str,
+) -> Option<Address<Virtual>> {
+    let phys_pages: PageSliceDescriptor<Physical> = (*mmio_descriptor).into();
+
+    KERNEL_MAPPING_RECORD.write(|mr| {
+        let dup = mr.find_duplicate(&phys_pages)?;
+
+        if let Err(x) = dup.add_user(new_user) {
+            warn!("{}", x);
+        }
+
+        Some(dup.virt_start_addr)
+    })
+}
+
+/// Human-readable print of all recorded kernel mappings.
+pub fn kernel_print() {
+    KERNEL_MAPPING_RECORD.read(|mr| mr.print());
+}

diff -uNr 13_exceptions_part2_peripheral_IRQs/src/memory/mmu/translation_table.rs 14_virtual_mem_part2_mmio_remap/src/memory/mmu/translation_table.rs
--- 13_exceptions_part2_peripheral_IRQs/src/memory/mmu/translation_table.rs
+++ 14_virtual_mem_part2_mmio_remap/src/memory/mmu/translation_table.rs
@@ -8,7 +8,105 @@
 #[path = "../../_arch/aarch64/memory/mmu/translation_table.rs"]
 mod arch_translation_table;

+use crate::memory::{
+    mmu::{AttributeFields, PageSliceDescriptor},
+    Address, Physical, Virtual,
+};
+
 //--------------------------------------------------------------------------------------------------
 // Architectural Public Reexports
 //--------------------------------------------------------------------------------------------------
-pub use arch_translation_table::KernelTranslationTable;
+#[cfg(target_arch = "aarch64")]
+pub use arch_translation_table::FixedSizeTranslationTable;
+
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
+/// Translation table interfaces.
+pub mod interface {
+    use super::*;
+
+    /// Translation table operations.
+    pub trait TranslationTable {
+        /// Anything that needs to run before any of the other provided functions can be used.
+        ///
+        /// # Safety
+        ///
+        /// - Implementor must ensure that this function can run only once or is harmless if invoked
+        ///   multiple times.
+        fn init(&mut self);
+
+        /// The translation table's base address to be used for programming the MMU.
+        fn phys_base_address(&self) -> Address<Physical>;
+
+        /// Map the given virtual pages to the given physical pages.
+        ///
+        /// # Safety
+        ///
+        /// - Using wrong attributes can cause multiple issues of different nature in the system.
+        /// - It is not required that the architectural implementation prevents aliasing. That is,
+        ///   mapping to the same physical memory using multiple virtual addresses, which would
+        ///   break Rust's ownership assumptions. This should be protected against in the kernel's
+        ///   generic MMU code.
+        unsafe fn map_pages_at(
+            &mut self,
+            virt_pages: &PageSliceDescriptor<Virtual>,
+            phys_pages: &PageSliceDescriptor<Physical>,
+            attr: &AttributeFields,
+        ) -> Result<(), &'static str>;
+
+        /// Obtain a free virtual page slice in the MMIO region.
+        ///
+        /// The "MMIO region" is a distinct region of the implementor's choice, which allows
+        /// differentiating MMIO addresses from others. This can speed up debugging efforts.
+        /// Ideally, those MMIO addresses are also standing out visually so that a human eye can
+        /// identify them. For example, by allocating them from near the end of the virtual address
+        /// space.
+        fn next_mmio_virt_page_slice(
+            &mut self,
+            num_pages: usize,
+        ) -> Result<PageSliceDescriptor<Virtual>, &'static str>;
+
+        /// Check if a virtual page splice is in the "MMIO region".
+        fn is_virt_page_slice_mmio(&self, virt_pages: &PageSliceDescriptor<Virtual>) -> bool;
+    }
+}
+
+//--------------------------------------------------------------------------------------------------
+// Testing
+//--------------------------------------------------------------------------------------------------
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::bsp;
+    use arch_translation_table::MinSizeTranslationTable;
+    use interface::TranslationTable;
+    use test_macros::kernel_test;
+
+    /// Sanity checks for the TranslationTable implementation.
+    #[kernel_test]
+    fn translationtable_implementation_sanity() {
+        // This will occupy a lot of space on the stack.
+        let mut tables = MinSizeTranslationTable::new();
+
+        tables.init();
+
+        let x = tables.next_mmio_virt_page_slice(0);
+        assert!(x.is_err());
+
+        let x = tables.next_mmio_virt_page_slice(1_0000_0000);
+        assert!(x.is_err());
+
+        let x = tables.next_mmio_virt_page_slice(2).unwrap();
+        assert_eq!(x.size(), bsp::memory::mmu::KernelGranule::SIZE * 2);
+
+        assert_eq!(tables.is_virt_page_slice_mmio(&x), true);
+
+        assert_eq!(
+            tables.is_virt_page_slice_mmio(&PageSliceDescriptor::from_addr(Address::new(0), 1)),
+            false
+        );
+    }
+}

diff -uNr 13_exceptions_part2_peripheral_IRQs/src/memory/mmu/types.rs 14_virtual_mem_part2_mmio_remap/src/memory/mmu/types.rs
--- 13_exceptions_part2_peripheral_IRQs/src/memory/mmu/types.rs
+++ 14_virtual_mem_part2_mmio_remap/src/memory/mmu/types.rs
@@ -0,0 +1,210 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2020-2021 Andre Richter <andre.o.richter@gmail.com>
+
+//! Memory Management Unit types.
+
+use crate::{
+    bsp, common,
+    memory::{Address, AddressType, Physical, Virtual},
+};
+use core::{convert::From, marker::PhantomData};
+
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
+/// Generic page type.
+#[repr(C)]
+pub struct Page<ATYPE: AddressType> {
+    inner: [u8; bsp::memory::mmu::KernelGranule::SIZE],
+    _address_type: PhantomData<ATYPE>,
+}
+
+/// Type describing a slice of pages.
+#[derive(Copy, Clone, PartialOrd, PartialEq)]
+pub struct PageSliceDescriptor<ATYPE: AddressType> {
+    start: Address<ATYPE>,
+    num_pages: usize,
+}
+
+/// Architecture agnostic memory attributes.
+#[allow(missing_docs)]
+#[derive(Copy, Clone, PartialOrd, PartialEq)]
+pub enum MemAttributes {
+    CacheableDRAM,
+    Device,
+}
+
+/// Architecture agnostic access permissions.
+#[allow(missing_docs)]
+#[derive(Copy, Clone)]
+pub enum AccessPermissions {
+    ReadOnly,
+    ReadWrite,
+}
+
+/// Collection of memory attributes.
+#[allow(missing_docs)]
+#[derive(Copy, Clone)]
+pub struct AttributeFields {
+    pub mem_attributes: MemAttributes,
+    pub acc_perms: AccessPermissions,
+    pub execute_never: bool,
+}
+
+/// An MMIO descriptor for use in device drivers.
+#[derive(Copy, Clone)]
+pub struct MMIODescriptor {
+    start_addr: Address<Physical>,
+    size: usize,
+}
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+//------------------------------------------------------------------------------
+// Page
+//------------------------------------------------------------------------------
+
+impl<ATYPE: AddressType> Page<ATYPE> {
+    /// Get a pointer to the instance.
+    pub const fn as_ptr(&self) -> *const Page<ATYPE> {
+        self as *const _
+    }
+}
+
+//------------------------------------------------------------------------------
+// PageSliceDescriptor
+//------------------------------------------------------------------------------
+
+impl<ATYPE: AddressType> PageSliceDescriptor<ATYPE> {
+    /// Create an instance.
+    pub const fn from_addr(start: Address<ATYPE>, num_pages: usize) -> Self {
+        assert!(common::is_aligned(
+            start.into_usize(),
+            bsp::memory::mmu::KernelGranule::SIZE
+        ));
+        assert!(num_pages > 0);
+
+        Self { start, num_pages }
+    }
+
+    /// Return a pointer to the first page of the described slice.
+    const fn first_page_ptr(&self) -> *const Page<ATYPE> {
+        self.start.into_usize() as *const _
+    }
+
+    /// Return the number of Pages the slice describes.
+    pub const fn num_pages(&self) -> usize {
+        self.num_pages
+    }
+
+    /// Return the memory size this descriptor spans.
+    pub const fn size(&self) -> usize {
+        self.num_pages * bsp::memory::mmu::KernelGranule::SIZE
+    }
+
+    /// Return the start address.
+    pub const fn start_addr(&self) -> Address<ATYPE> {
+        self.start
+    }
+
+    /// Return the exclusive end address.
+    pub fn end_addr(&self) -> Address<ATYPE> {
+        self.start + self.size()
+    }
+
+    /// Return the inclusive end address.
+    pub fn end_addr_inclusive(&self) -> Address<ATYPE> {
+        self.start + (self.size() - 1)
+    }
+
+    /// Check if an address is contained within this descriptor.
+    pub fn contains(&self, addr: Address<ATYPE>) -> bool {
+        (addr >= self.start_addr()) && (addr <= self.end_addr_inclusive())
+    }
+
+    /// Return a non-mutable slice of Pages.
+    ///
+    /// # Safety
+    ///
+    /// - Same as applies for `core::slice::from_raw_parts`.
+    pub unsafe fn as_slice(&self) -> &[Page<ATYPE>] {
+        core::slice::from_raw_parts(self.first_page_ptr(), self.num_pages)
+    }
+}
+
+impl From<PageSliceDescriptor<Virtual>> for PageSliceDescriptor<Physical> {
+    fn from(desc: PageSliceDescriptor<Virtual>) -> Self {
+        Self {
+            start: Address::new(desc.start.into_usize()),
+            num_pages: desc.num_pages,
+        }
+    }
+}
+
+impl From<MMIODescriptor> for PageSliceDescriptor<Physical> {
+    fn from(desc: MMIODescriptor) -> Self {
+        let start_page_addr = desc
+            .start_addr
+            .align_down(bsp::memory::mmu::KernelGranule::SIZE);
+
+        let len = ((desc.end_addr_inclusive().into_usize() - start_page_addr.into_usize())
+            >> bsp::memory::mmu::KernelGranule::SHIFT)
+            + 1;
+
+        Self {
+            start: start_page_addr,
+            num_pages: len,
+        }
+    }
+}
+
+//------------------------------------------------------------------------------
+// MMIODescriptor
+//------------------------------------------------------------------------------
+
+impl MMIODescriptor {
+    /// Create an instance.
+    pub const fn new(start_addr: Address<Physical>, size: usize) -> Self {
+        assert!(size > 0);
+
+        Self { start_addr, size }
+    }
+
+    /// Return the start address.
+    pub const fn start_addr(&self) -> Address<Physical> {
+        self.start_addr
+    }
+
+    /// Return the inclusive end address.
+    pub fn end_addr_inclusive(&self) -> Address<Physical> {
+        self.start_addr + (self.size - 1)
+    }
+
+    /// Return the size.
+    pub const fn size(&self) -> usize {
+        self.size
+    }
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
+    /// Check if the size of `struct Page` is as expected.
+    #[kernel_test]
+    fn size_of_page_equals_granule_size() {
+        assert_eq!(
+            core::mem::size_of::<Page<Physical>>(),
+            bsp::memory::mmu::KernelGranule::SIZE
+        );
+    }
+}

diff -uNr 13_exceptions_part2_peripheral_IRQs/src/memory/mmu.rs 14_virtual_mem_part2_mmio_remap/src/memory/mmu.rs
--- 13_exceptions_part2_peripheral_IRQs/src/memory/mmu.rs
+++ 14_virtual_mem_part2_mmio_remap/src/memory/mmu.rs
@@ -3,29 +3,23 @@
 // Copyright (c) 2020-2021 Andre Richter <andre.o.richter@gmail.com>

 //! Memory Management Unit.
-//!
-//! In order to decouple `BSP` and `arch` parts of the MMU code (to keep them pluggable), this file
-//! provides types for composing an architecture-agnostic description of the kernel's virtual memory
-//! layout.
-//!
-//! The `BSP` provides such a description through the `bsp::memory::mmu::virt_mem_layout()`
-//! function.
-//!
-//! The `MMU` driver of the `arch` code uses `bsp::memory::mmu::virt_mem_layout()` to compile and
-//! install respective translation tables.

 #[cfg(target_arch = "aarch64")]
 #[path = "../_arch/aarch64/memory/mmu.rs"]
 mod arch_mmu;

+mod mapping_record;
 mod translation_table;
+mod types;

-use core::{fmt, ops::RangeInclusive};
+use crate::{
+    bsp,
+    memory::{Address, Physical, Virtual},
+    synchronization, warn,
+};
+use core::fmt;

-//--------------------------------------------------------------------------------------------------
-// Architectural Public Reexports
-//--------------------------------------------------------------------------------------------------
-pub use arch_mmu::mmu;
+pub use types::*;

 //--------------------------------------------------------------------------------------------------
 // Public Definitions
@@ -45,13 +39,15 @@

     /// MMU functions.
     pub trait MMU {
-        /// Called by the kernel during early init. Supposed to take the translation tables from the
-        /// `BSP`-supplied `virt_mem_layout()` and install/activate them for the respective MMU.
+        /// Turns on the MMU for the first time and enables data and instruction caching.
         ///
         /// # Safety
         ///
         /// - Changes the HW's global state.
-        unsafe fn enable_mmu_and_caching(&self) -> Result<(), MMUEnableError>;
+        unsafe fn enable_mmu_and_caching(
+            &self,
+            phys_tables_base_addr: Address<Physical>,
+        ) -> Result<(), MMUEnableError>;

         /// Returns true if the MMU is enabled, false otherwise.
         fn is_enabled(&self) -> bool;
@@ -64,55 +60,43 @@
 /// Describes properties of an address space.
 pub struct AddressSpace<const AS_SIZE: usize>;

-/// Architecture agnostic translation types.
-#[allow(missing_docs)]
-#[derive(Copy, Clone)]
-pub enum Translation {
-    Identity,
-    Offset(usize),
-}
-
-/// Architecture agnostic memory attributes.
-#[allow(missing_docs)]
-#[derive(Copy, Clone)]
-pub enum MemAttributes {
-    CacheableDRAM,
-    Device,
-}
-
-/// Architecture agnostic access permissions.
-#[allow(missing_docs)]
-#[derive(Copy, Clone)]
-pub enum AccessPermissions {
-    ReadOnly,
-    ReadWrite,
-}
-
-/// Collection of memory attributes.
-#[allow(missing_docs)]
-#[derive(Copy, Clone)]
-pub struct AttributeFields {
-    pub mem_attributes: MemAttributes,
-    pub acc_perms: AccessPermissions,
-    pub execute_never: bool,
+/// Intended to be implemented for [`AddressSpace`].
+pub trait AssociatedTranslationTable {
+    /// A translation table whose address range is:
+    ///
+    /// [AS_SIZE - 1, 0]
+    type TableStartFromBottom;
 }

-/// Architecture agnostic descriptor for a memory range.
-#[allow(missing_docs)]
-pub struct TranslationDescriptor {
-    pub name: &'static str,
-    pub virtual_range: fn() -> RangeInclusive<usize>,
-    pub physical_range_translation: Translation,
-    pub attribute_fields: AttributeFields,
-}
+//--------------------------------------------------------------------------------------------------
+// Private Code
+//--------------------------------------------------------------------------------------------------
+use interface::MMU;
+use synchronization::interface::ReadWriteEx;
+use translation_table::interface::TranslationTable;
+
+/// Map pages in the kernel's translation tables.
+///
+/// No input checks done, input is passed through to the architectural implementation.
+///
+/// # Safety
+///
+/// - See `map_pages_at()`.
+/// - Does not prevent aliasing.
+unsafe fn kernel_map_pages_at_unchecked(
+    name: &'static str,
+    virt_pages: &PageSliceDescriptor<Virtual>,
+    phys_pages: &PageSliceDescriptor<Physical>,
+    attr: &AttributeFields,
+) -> Result<(), &'static str> {
+    bsp::memory::mmu::kernel_translation_tables()
+        .write(|tables| tables.map_pages_at(virt_pages, phys_pages, attr))?;

-/// Type for expressing the kernel's virtual memory layout.
-pub struct KernelVirtualLayout<const NUM_SPECIAL_RANGES: usize> {
-    /// The last (inclusive) address of the address space.
-    max_virt_addr_inclusive: usize,
+    if let Err(x) = mapping_record::kernel_add(name, virt_pages, phys_pages, attr) {
+        warn!("{}", x);
+    }

-    /// Array of descriptors for non-standard (normal cacheable DRAM) memory regions.
-    inner: [TranslationDescriptor; NUM_SPECIAL_RANGES],
+    Ok(())
 }

 //--------------------------------------------------------------------------------------------------
@@ -132,6 +116,9 @@
     /// The granule's size.
     pub const SIZE: usize = Self::size_checked();

+    /// The granule's mask.
+    pub const MASK: usize = Self::SIZE - 1;
+
     /// The granule's shift, aka log2(size).
     pub const SHIFT: usize = Self::SIZE.trailing_zeros() as usize;

@@ -159,110 +146,103 @@
     }
 }

-impl Default for AttributeFields {
-    fn default() -> AttributeFields {
-        AttributeFields {
-            mem_attributes: MemAttributes::CacheableDRAM,
-            acc_perms: AccessPermissions::ReadWrite,
-            execute_never: true,
-        }
-    }
-}
-
-/// Human-readable output of a TranslationDescriptor.
-impl fmt::Display for TranslationDescriptor {
-    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
-        // Call the function to which self.range points, and dereference the result, which causes
-        // Rust to copy the value.
-        let start = *(self.virtual_range)().start();
-        let end = *(self.virtual_range)().end();
-        let size = end - start + 1;
-
-        // log2(1024).
-        const KIB_RSHIFT: u32 = 10;
-
-        // log2(1024 * 1024).
-        const MIB_RSHIFT: u32 = 20;
-
-        let (size, unit) = if (size >> MIB_RSHIFT) > 0 {
-            (size >> MIB_RSHIFT, "MiB")
-        } else if (size >> KIB_RSHIFT) > 0 {
-            (size >> KIB_RSHIFT, "KiB")
-        } else {
-            (size, "Byte")
-        };
-
-        let attr = match self.attribute_fields.mem_attributes {
-            MemAttributes::CacheableDRAM => "C",
-            MemAttributes::Device => "Dev",
-        };
-
-        let acc_p = match self.attribute_fields.acc_perms {
-            AccessPermissions::ReadOnly => "RO",
-            AccessPermissions::ReadWrite => "RW",
-        };
-
-        let xn = if self.attribute_fields.execute_never {
-            "PXN"
-        } else {
-            "PX"
-        };
-
-        write!(
-            f,
-            "      {:#010x} - {:#010x} | {: >3} {} | {: <3} {} {: <3} | {}",
-            start, end, size, unit, attr, acc_p, xn, self.name
-        )
-    }
-}
-
-impl<const NUM_SPECIAL_RANGES: usize> KernelVirtualLayout<{ NUM_SPECIAL_RANGES }> {
-    /// Create a new instance.
-    pub const fn new(max: usize, layout: [TranslationDescriptor; NUM_SPECIAL_RANGES]) -> Self {
-        Self {
-            max_virt_addr_inclusive: max,
-            inner: layout,
-        }
-    }
-
-    /// For a virtual address, find and return the physical output address and corresponding
-    /// attributes.
-    ///
-    /// If the address is not found in `inner`, return an identity mapped default with normal
-    /// cacheable DRAM attributes.
-    pub fn virt_addr_properties(
-        &self,
-        virt_addr: usize,
-    ) -> Result<(usize, AttributeFields), &'static str> {
-        if virt_addr > self.max_virt_addr_inclusive {
-            return Err("Address out of range");
-        }
-
-        for i in self.inner.iter() {
-            if (i.virtual_range)().contains(&virt_addr) {
-                let output_addr = match i.physical_range_translation {
-                    Translation::Identity => virt_addr,
-                    Translation::Offset(a) => a + (virt_addr - (i.virtual_range)().start()),
-                };
-
-                return Ok((output_addr, i.attribute_fields));
-            }
-        }
-
-        Ok((virt_addr, AttributeFields::default()))
-    }
-
-    /// Print the memory layout.
-    pub fn print_layout(&self) {
-        use crate::info;
-
-        for i in self.inner.iter() {
-            info!("{}", i);
-        }
-    }
-
-    #[cfg(test)]
-    pub fn inner(&self) -> &[TranslationDescriptor; NUM_SPECIAL_RANGES] {
-        &self.inner
-    }
+/// Raw mapping of virtual to physical pages in the kernel translation tables.
+///
+/// Prevents mapping into the MMIO range of the tables.
+///
+/// # Safety
+///
+/// - See `kernel_map_pages_at_unchecked()`.
+/// - Does not prevent aliasing. Currently, the callers must be trusted.
+pub unsafe fn kernel_map_pages_at(
+    name: &'static str,
+    virt_pages: &PageSliceDescriptor<Virtual>,
+    phys_pages: &PageSliceDescriptor<Physical>,
+    attr: &AttributeFields,
+) -> Result<(), &'static str> {
+    let is_mmio = bsp::memory::mmu::kernel_translation_tables()
+        .read(|tables| tables.is_virt_page_slice_mmio(virt_pages));
+    if is_mmio {
+        return Err("Attempt to manually map into MMIO region");
+    }
+
+    kernel_map_pages_at_unchecked(name, virt_pages, phys_pages, attr)?;
+
+    Ok(())
+}
+
+/// MMIO remapping in the kernel translation tables.
+///
+/// Typically used by device drivers.
+///
+/// # Safety
+///
+/// - Same as `kernel_map_pages_at_unchecked()`, minus the aliasing part.
+pub unsafe fn kernel_map_mmio(
+    name: &'static str,
+    mmio_descriptor: &MMIODescriptor,
+) -> Result<Address<Virtual>, &'static str> {
+    let phys_pages: PageSliceDescriptor<Physical> = (*mmio_descriptor).into();
+    let offset_into_start_page =
+        mmio_descriptor.start_addr().into_usize() & bsp::memory::mmu::KernelGranule::MASK;
+
+    // Check if an identical page slice has been mapped for another driver. If so, reuse it.
+    let virt_addr = if let Some(addr) =
+        mapping_record::kernel_find_and_insert_mmio_duplicate(mmio_descriptor, name)
+    {
+        addr
+    // Otherwise, allocate a new virtual page slice and map it.
+    } else {
+        let virt_pages: PageSliceDescriptor<Virtual> =
+            bsp::memory::mmu::kernel_translation_tables()
+                .write(|tables| tables.next_mmio_virt_page_slice(phys_pages.num_pages()))?;
+
+        kernel_map_pages_at_unchecked(
+            name,
+            &virt_pages,
+            &phys_pages,
+            &AttributeFields {
+                mem_attributes: MemAttributes::Device,
+                acc_perms: AccessPermissions::ReadWrite,
+                execute_never: true,
+            },
+        )?;
+
+        virt_pages.start_addr()
+    };
+
+    Ok(virt_addr + offset_into_start_page)
+}
+
+/// Map the kernel's binary. Returns the translation table's base address.
+///
+/// # Safety
+///
+/// - See [`bsp::memory::mmu::kernel_map_binary()`].
+pub unsafe fn kernel_map_binary() -> Result<Address<Physical>, &'static str> {
+    let phys_kernel_tables_base_addr =
+        bsp::memory::mmu::kernel_translation_tables().write(|tables| {
+            tables.init();
+            tables.phys_base_address()
+        });
+
+    bsp::memory::mmu::kernel_map_binary()?;
+
+    Ok(phys_kernel_tables_base_addr)
+}
+
+/// Enable the MMU and data + instruction caching.
+///
+/// # Safety
+///
+/// - Crucial function during kernel init. Changes the the complete memory view of the processor.
+pub unsafe fn enable_mmu_and_caching(
+    phys_tables_base_addr: Address<Physical>,
+) -> Result<(), MMUEnableError> {
+    arch_mmu::mmu().enable_mmu_and_caching(phys_tables_base_addr)
+}
+
+/// Human-readable print of all recorded kernel mappings.
+pub fn kernel_print_mappings() {
+    mapping_record::kernel_print()
 }

diff -uNr 13_exceptions_part2_peripheral_IRQs/src/memory.rs 14_virtual_mem_part2_mmio_remap/src/memory.rs
--- 13_exceptions_part2_peripheral_IRQs/src/memory.rs
+++ 14_virtual_mem_part2_mmio_remap/src/memory.rs
@@ -6,12 +6,136 @@

 pub mod mmu;

-use core::ops::RangeInclusive;
+use crate::common;
+use core::{
+    fmt,
+    marker::PhantomData,
+    ops::{AddAssign, RangeInclusive, SubAssign},
+};
+
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
+/// Metadata trait for marking the type of an address.
+pub trait AddressType: Copy + Clone + PartialOrd + PartialEq {}
+
+/// Zero-sized type to mark a physical address.
+#[derive(Copy, Clone, PartialOrd, PartialEq)]
+pub enum Physical {}
+
+/// Zero-sized type to mark a virtual address.
+#[derive(Copy, Clone, PartialOrd, PartialEq)]
+pub enum Virtual {}
+
+/// Generic address type.
+#[derive(Copy, Clone, PartialOrd, PartialEq)]
+pub struct Address<ATYPE: AddressType> {
+    value: usize,
+    _address_type: PhantomData<fn() -> ATYPE>,
+}

 //--------------------------------------------------------------------------------------------------
 // Public Code
 //--------------------------------------------------------------------------------------------------

+impl AddressType for Physical {}
+impl AddressType for Virtual {}
+
+impl<ATYPE: AddressType> Address<ATYPE> {
+    /// Create an instance.
+    pub const fn new(value: usize) -> Self {
+        Self {
+            value,
+            _address_type: PhantomData,
+        }
+    }
+
+    /// Align down.
+    pub const fn align_down(self, alignment: usize) -> Self {
+        let aligned = common::align_down(self.value, alignment);
+
+        Self {
+            value: aligned,
+            _address_type: PhantomData,
+        }
+    }
+
+    /// Converts `Address` into an usize.
+    pub const fn into_usize(self) -> usize {
+        self.value
+    }
+}
+
+impl<ATYPE: AddressType> core::ops::Add<usize> for Address<ATYPE> {
+    type Output = Self;
+
+    fn add(self, other: usize) -> Self {
+        Self {
+            value: self.value + other,
+            _address_type: PhantomData,
+        }
+    }
+}
+
+impl<ATYPE: AddressType> AddAssign for Address<ATYPE> {
+    fn add_assign(&mut self, other: Self) {
+        *self = Self {
+            value: self.value + other.into_usize(),
+            _address_type: PhantomData,
+        };
+    }
+}
+
+impl<ATYPE: AddressType> core::ops::Sub<usize> for Address<ATYPE> {
+    type Output = Self;
+
+    fn sub(self, other: usize) -> Self {
+        Self {
+            value: self.value - other,
+            _address_type: PhantomData,
+        }
+    }
+}
+
+impl<ATYPE: AddressType> SubAssign for Address<ATYPE> {
+    fn sub_assign(&mut self, other: Self) {
+        *self = Self {
+            value: self.value - other.into_usize(),
+            _address_type: PhantomData,
+        };
+    }
+}
+
+impl fmt::Display for Address<Physical> {
+    // Don't expect to see physical addresses greater than 40 bit.
+    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
+        let q3: u8 = ((self.value >> 32) & 0xff) as u8;
+        let q2: u16 = ((self.value >> 16) & 0xffff) as u16;
+        let q1: u16 = (self.value & 0xffff) as u16;
+
+        write!(f, "0x")?;
+        write!(f, "{:02x}_", q3)?;
+        write!(f, "{:04x}_", q2)?;
+        write!(f, "{:04x}", q1)
+    }
+}
+
+impl fmt::Display for Address<Virtual> {
+    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
+        let q4: u16 = ((self.value >> 48) & 0xffff) as u16;
+        let q3: u16 = ((self.value >> 32) & 0xffff) as u16;
+        let q2: u16 = ((self.value >> 16) & 0xffff) as u16;
+        let q1: u16 = (self.value & 0xffff) as u16;
+
+        write!(f, "0x")?;
+        write!(f, "{:04x}_", q4)?;
+        write!(f, "{:04x}_", q3)?;
+        write!(f, "{:04x}_", q2)?;
+        write!(f, "{:04x}", q1)
+    }
+}
+
 /// Zero out an inclusive memory range.
 ///
 /// # Safety

diff -uNr 13_exceptions_part2_peripheral_IRQs/tests/02_exception_sync_page_fault.rs 14_virtual_mem_part2_mmio_remap/tests/02_exception_sync_page_fault.rs
--- 13_exceptions_part2_peripheral_IRQs/tests/02_exception_sync_page_fault.rs
+++ 14_virtual_mem_part2_mmio_remap/tests/02_exception_sync_page_fault.rs
@@ -21,7 +21,7 @@

 #[no_mangle]
 unsafe fn kernel_init() -> ! {
-    use memory::mmu::interface::MMU;
+    use libkernel::driver::interface::DriverManager;

     exception::handling_init();
     bsp::console::qemu_bring_up_console();
@@ -29,10 +29,30 @@
     println!("Testing synchronous exception handling by causing a page fault");
     println!("-------------------------------------------------------------------\n");

-    if let Err(string) = memory::mmu::mmu().enable_mmu_and_caching() {
-        println!("MMU: {}", string);
+    let phys_kernel_tables_base_addr = match memory::mmu::kernel_map_binary() {
+        Err(string) => {
+            println!("Error mapping kernel binary: {}", string);
+            cpu::qemu_exit_failure()
+        }
+        Ok(addr) => addr,
+    };
+
+    if let Err(e) = memory::mmu::enable_mmu_and_caching(phys_kernel_tables_base_addr) {
+        println!("Enabling MMU failed: {}", e);
         cpu::qemu_exit_failure()
     }
+    // Printing will silently fail from here on, because the driver's MMIO is not remapped yet.
+
+    // Bring up the drivers needed for printing first.
+    for i in bsp::driver::driver_manager()
+        .early_print_device_drivers()
+        .iter()
+    {
+        // Any encountered errors cannot be printed yet, obviously, so just safely park the CPU.
+        i.init().unwrap_or_else(|_| cpu::qemu_exit_failure());
+    }
+    bsp::driver::driver_manager().post_early_print_device_driver_init();
+    // Printing available again from here on.

     println!("Writing beyond mapped area to address 9 GiB...");
     let big_addr: u64 = 9 * 1024 * 1024 * 1024;

```
