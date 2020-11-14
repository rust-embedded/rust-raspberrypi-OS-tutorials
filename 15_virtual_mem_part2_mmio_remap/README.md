# Tutorial 15 - Virtual Memory Part 2: MMIO Remap

## tl;dr

- We introduce a first set of changes which we eventually need for separating `kernel` and `user`
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
  - [A New Mapping API in `src/memory/mmu.rs`](#a-new-mapping-api-in-srcmemorymmurs)
  - [Using the new API in `bsp` code and drivers](#using-the-new-api-in-bsp-code-and-drivers)
  - [Additional Changes](#additional-changes)
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
1. For now, the `kernel binary` stays identity mapped. This will be changed in the next tutorial as
   it is a quite difficult and peculiar exercise to remap the kernel.
1. Device `MMIO regions` are lazily remapped during the device driver's `init()`.
   1. The remappings will populate the top of the virtual address space. In the `AArch64 MMU
      Driver`, we provide the top `256 MiB` for it.
   1. It is possible to define the size of the virtual address space at compile time. We chose `8
      GiB` for now, which means remapped MMIO virtual addresses will start at `7936 MiB`
      (`0x1F0000000`).
1. We keep using `TTBR0` for the kernel page tables for now. This will be changed when we remap the
   `kernel binary` in the next tutorial.

[ARM Cortex-A Series Programmer’s Guide for ARMv8-A]: https://developer.arm.com/documentation/den0024/latest/
[higher half kernel]: https://wiki.osdev.org/Higher_Half_Kernel

## Implementation

Until now, the whole address space of the board was identity mapped at once. The **architecture**
(`src/_arch/_/memory/**`) and **bsp** (`src/bsp/_/memory/**`) parts of the kernel worked
together directly while setting up the translation tables, without any indirection through **generic
kernel code** (`src/memory/**`).

The way it worked was that the `architectural MMU driver` would query the `bsp code` about the start
and end of the physical address space, and any special regions in this space that need a mapping
that _is not_ normal chacheable DRAM. It would then go ahead and map the whole address space at once
and never touch the page tables again during runtime.

Changing in this tutorial, **architecture** and **bsp** code will no longer talk to each other
directly. Instead, this is decoupled now through the kernel's **generic MMU subsystem code**.

### A New Mapping API in `src/memory/mmu.rs`

First, we define an interface for operating on `translation tables`:

```rust
/// Translation table operations.
pub trait TranslationTable {
    /// Anything that needs to run before any of the other provided functions can be used.
    unsafe fn init(&mut self);

    /// The translation table's base address to be used for programming the MMU.
    fn phys_base_address(&self) -> Address<Physical>;

    /// Map the given physical pages to the given virtual pages.
    unsafe fn map_pages_at(
        &mut self,
        phys_pages: &PageSliceDescriptor<Physical>,
        virt_pages: &PageSliceDescriptor<Virtual>,
        attr: &AttributeFields,
    ) -> Result<(), &'static str>;

    /// Obtain a free virtual page slice in the MMIO region.
    ///
    /// The "MMIO region" is a distinct region of the implementor's choice, which allows
    /// differentiating MMIO addresses from others. This can speed up debugging efforts.
    /// Ideally, those MMIO addresses are also standing out visually so that a human eye can
    /// identify them. For example, by allocating them from near the end of the virtual address
    /// space.
    fn next_mmio_virt_page_slice(
        &mut self,
        num_pages: usize,
    ) -> Result<PageSliceDescriptor<Virtual>, &'static str>;

    /// Check if a virtual page splice is in the "MMIO region".
    fn is_virt_page_slice_mmio(&self, virt_pages: &PageSliceDescriptor<Virtual>) -> bool;
}
```

The MMU driver (`src/_arch/_/memory/mmu.rs`) has one global instance for the kernel tables which
implements this interface, and which can be accessed by calling
`arch_mmu::kernel_translation_tables()` in the generic kernel code (`src/memory/mmu.rs`). From
there, we provice a couple of memory mapping functions that wrap around this interface , and which
are exported for the rest of the kernel to use:

```rust
/// Raw mapping of virtual to physical pages in the kernel translation tables.
///
/// Prevents mapping into the MMIO range of the tables.
pub unsafe fn kernel_map_pages_at(
    name: &'static str,
    phys_pages: &PageSliceDescriptor<Physical>,
    virt_pages: &PageSliceDescriptor<Virtual>,
    attr: &AttributeFields,
) -> Result<(), &'static str>;

/// MMIO remapping in the kernel translation tables.
///
/// Typically used by device drivers.
pub unsafe fn kernel_map_mmio(
    name: &'static str,
    phys_mmio_descriptor: &MMIODescriptor<Physical>,
) -> Result<Address<Virtual>, &'static str>;

/// Map the kernel's binary and enable the MMU.
pub unsafe fn kernel_map_binary_and_enable_mmu() -> Result<(), &'static str> ;
```

### Using the new API in `bsp` code and drivers

For now, there are two places where the new API is used. First, in `src/bsp/_/memory/mmu.rs`, which
provides a dedicated call to **map the kernel binary** (because it is the `BSP` that provides the
`linker script`, which in turn defines the final layout of the kernel in memory):

```rust
/// Map the kernel binary.
pub unsafe fn kernel_map_binary() -> Result<(), &'static str> {
    kernel_mmu::kernel_map_pages_at(
        "Kernel boot-core stack",
        &phys_stack_page_desc(),
        &virt_stack_page_desc(),
        &AttributeFields {
            mem_attributes: MemAttributes::CacheableDRAM,
            acc_perms: AccessPermissions::ReadWrite,
            execute_never: true,
        },
    )?;

    kernel_mmu::kernel_map_pages_at(
        "Kernel code and RO data",
        // omitted for brevity.
    )?;

    kernel_mmu::kernel_map_pages_at(
        "Kernel data and bss",
        // omitted for brevity.
    )?;

    Ok(())
}
```

Second, in device drivers, which now expect an `MMIODescriptor` type instead of a raw address. The
following is an example for the `UART`:

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

    let mut r = &self.inner;
    r.lock(|inner| inner.init(Some(virt_addr.into_usize())))?;

     // omitted for brevity.

     Ok(())
}
```

### Supporting Changes

There's a couple of changes not covered in this tutorial text, but the reader should ideally skim
through them:

- [`src/memory/mmu/types.rs`](src/memory/mmu/types.rs) introduces a couple of supporting types, like
  `Address<ATYPE>`, which is used to differentiate between `Physical` and `Virtual` addresses.
- [`src/memory/mmu/mapping_record.rs`](src/memory/mmu/mapping_record.rs) provides the generic kernel
  code's way of tracking previous memory mappings for use cases such as reusing existing mappings
  (in case of drivers that have their MMIO ranges in the same `64 KiB` page) or printing mappings
  statistics.

## Test it

When you load the kernel, you can now see that the driver's MMIO virtual addresses start at
`0x1F0000000`:

Raspberry Pi 3:

```console
$ make chainboot
[...]
Minipush 1.0

[MP] ⏳ Waiting for /dev/ttyUSB0
[MP] ✅ Connected
 __  __ _      _ _                 _
|  \/  (_)_ _ (_) |   ___  __ _ __| |
| |\/| | | ' \| | |__/ _ \/ _` / _` |
|_|  |_|_|_||_|_|____\___/\__,_\__,_|

           Raspberry Pi 3

[ML] Requesting binary
[MP] ⏩ Pushing 67 KiB ========================================🦀 100% 33 KiB/s Time: 00:00:02
[ML] Loaded! Executing the payload now

[    3.041355] Booting on: Raspberry Pi 3
[    3.042438] MMU online:
[    3.043609]       -----------------------------------------------------------------------------------------------------------------
[    3.049466]               Virtual                      Physical            Size       Attr                    Entity
[    3.055323]       -----------------------------------------------------------------------------------------------------------------
[    3.061183]       0x000070000..0x00007FFFF --> 0x000070000..0x00007FFFF |  64 KiB | C   RW XN | Kernel boot-core stack
[    3.066476]       0x000080000..0x00008FFFF --> 0x000080000..0x00008FFFF |  64 KiB | C   RO X  | Kernel code and RO data
[    3.071812]       0x000090000..0x0001AFFFF --> 0x000090000..0x0001AFFFF |   1 MiB | C   RW XN | Kernel data and bss
[    3.076975]       0x1F0000000..0x1F000FFFF --> 0x03F200000..0x03F20FFFF |  64 KiB | Dev RW XN | BCM GPIO
[    3.081658]                                                                                   | BCM PL011 UART
[    3.086606]       0x1F0010000..0x1F001FFFF --> 0x03F000000..0x03F00FFFF |  64 KiB | Dev RW XN | BCM Peripheral Interrupt Controller
[    3.092462]       -----------------------------------------------------------------------------------------------------------------
```

Raspberry Pi 4:

```console
$ BSP=rpi4 make chainboot
[...]
Minipush 1.0

[MP] ⏳ Waiting for /dev/ttyUSB0
[MP] ✅ Connected
 __  __ _      _ _                 _
|  \/  (_)_ _ (_) |   ___  __ _ __| |
| |\/| | | ' \| | |__/ _ \/ _` / _` |
|_|  |_|_|_||_|_|____\___/\__,_\__,_|

           Raspberry Pi 4

[ML] Requesting binary
[MP] ⏩ Pushing 74 KiB ========================================🦀 100% 24 KiB/s Time: 00:00:03
[ML] Loaded! Executing the payload now

[    3.376642] Booting on: Raspberry Pi 4
[    3.377030] MMU online:
[    3.378202]       -----------------------------------------------------------------------------------------------------------------
[    3.384059]               Virtual                      Physical            Size       Attr                    Entity
[    3.389916]       -----------------------------------------------------------------------------------------------------------------
[    3.395775]       0x000070000..0x00007FFFF --> 0x000070000..0x00007FFFF |  64 KiB | C   RW XN | Kernel boot-core stack
[    3.401069]       0x000080000..0x00008FFFF --> 0x000080000..0x00008FFFF |  64 KiB | C   RO X  | Kernel code and RO data
[    3.406404]       0x000090000..0x0001AFFFF --> 0x000090000..0x0001AFFFF |   1 MiB | C   RW XN | Kernel data and bss
[    3.411566]       0x1F0000000..0x1F000FFFF --> 0x0FE200000..0x0FE20FFFF |  64 KiB | Dev RW XN | BCM GPIO
[    3.416251]                                                                                   | BCM PL011 UART
[    3.421198]       0x1F0010000..0x1F001FFFF --> 0x0FF840000..0x0FF84FFFF |  64 KiB | Dev RW XN | GICD
[    3.425709]                                                                                   | GICC
[    3.430221]       -----------------------------------------------------------------------------------------------------------------
```

## Diff to previous
```diff

diff -uNr 14_exceptions_part2_peripheral_IRQs/src/_arch/aarch64/cpu.rs 15_virtual_mem_part2_mmio_remap/src/_arch/aarch64/cpu.rs
--- 14_exceptions_part2_peripheral_IRQs/src/_arch/aarch64/cpu.rs
+++ 15_virtual_mem_part2_mmio_remap/src/_arch/aarch64/cpu.rs
@@ -68,7 +68,7 @@
     ELR_EL2.set(runtime_init::runtime_init as *const () as u64);

     // Set up SP_EL1 (stack pointer), which will be used by EL1 once we "return" to it.
-    SP_EL1.set(bsp::memory::boot_core_stack_end() as u64);
+    SP_EL1.set(bsp::memory::phys_boot_core_stack_end().into_usize() as u64);

     // Use `eret` to "return" to EL1. This results in execution of runtime_init() in EL1.
     asm::eret()

diff -uNr 14_exceptions_part2_peripheral_IRQs/src/_arch/aarch64/memory/mmu.rs 15_virtual_mem_part2_mmio_remap/src/_arch/aarch64/memory/mmu.rs
--- 14_exceptions_part2_peripheral_IRQs/src/_arch/aarch64/memory/mmu.rs
+++ 15_virtual_mem_part2_mmio_remap/src/_arch/aarch64/memory/mmu.rs
@@ -4,10 +4,19 @@

 //! Memory Management Unit Driver.
 //!
-//! Static translation tables, compiled on boot; Everything 64 KiB granule.
+//! Only 64 KiB granule is supported.

-use super::{AccessPermissions, AttributeFields, MemAttributes};
-use crate::{bsp, memory};
+use crate::{
+    bsp,
+    memory::{
+        mmu,
+        mmu::{
+            AccessPermissions, Address, AddressType, AttributeFields, MemAttributes, Page,
+            PageSliceDescriptor, Physical, Virtual,
+        },
+    },
+    synchronization::InitStateLock,
+};
 use core::convert;
 use cortex_a::{barrier, regs::*};
 use register::{register_bitfields, InMemoryRegister};
@@ -15,6 +24,7 @@
 //--------------------------------------------------------------------------------------------------
 // Private Definitions
 //--------------------------------------------------------------------------------------------------
+use mmu::interface::TranslationGranule;

 // A table descriptor, as per ARMv8-A Architecture Reference Manual Figure D5-15.
 register_bitfields! {u64,
@@ -81,9 +91,6 @@
     ]
 }

-const SIXTYFOUR_KIB_SHIFT: usize = 16; //  log2(64 * 1024)
-const FIVETWELVE_MIB_SHIFT: usize = 29; // log2(512 * 1024 * 1024)
-
 /// A table descriptor for 64 KiB aperture.
 ///
 /// The output points to the next table.
@@ -98,36 +105,65 @@
 #[repr(transparent)]
 struct PageDescriptor(InMemoryRegister<u64, STAGE1_PAGE_DESCRIPTOR::Register>);

+#[derive(Copy, Clone)]
+enum Granule512MiB {}
+
+trait BaseAddr {
+    fn phys_base_addr(&self) -> Address<Physical>;
+}
+
+/// Constants for indexing the MAIR_EL1.
+#[allow(dead_code)]
+mod mair {
+    pub const DEVICE: u64 = 0;
+    pub const NORMAL: u64 = 1;
+}
+
+/// Memory Management Unit type.
+struct MemoryManagementUnit;
+
+/// This constant is the power-of-two exponent that defines the virtual address space size.
+///
+/// Values tested and known to be working:
+///   - 30 (1 GiB)
+///   - 31 (2 GiB)
+///   - 32 (4 GiB)
+///   - 33 (8 GiB)
+const ADDR_SPACE_SIZE_EXPONENT: usize = 33;
+
+const NUM_LVL2_TABLES: usize = (1 << ADDR_SPACE_SIZE_EXPONENT) >> Granule512MiB::SHIFT;
+const T0SZ: u64 = (64 - ADDR_SPACE_SIZE_EXPONENT) as u64;
+
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
 /// Big monolithic struct for storing the translation tables. Individual levels must be 64 KiB
 /// aligned, hence the "reverse" order of appearance.
 #[repr(C)]
 #[repr(align(65536))]
-struct FixedSizeTranslationTable<const NUM_TABLES: usize> {
+pub(in crate::memory::mmu) struct FixedSizeTranslationTable<const NUM_TABLES: usize> {
     /// Page descriptors, covering 64 KiB windows per entry.
     lvl3: [[PageDescriptor; 8192]; NUM_TABLES],

     /// Table descriptors, covering 512 MiB windows.
     lvl2: [TableDescriptor; NUM_TABLES],
-}

-/// Usually evaluates to 1 GiB for RPi3 and 4 GiB for RPi 4.
-const NUM_LVL2_TABLES: usize = bsp::memory::mmu::addr_space_size() >> FIVETWELVE_MIB_SHIFT;
-type ArchTranslationTable = FixedSizeTranslationTable<NUM_LVL2_TABLES>;
+    /// Index of the next free MMIO page.
+    cur_l3_mmio_index: usize,

-trait BaseAddr {
-    fn base_addr_u64(&self) -> u64;
-    fn base_addr_usize(&self) -> usize;
+    /// Have the tables been initialized?
+    initialized: bool,
 }

-/// Constants for indexing the MAIR_EL1.
-#[allow(dead_code)]
-mod mair {
-    pub const DEVICE: u64 = 0;
-    pub const NORMAL: u64 = 1;
-}
+pub(in crate::memory::mmu) type ArchTranslationTable = FixedSizeTranslationTable<NUM_LVL2_TABLES>;

-/// Memory Management Unit type.
-struct MemoryManagementUnit;
+// Supported translation granules are exported below, so that BSP code can pick between the options.
+// This driver only supports 64 KiB at the moment.
+
+#[derive(Copy, Clone)]
+/// 64 KiB translation granule.
+pub enum Granule64KiB {}

 //--------------------------------------------------------------------------------------------------
 // Global instances
@@ -138,7 +174,8 @@
 /// # Safety
 ///
 /// - Supposed to land in `.bss`. Therefore, ensure that all initial member values boil down to "0".
-static mut KERNEL_TABLES: ArchTranslationTable = ArchTranslationTable::new();
+static KERNEL_TABLES: InitStateLock<ArchTranslationTable> =
+    InitStateLock::new(ArchTranslationTable::new());

 static MMU: MemoryManagementUnit = MemoryManagementUnit;

@@ -146,13 +183,15 @@
 // Private Code
 //--------------------------------------------------------------------------------------------------

-impl<T, const N: usize> BaseAddr for [T; N] {
-    fn base_addr_u64(&self) -> u64 {
-        self as *const T as u64
-    }
+impl mmu::interface::TranslationGranule for Granule512MiB {
+    const SIZE: usize = 512 * 1024 * 1024;
+    const SHIFT: usize = 29; // log2(SIZE)
+}

-    fn base_addr_usize(&self) -> usize {
-        self as *const _ as usize
+impl<T, const N: usize> BaseAddr for [T; N] {
+    fn phys_base_addr(&self) -> Address<Physical> {
+        // The binary is still identity mapped, so we don't need to convert here.
+        Address::new(self as *const _ as usize)
     }
 }

@@ -160,7 +199,7 @@
     fn from(next_lvl_table_addr: usize) -> Self {
         let val = InMemoryRegister::<u64, STAGE1_TABLE_DESCRIPTOR::Register>::new(0);

-        let shifted = next_lvl_table_addr >> SIXTYFOUR_KIB_SHIFT;
+        let shifted = next_lvl_table_addr >> Granule64KiB::SHIFT;
         val.write(
             STAGE1_TABLE_DESCRIPTOR::VALID::True
                 + STAGE1_TABLE_DESCRIPTOR::TYPE::Table
@@ -207,23 +246,32 @@

 impl PageDescriptor {
     /// Create an instance.
-    fn new(output_addr: usize, attribute_fields: AttributeFields) -> Self {
+    fn new(output_addr: *const Page<Physical>, attribute_fields: &AttributeFields) -> Self {
         let val = InMemoryRegister::<u64, STAGE1_PAGE_DESCRIPTOR::Register>::new(0);

-        let shifted = output_addr as u64 >> SIXTYFOUR_KIB_SHIFT;
+        let shifted = output_addr as u64 >> Granule64KiB::SHIFT;
         val.write(
             STAGE1_PAGE_DESCRIPTOR::VALID::True
                 + STAGE1_PAGE_DESCRIPTOR::AF::True
-                + attribute_fields.into()
+                + attribute_fields.clone().into()
                 + STAGE1_PAGE_DESCRIPTOR::TYPE::Table
                 + STAGE1_PAGE_DESCRIPTOR::OUTPUT_ADDR_64KiB.val(shifted),
         );

         Self(val)
     }
+
+    /// Returns the valid bit.
+    fn is_valid(&self) -> bool {
+        self.0.is_set(STAGE1_PAGE_DESCRIPTOR::VALID)
+    }
 }

 impl<const NUM_TABLES: usize> FixedSizeTranslationTable<{ NUM_TABLES }> {
+    // Reserve the last 256 MiB of the address space for MMIO mappings.
+    const L2_MMIO_START_INDEX: usize = NUM_TABLES - 1;
+    const L3_MMIO_START_INDEX: usize = 8192 / 2;
+
     /// Create an instance.
     pub const fn new() -> Self {
         assert!(NUM_TABLES > 0);
@@ -231,7 +279,55 @@
         Self {
             lvl3: [[PageDescriptor(InMemoryRegister::new(0)); 8192]; NUM_TABLES],
             lvl2: [TableDescriptor(InMemoryRegister::new(0)); NUM_TABLES],
+            cur_l3_mmio_index: 0,
+            initialized: false,
+        }
+    }
+
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
+    fn lvl2_lvl3_index_from<ATYPE: AddressType>(
+        &self,
+        addr: *const Page<ATYPE>,
+    ) -> Result<(usize, usize), &'static str> {
+        let lvl2_index = addr as usize >> Granule512MiB::SHIFT;
+        let lvl3_index = (addr as usize & Granule512MiB::MASK) >> Granule64KiB::SHIFT;
+
+        if lvl2_index > (NUM_TABLES - 1) {
+            return Err("Virtual page is out of bounds of translation table");
         }
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
     }
 }

@@ -248,28 +344,6 @@
     );
 }

-/// Iterates over all static translation table entries and fills them at once.
-///
-/// # Safety
-///
-/// - Modifies a `static mut`. Ensure it only happens from here.
-unsafe fn populate_tt_entries() -> Result<(), &'static str> {
-    for (l2_nr, l2_entry) in KERNEL_TABLES.lvl2.iter_mut().enumerate() {
-        *l2_entry = KERNEL_TABLES.lvl3[l2_nr].base_addr_usize().into();
-
-        for (l3_nr, l3_entry) in KERNEL_TABLES.lvl3[l2_nr].iter_mut().enumerate() {
-            let virt_addr = (l2_nr << FIVETWELVE_MIB_SHIFT) + (l3_nr << SIXTYFOUR_KIB_SHIFT);
-
-            let (output_addr, attribute_fields) =
-                bsp::memory::mmu::virt_mem_layout().virt_addr_properties(virt_addr)?;
-
-            *l3_entry = PageDescriptor::new(output_addr, attribute_fields);
-        }
-    }
-
-    Ok(())
-}
-
 /// Configure various settings of stage 1 of the EL1 translation regime.
 fn configure_translation_control() {
     let ips = ID_AA64MMFR0_EL1.read(ID_AA64MMFR0_EL1::PARange);
@@ -282,7 +356,7 @@
             + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
             + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
             + TCR_EL1::EPD0::EnableTTBR0Walks
-            + TCR_EL1::T0SZ.val(32), // TTBR0 spans 4 GiB total.
+            + TCR_EL1::T0SZ.val(T0SZ),
     );
 }

@@ -290,17 +364,126 @@
 // Public Code
 //--------------------------------------------------------------------------------------------------

+/// Return a guarded reference to the kernel's translation tables.
+pub(in crate::memory::mmu) fn kernel_translation_tables(
+) -> &'static InitStateLock<ArchTranslationTable> {
+    &KERNEL_TABLES
+}
+
 /// Return a reference to the MMU instance.
-pub fn mmu() -> &'static impl memory::mmu::interface::MMU {
+pub(in crate::memory::mmu) fn mmu() -> &'static impl mmu::interface::MMU {
     &MMU
 }

 //------------------------------------------------------------------------------
 // OS Interface Code
 //------------------------------------------------------------------------------
+impl mmu::interface::TranslationGranule for Granule64KiB {
+    const SIZE: usize = 64 * 1024;
+    const SHIFT: usize = 16; // log2(SIZE)
+}
+
+impl<const NUM_TABLES: usize> mmu::interface::TranslationTable
+    for FixedSizeTranslationTable<{ NUM_TABLES }>
+{
+    unsafe fn init(&mut self) {
+        if self.initialized {
+            return;
+        }
+
+        // Populate the l2 entries.
+        for (lvl2_nr, lvl2_entry) in self.lvl2.iter_mut().enumerate() {
+            *lvl2_entry = self.lvl3[lvl2_nr].phys_base_addr().into_usize().into();
+        }
+
+        self.cur_l3_mmio_index = Self::L3_MMIO_START_INDEX;
+        self.initialized = true;
+    }
+
+    fn phys_base_address(&self) -> Address<Physical> {
+        self.lvl2.phys_base_addr()
+    }
+
+    unsafe fn map_pages_at(
+        &mut self,
+        phys_pages: &PageSliceDescriptor<Physical>,
+        virt_pages: &PageSliceDescriptor<Virtual>,
+        attr: &AttributeFields,
+    ) -> Result<(), &'static str> {
+        assert_eq!(self.initialized, true, "Translation tables not initialized");
+
+        let p = phys_pages.as_slice();
+        let v = virt_pages.as_slice();
+
+        if p.len() != v.len() {
+            return Err("Tried to map page slices with unequal sizes");
+        }

-impl memory::mmu::interface::MMU for MemoryManagementUnit {
-    unsafe fn init(&self) -> Result<(), &'static str> {
+        // No work to do for empty slices.
+        if p.is_empty() {
+            return Ok(());
+        }
+
+        if p.last().unwrap().as_ptr() >= bsp::memory::mmu::phys_addr_space_end_page() {
+            return Err("Tried to map outside of physical address space");
+        }
+
+        let iter = p.iter().zip(v.iter());
+        for (phys_page, virt_page) in iter {
+            let page_descriptor = self.page_descriptor_from(virt_page.as_ptr())?;
+            if page_descriptor.is_valid() {
+                return Err("Virtual page is already mapped");
+            }
+
+            *page_descriptor = PageDescriptor::new(phys_page.as_ptr(), &attr);
+        }
+
+        Ok(())
+    }
+
+    fn next_mmio_virt_page_slice(
+        &mut self,
+        num_pages: usize,
+    ) -> Result<PageSliceDescriptor<Virtual>, &'static str> {
+        assert_eq!(self.initialized, true, "Translation tables not initialized");
+
+        if num_pages == 0 {
+            return Err("num_pages == 0");
+        }
+
+        if (self.cur_l3_mmio_index + num_pages) > 8191 {
+            return Err("Not enough MMIO space left");
+        }
+
+        let addr = (Self::L2_MMIO_START_INDEX << Granule512MiB::SHIFT)
+            | (self.cur_l3_mmio_index << Granule64KiB::SHIFT);
+        self.cur_l3_mmio_index += num_pages;
+
+        Ok(PageSliceDescriptor::from_addr(
+            Address::new(addr),
+            num_pages,
+        ))
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
+    }
+}
+
+impl mmu::interface::MMU for MemoryManagementUnit {
+    unsafe fn enable(
+        &self,
+        phys_kernel_table_base_addr: Address<Physical>,
+    ) -> Result<(), &'static str> {
         // Fail early if translation granule is not supported. Both RPis support it, though.
         if !ID_AA64MMFR0_EL1.matches_all(ID_AA64MMFR0_EL1::TGran64::Supported) {
             return Err("Translation granule not supported in HW");
@@ -309,11 +492,8 @@
         // Prepare the memory attribute indirection register.
         set_up_mair();

-        // Populate translation tables.
-        populate_tt_entries()?;
-
         // Set the "Translation Table Base Register".
-        TTBR0_EL1.set_baddr(KERNEL_TABLES.lvl2.base_addr_u64());
+        TTBR0_EL1.set_baddr(phys_kernel_table_base_addr.into_usize() as u64);

         configure_translation_control();

@@ -337,6 +517,9 @@
 //--------------------------------------------------------------------------------------------------

 #[cfg(test)]
+pub(in crate::memory::mmu) type MinSizeArchTranslationTable = FixedSizeTranslationTable<1>;
+
+#[cfg(test)]
 mod tests {
     use super::*;
     use test_macros::kernel_test;
@@ -363,7 +546,7 @@
     #[kernel_test]
     fn kernel_tables_in_bss() {
         let bss_range = bsp::memory::bss_range_inclusive();
-        let kernel_tables_addr = unsafe { &KERNEL_TABLES as *const _ as usize as *mut u64 };
+        let kernel_tables_addr = &KERNEL_TABLES as *const _ as usize as *mut u64;

         assert!(bss_range.contains(&kernel_tables_addr));
     }

diff -uNr 14_exceptions_part2_peripheral_IRQs/src/_arch/aarch64/time.rs 15_virtual_mem_part2_mmio_remap/src/_arch/aarch64/time.rs
--- 14_exceptions_part2_peripheral_IRQs/src/_arch/aarch64/time.rs
+++ 15_virtual_mem_part2_mmio_remap/src/_arch/aarch64/time.rs
@@ -55,7 +55,7 @@
         }

         // Calculate the register compare value.
-        let frq = CNTFRQ_EL0.get() as u64;
+        let frq = CNTFRQ_EL0.get();
         let x = match frq.checked_mul(duration.as_nanos() as u64) {
             None => {
                 warn!("Spin duration too long, skipping");

diff -uNr 14_exceptions_part2_peripheral_IRQs/src/bsp/device_driver/arm/gicv2/gicc.rs 15_virtual_mem_part2_mmio_remap/src/bsp/device_driver/arm/gicv2/gicc.rs
--- 14_exceptions_part2_peripheral_IRQs/src/bsp/device_driver/arm/gicv2/gicc.rs
+++ 15_virtual_mem_part2_mmio_remap/src/bsp/device_driver/arm/gicv2/gicc.rs
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
+        let mut r = &self.registers;
+        r.write(|regs| *regs = Registers::new(new_mmio_start_addr));
+    }
+
     /// Accept interrupts of any priority.
     ///
     /// Quoting the GICv2 Architecture Specification:
@@ -87,7 +95,10 @@
     /// - GICC MMIO registers are banked per CPU core. It is therefore safe to have `&self` instead
     ///   of `&mut self`.
     pub fn priority_accept_all(&self) {
-        self.registers.PMR.write(PMR::Priority.val(255)); // Comment in arch spec.
+        let mut r = &self.registers;
+        r.read(|regs| {
+            regs.PMR.write(PMR::Priority.val(255)); // Comment in arch spec.
+        });
     }

     /// Enable the interface - start accepting IRQs.
@@ -97,7 +108,10 @@
     /// - GICC MMIO registers are banked per CPU core. It is therefore safe to have `&self` instead
     ///   of `&mut self`.
     pub fn enable(&self) {
-        self.registers.CTLR.write(CTLR::Enable::SET);
+        let mut r = &self.registers;
+        r.read(|regs| {
+            regs.CTLR.write(CTLR::Enable::SET);
+        });
     }

     /// Extract the number of the highest-priority pending IRQ.
@@ -113,7 +127,8 @@
         &self,
         _ic: &exception::asynchronous::IRQContext<'irq_context>,
     ) -> usize {
-        self.registers.IAR.read(IAR::InterruptID) as usize
+        let mut r = &self.registers;
+        r.read(|regs| regs.IAR.read(IAR::InterruptID) as usize)
     }

     /// Complete handling of the currently active IRQ.
@@ -132,6 +147,9 @@
         irq_number: u32,
         _ic: &exception::asynchronous::IRQContext<'irq_context>,
     ) {
-        self.registers.EOIR.write(EOIR::EOIINTID.val(irq_number));
+        let mut r = &self.registers;
+        r.read(|regs| {
+            regs.EOIR.write(EOIR::EOIINTID.val(irq_number));
+        });
     }
 }

diff -uNr 14_exceptions_part2_peripheral_IRQs/src/bsp/device_driver/arm/gicv2/gicd.rs 15_virtual_mem_part2_mmio_remap/src/bsp/device_driver/arm/gicv2/gicd.rs
--- 14_exceptions_part2_peripheral_IRQs/src/bsp/device_driver/arm/gicv2/gicd.rs
+++ 15_virtual_mem_part2_mmio_remap/src/bsp/device_driver/arm/gicv2/gicd.rs
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
@@ -127,10 +129,18 @@
     pub const unsafe fn new(mmio_start_addr: usize) -> Self {
         Self {
             shared_registers: IRQSafeNullLock::new(SharedRegisters::new(mmio_start_addr)),
-            banked_registers: BankedRegisters::new(mmio_start_addr),
+            banked_registers: InitStateLock::new(BankedRegisters::new(mmio_start_addr)),
         }
     }

+    pub unsafe fn set_mmio(&self, new_mmio_start_addr: usize) {
+        let mut r = &self.shared_registers;
+        r.lock(|regs| *regs = SharedRegisters::new(new_mmio_start_addr));
+
+        let mut r = &self.banked_registers;
+        r.write(|regs| *regs = BankedRegisters::new(new_mmio_start_addr));
+    }
+
     /// Use a banked ITARGETSR to retrieve the executing core's GIC target mask.
     ///
     /// Quoting the GICv2 Architecture Specification:
@@ -138,7 +148,8 @@
     ///   "GICD_ITARGETSR0 to GICD_ITARGETSR7 are read-only, and each field returns a value that
     ///    corresponds only to the processor reading the register."
     fn local_gic_target_mask(&self) -> u32 {
-        self.banked_registers.ITARGETSR[0].read(ITARGETSR::Offset0)
+        let mut r = &self.banked_registers;
+        r.read(|regs| regs.ITARGETSR[0].read(ITARGETSR::Offset0))
     }

     /// Route all SPIs to the boot core and enable the distributor.
@@ -179,8 +190,11 @@
         match irq_num {
             // Private.
             0..=31 => {
-                let enable_reg = &self.banked_registers.ISENABLER;
-                enable_reg.set(enable_reg.get() | enable_bit);
+                let mut r = &self.banked_registers;
+                r.read(|regs| {
+                    let enable_reg = &regs.ISENABLER;
+                    enable_reg.set(enable_reg.get() | enable_bit);
+                })
             }
             // Shared.
             _ => {

diff -uNr 14_exceptions_part2_peripheral_IRQs/src/bsp/device_driver/arm/gicv2.rs 15_virtual_mem_part2_mmio_remap/src/bsp/device_driver/arm/gicv2.rs
--- 14_exceptions_part2_peripheral_IRQs/src/bsp/device_driver/arm/gicv2.rs
+++ 15_virtual_mem_part2_mmio_remap/src/bsp/device_driver/arm/gicv2.rs
@@ -79,7 +79,11 @@
 mod gicc;
 mod gicd;

-use crate::{bsp, cpu, driver, exception, synchronization, synchronization::InitStateLock};
+use crate::{
+    bsp, cpu, driver, exception, memory, memory::mmu::Physical, synchronization,
+    synchronization::InitStateLock,
+};
+use core::sync::atomic::{AtomicBool, Ordering};

 //--------------------------------------------------------------------------------------------------
 // Private Definitions
@@ -96,12 +100,18 @@

 /// Representation of the GIC.
 pub struct GICv2 {
+    gicd_phys_mmio_descriptor: memory::mmu::MMIODescriptor<Physical>,
+    gicc_phys_mmio_descriptor: memory::mmu::MMIODescriptor<Physical>,
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
@@ -118,11 +128,17 @@
     ///
     /// # Safety
     ///
-    /// - The user must ensure to provide a correct MMIO start address.
-    pub const unsafe fn new(gicd_mmio_start_addr: usize, gicc_mmio_start_addr: usize) -> Self {
+    /// - The user must ensure to provide correct MMIO descriptors.
+    pub const unsafe fn new(
+        gicd_phys_mmio_descriptor: memory::mmu::MMIODescriptor<Physical>,
+        gicc_phys_mmio_descriptor: memory::mmu::MMIODescriptor<Physical>,
+    ) -> Self {
         Self {
-            gicd: gicd::GICD::new(gicd_mmio_start_addr),
-            gicc: gicc::GICC::new(gicc_mmio_start_addr),
+            gicd_phys_mmio_descriptor,
+            gicc_phys_mmio_descriptor,
+            gicd: gicd::GICD::new(gicd_phys_mmio_descriptor.start_addr().into_usize()),
+            gicc: gicc::GICC::new(gicc_phys_mmio_descriptor.start_addr().into_usize()),
+            is_mmio_remapped: AtomicBool::new(false),
             handler_table: InitStateLock::new([None; Self::NUM_IRQS]),
         }
     }
@@ -139,6 +155,22 @@
     }

     unsafe fn init(&self) -> Result<(), &'static str> {
+        let remapped = self.is_mmio_remapped.load(Ordering::Relaxed);
+        if !remapped {
+            let mut virt_addr;
+
+            // GICD
+            virt_addr = memory::mmu::kernel_map_mmio("GICD", &self.gicd_phys_mmio_descriptor)?;
+            self.gicd.set_mmio(virt_addr.into_usize());
+
+            // GICC
+            virt_addr = memory::mmu::kernel_map_mmio("GICC", &self.gicc_phys_mmio_descriptor)?;
+            self.gicc.set_mmio(virt_addr.into_usize());
+
+            // Conclude remapping.
+            self.is_mmio_remapped.store(true, Ordering::Relaxed);
+        }
+
         if cpu::smp::core_id::<usize>() == bsp::cpu::BOOT_CORE_ID {
             self.gicd.boot_core_init();
         }

diff -uNr 14_exceptions_part2_peripheral_IRQs/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs 15_virtual_mem_part2_mmio_remap/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs
--- 14_exceptions_part2_peripheral_IRQs/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs
+++ 15_virtual_mem_part2_mmio_remap/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs
@@ -5,9 +5,10 @@
 //! GPIO Driver.

 use crate::{
-    bsp::device_driver::common::MMIODerefWrapper, driver, synchronization,
-    synchronization::IRQSafeNullLock,
+    bsp::device_driver::common::MMIODerefWrapper, driver, memory, memory::mmu::Physical,
+    synchronization, synchronization::IRQSafeNullLock,
 };
+use core::sync::atomic::{AtomicUsize, Ordering};
 use register::{mmio::*, register_bitfields, register_structs};

 //--------------------------------------------------------------------------------------------------
@@ -117,6 +118,8 @@

 /// Representation of the GPIO HW.
 pub struct GPIO {
+    phys_mmio_descriptor: memory::mmu::MMIODescriptor<Physical>,
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
@@ -190,10 +206,14 @@
     ///
     /// # Safety
     ///
-    /// - The user must ensure to provide a correct MMIO start address.
-    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
+    /// - The user must ensure to provide correct MMIO descriptors.
+    pub const unsafe fn new(phys_mmio_descriptor: memory::mmu::MMIODescriptor<Physical>) -> Self {
         Self {
-            inner: IRQSafeNullLock::new(GPIOInner::new(mmio_start_addr)),
+            phys_mmio_descriptor,
+            virt_mmio_start_addr: AtomicUsize::new(0),
+            inner: IRQSafeNullLock::new(GPIOInner::new(
+                phys_mmio_descriptor.start_addr().into_usize(),
+            )),
         }
     }

@@ -213,4 +233,27 @@
     fn compatible(&self) -> &'static str {
         "BCM GPIO"
     }
+
+    unsafe fn init(&self) -> Result<(), &'static str> {
+        let virt_addr =
+            memory::mmu::kernel_map_mmio(self.compatible(), &self.phys_mmio_descriptor)?;
+
+        let mut r = &self.inner;
+        r.lock(|inner| inner.init(Some(virt_addr.into_usize())))?;
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

diff -uNr 14_exceptions_part2_peripheral_IRQs/src/bsp/device_driver/bcm/bcm2xxx_interrupt_controller/peripheral_ic.rs 15_virtual_mem_part2_mmio_remap/src/bsp/device_driver/bcm/bcm2xxx_interrupt_controller/peripheral_ic.rs
--- 14_exceptions_part2_peripheral_IRQs/src/bsp/device_driver/bcm/bcm2xxx_interrupt_controller/peripheral_ic.rs
+++ 15_virtual_mem_part2_mmio_remap/src/bsp/device_driver/bcm/bcm2xxx_interrupt_controller/peripheral_ic.rs
@@ -2,12 +2,14 @@
 //
 // Copyright (c) 2020 Andre Richter <andre.o.richter@gmail.com>

-//! Peripheral Interrupt regsler Driver.
+//! Peripheral Interrupt Controller Driver.

 use super::{InterruptController, PendingIRQs, PeripheralIRQ};
 use crate::{
     bsp::device_driver::common::MMIODerefWrapper,
-    exception, synchronization,
+    driver, exception, memory,
+    memory::mmu::Physical,
+    synchronization,
     synchronization::{IRQSafeNullLock, InitStateLock},
 };
 use register::{mmio::*, register_structs};
@@ -51,11 +53,13 @@

 /// Representation of the peripheral interrupt controller.
 pub struct PeripheralIC {
+    phys_mmio_descriptor: memory::mmu::MMIODescriptor<Physical>,
+
     /// Access to write registers is guarded with a lock.
     wo_registers: IRQSafeNullLock<WriteOnlyRegisters>,

     /// Register read access is unguarded.
-    ro_registers: ReadOnlyRegisters,
+    ro_registers: InitStateLock<ReadOnlyRegisters>,

     /// Stores registered IRQ handlers. Writable only during kernel init. RO afterwards.
     handler_table: InitStateLock<HandlerTable>,
@@ -70,21 +74,27 @@
     ///
     /// # Safety
     ///
-    /// - The user must ensure to provide a correct MMIO start address.
-    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
+    /// - The user must ensure to provide correct MMIO descriptors.
+    pub const unsafe fn new(phys_mmio_descriptor: memory::mmu::MMIODescriptor<Physical>) -> Self {
+        let addr = phys_mmio_descriptor.start_addr().into_usize();
+
         Self {
-            wo_registers: IRQSafeNullLock::new(WriteOnlyRegisters::new(mmio_start_addr)),
-            ro_registers: ReadOnlyRegisters::new(mmio_start_addr),
+            phys_mmio_descriptor,
+            wo_registers: IRQSafeNullLock::new(WriteOnlyRegisters::new(addr)),
+            ro_registers: InitStateLock::new(ReadOnlyRegisters::new(addr)),
             handler_table: InitStateLock::new([None; InterruptController::NUM_PERIPHERAL_IRQS]),
         }
     }

     /// Query the list of pending IRQs.
     fn pending_irqs(&self) -> PendingIRQs {
-        let pending_mask: u64 = (u64::from(self.ro_registers.PENDING_2.get()) << 32)
-            | u64::from(self.ro_registers.PENDING_1.get());
+        let mut r = &self.ro_registers;
+        r.read(|regs| {
+            let pending_mask: u64 =
+                (u64::from(regs.PENDING_2.get()) << 32) | u64::from(regs.PENDING_1.get());

-        PendingIRQs::new(pending_mask)
+            PendingIRQs::new(pending_mask)
+        })
     }
 }

@@ -93,6 +103,26 @@
 //------------------------------------------------------------------------------
 use synchronization::interface::{Mutex, ReadWriteEx};

+impl driver::interface::DeviceDriver for PeripheralIC {
+    fn compatible(&self) -> &'static str {
+        "BCM Peripheral Interrupt Controller"
+    }
+
+    unsafe fn init(&self) -> Result<(), &'static str> {
+        let virt_addr =
+            memory::mmu::kernel_map_mmio(self.compatible(), &self.phys_mmio_descriptor)?
+                .into_usize();
+
+        let mut r = &self.wo_registers;
+        r.lock(|regs| *regs = WriteOnlyRegisters::new(virt_addr));
+
+        let mut r = &self.ro_registers;
+        r.write(|regs| *regs = ReadOnlyRegisters::new(virt_addr));
+
+        Ok(())
+    }
+}
+
 impl exception::asynchronous::interface::IRQManager for PeripheralIC {
     type IRQNumberType = PeripheralIRQ;


diff -uNr 14_exceptions_part2_peripheral_IRQs/src/bsp/device_driver/bcm/bcm2xxx_interrupt_controller.rs 15_virtual_mem_part2_mmio_remap/src/bsp/device_driver/bcm/bcm2xxx_interrupt_controller.rs
--- 14_exceptions_part2_peripheral_IRQs/src/bsp/device_driver/bcm/bcm2xxx_interrupt_controller.rs
+++ 15_virtual_mem_part2_mmio_remap/src/bsp/device_driver/bcm/bcm2xxx_interrupt_controller.rs
@@ -6,7 +6,7 @@

 mod peripheral_ic;

-use crate::{driver, exception};
+use crate::{driver, exception, memory, memory::mmu::Physical};

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
+        _phys_local_mmio_descriptor: memory::mmu::MMIODescriptor<Physical>,
+        phys_periph_mmio_descriptor: memory::mmu::MMIODescriptor<Physical>,
+    ) -> Self {
         Self {
-            periph: peripheral_ic::PeripheralIC::new(periph_mmio_start_addr),
+            periph: peripheral_ic::PeripheralIC::new(phys_periph_mmio_descriptor),
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

diff -uNr 14_exceptions_part2_peripheral_IRQs/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs 15_virtual_mem_part2_mmio_remap/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
--- 14_exceptions_part2_peripheral_IRQs/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
+++ 15_virtual_mem_part2_mmio_remap/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
@@ -5,10 +5,13 @@
 //! PL011 UART driver.

 use crate::{
-    bsp, bsp::device_driver::common::MMIODerefWrapper, console, cpu, driver, exception,
-    synchronization, synchronization::IRQSafeNullLock,
+    bsp, bsp::device_driver::common::MMIODerefWrapper, console, cpu, driver, exception, memory,
+    memory::mmu::Physical, synchronization, synchronization::IRQSafeNullLock,
+};
+use core::{
+    fmt,
+    sync::atomic::{AtomicUsize, Ordering},
 };
-use core::fmt;
 use register::{mmio::*, register_bitfields, register_structs};

 //--------------------------------------------------------------------------------------------------
@@ -202,6 +205,8 @@

 /// Representation of the UART.
 pub struct PL011Uart {
+    phys_mmio_descriptor: memory::mmu::MMIODescriptor<Physical>,
+    virt_mmio_start_addr: AtomicUsize,
     inner: IRQSafeNullLock<PL011UartInner>,
     irq_number: bsp::device_driver::IRQNumber,
 }
@@ -232,7 +237,15 @@
     /// approximation we can get. A 5 modulo error margin is acceptable for UART and we're now at 0,01 modulo.
     ///
     /// This results in 8N1 and 230400 baud (we set the clock to 48 MHz in config.txt).
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
         // Turn it off temporarily.
         self.registers.CR.set(0);

@@ -249,6 +262,8 @@
         self.registers
             .CR
             .write(CR::UARTEN::Enabled + CR::TXE::Enabled + CR::RXE::Enabled);
+
+        Ok(())
     }

     /// Send a character.
@@ -318,13 +333,18 @@
     ///
     /// # Safety
     ///
-    /// - The user must ensure to provide a correct MMIO start address.
+    /// - The user must ensure to provide correct MMIO descriptors.
+    /// - The user must ensure to provide correct IRQ numbers.
     pub const unsafe fn new(
-        mmio_start_addr: usize,
+        phys_mmio_descriptor: memory::mmu::MMIODescriptor<Physical>,
         irq_number: bsp::device_driver::IRQNumber,
     ) -> Self {
         Self {
-            inner: IRQSafeNullLock::new(PL011UartInner::new(mmio_start_addr)),
+            phys_mmio_descriptor,
+            virt_mmio_start_addr: AtomicUsize::new(0),
+            inner: IRQSafeNullLock::new(PL011UartInner::new(
+                phys_mmio_descriptor.start_addr().into_usize(),
+            )),
             irq_number,
         }
     }
@@ -341,8 +361,14 @@
     }

     unsafe fn init(&self) -> Result<(), &'static str> {
+        let virt_addr =
+            memory::mmu::kernel_map_mmio(self.compatible(), &self.phys_mmio_descriptor)?;
+
         let mut r = &self.inner;
-        r.lock(|inner| inner.init());
+        r.lock(|inner| inner.init(Some(virt_addr.into_usize())))?;
+
+        self.virt_mmio_start_addr
+            .store(virt_addr.into_usize(), Ordering::Relaxed);

         Ok(())
     }
@@ -361,6 +387,16 @@

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

diff -uNr 14_exceptions_part2_peripheral_IRQs/src/bsp/raspberrypi/console.rs 15_virtual_mem_part2_mmio_remap/src/bsp/raspberrypi/console.rs
--- 14_exceptions_part2_peripheral_IRQs/src/bsp/raspberrypi/console.rs
+++ 15_virtual_mem_part2_mmio_remap/src/bsp/raspberrypi/console.rs
@@ -5,7 +5,7 @@
 //! BSP console facilities.

 use super::memory;
-use crate::{bsp::device_driver, console};
+use crate::{bsp::device_driver, console, cpu};
 use core::fmt;

 //--------------------------------------------------------------------------------------------------
@@ -23,11 +23,25 @@
 ///
 /// - Use only for printing during a panic.
 pub unsafe fn panic_console_out() -> impl fmt::Write {
-    let mut panic_gpio = device_driver::PanicGPIO::new(memory::map::mmio::GPIO_START);
-    let mut panic_uart = device_driver::PanicUart::new(memory::map::mmio::PL011_UART_START);
+    use crate::driver::interface::DeviceDriver;

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


diff -uNr 14_exceptions_part2_peripheral_IRQs/src/bsp/raspberrypi/driver.rs 15_virtual_mem_part2_mmio_remap/src/bsp/raspberrypi/driver.rs
--- 14_exceptions_part2_peripheral_IRQs/src/bsp/raspberrypi/driver.rs
+++ 15_virtual_mem_part2_mmio_remap/src/bsp/raspberrypi/driver.rs
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

diff -uNr 14_exceptions_part2_peripheral_IRQs/src/bsp/raspberrypi/link.ld 15_virtual_mem_part2_mmio_remap/src/bsp/raspberrypi/link.ld
--- 14_exceptions_part2_peripheral_IRQs/src/bsp/raspberrypi/link.ld
+++ 15_virtual_mem_part2_mmio_remap/src/bsp/raspberrypi/link.ld
@@ -42,6 +42,11 @@
         . += 8;
         __bss_end_inclusive = . - 8;
     }
+    . = ALIGN(65536);
+    __data_end = .;
+
+    __ro_size = __ro_end - __ro_start;
+    __data_size = __data_end - __ro_end;

     /DISCARD/ : { *(.comment*) }
 }

diff -uNr 14_exceptions_part2_peripheral_IRQs/src/bsp/raspberrypi/memory/mmu.rs 15_virtual_mem_part2_mmio_remap/src/bsp/raspberrypi/memory/mmu.rs
--- 14_exceptions_part2_peripheral_IRQs/src/bsp/raspberrypi/memory/mmu.rs
+++ 15_virtual_mem_part2_mmio_remap/src/bsp/raspberrypi/memory/mmu.rs
@@ -4,72 +4,128 @@

 //! BSP Memory Management Unit.

-use super::map as memory_map;
-use crate::memory::mmu::*;
-use core::ops::RangeInclusive;
+use crate::{
+    common,
+    memory::{
+        mmu as kernel_mmu,
+        mmu::{
+            interface, AccessPermissions, AttributeFields, Granule64KiB, MemAttributes, Page,
+            PageSliceDescriptor, Physical, Virtual,
+        },
+    },
+};

 //--------------------------------------------------------------------------------------------------
 // Public Definitions
 //--------------------------------------------------------------------------------------------------

-const NUM_MEM_RANGES: usize = 2;
-
-/// The virtual memory layout.
-///
-/// The layout must contain only special ranges, aka anything that is _not_ normal cacheable DRAM.
-/// It is agnostic of the paging granularity that the architecture's MMU will use.
-pub static LAYOUT: KernelVirtualLayout<{ NUM_MEM_RANGES }> = KernelVirtualLayout::new(
-    memory_map::END_INCLUSIVE,
-    [
-        TranslationDescriptor {
-            name: "Kernel code and RO data",
-            virtual_range: ro_range_inclusive,
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
+/// The translation granule chosen by this BSP. This will be used everywhere else in the kernel to
+/// derive respective data structures and their sizes. For example, the `crate::memory::mmu::Page`.
+pub type KernelGranule = Granule64KiB;

 //--------------------------------------------------------------------------------------------------
 // Private Code
 //--------------------------------------------------------------------------------------------------
+use interface::TranslationGranule;
+
+/// Helper function for calculating the number of pages the given parameter spans.
+const fn size_to_num_pages(size: usize) -> usize {
+    assert!(size > 0);
+    assert!(size modulo KernelGranule::SIZE == 0);
+
+    size >> KernelGranule::SHIFT
+}
+
+/// The boot core's stack.
+fn virt_stack_page_desc() -> PageSliceDescriptor<Virtual> {
+    let num_pages = size_to_num_pages(super::boot_core_stack_size());
+
+    PageSliceDescriptor::from_addr(super::virt_boot_core_stack_start(), num_pages)
+}
+
+/// The Read-Only (RO) pages of the kernel binary.
+fn virt_ro_page_desc() -> PageSliceDescriptor<Virtual> {
+    let num_pages = size_to_num_pages(super::ro_size());
+
+    PageSliceDescriptor::from_addr(super::virt_ro_start(), num_pages)
+}
+
+/// The data pages of the kernel binary.
+fn virt_data_page_desc() -> PageSliceDescriptor<Virtual> {
+    let num_pages = size_to_num_pages(super::data_size());

-fn ro_range_inclusive() -> RangeInclusive<usize> {
-    // Notice the subtraction to turn the exclusive end into an inclusive end.
-    #[allow(clippy::range_minus_one)]
-    RangeInclusive::new(super::ro_start(), super::ro_end() - 1)
+    PageSliceDescriptor::from_addr(super::virt_data_start(), num_pages)
 }

-fn mmio_range_inclusive() -> RangeInclusive<usize> {
-    RangeInclusive::new(memory_map::mmio::START, memory_map::mmio::END_INCLUSIVE)
+// The binary is still identity mapped, so we don't need to convert in the following.
+
+/// The boot core's stack.
+fn phys_stack_page_desc() -> PageSliceDescriptor<Physical> {
+    virt_stack_page_desc().into()
+}
+
+/// The Read-Only (RO) pages of the kernel binary.
+fn phys_ro_page_desc() -> PageSliceDescriptor<Physical> {
+    virt_ro_page_desc().into()
+}
+
+/// The data pages of the kernel binary.
+fn phys_data_page_desc() -> PageSliceDescriptor<Physical> {
+    virt_data_page_desc().into()
 }

 //--------------------------------------------------------------------------------------------------
 // Public Code
 //--------------------------------------------------------------------------------------------------

-/// Return the address space size in bytes.
-pub const fn addr_space_size() -> usize {
-    memory_map::END_INCLUSIVE + 1
+/// Pointer to the last page of the physical address space.
+pub fn phys_addr_space_end_page() -> *const Page<Physical> {
+    common::align_down(
+        super::phys_addr_space_end().into_usize(),
+        KernelGranule::SIZE,
+    ) as *const Page<_>
 }

-/// Return a reference to the virtual memory layout.
-pub fn virt_mem_layout() -> &'static KernelVirtualLayout<{ NUM_MEM_RANGES }> {
-    &LAYOUT
+/// Map the kernel binary.
+///
+/// # Safety
+///
+/// - Any miscalculation or attribute error will likely be fatal. Needs careful manual checking.
+pub unsafe fn kernel_map_binary() -> Result<(), &'static str> {
+    kernel_mmu::kernel_map_pages_at(
+        "Kernel boot-core stack",
+        &phys_stack_page_desc(),
+        &virt_stack_page_desc(),
+        &AttributeFields {
+            mem_attributes: MemAttributes::CacheableDRAM,
+            acc_perms: AccessPermissions::ReadWrite,
+            execute_never: true,
+        },
+    )?;
+
+    kernel_mmu::kernel_map_pages_at(
+        "Kernel code and RO data",
+        &phys_ro_page_desc(),
+        &virt_ro_page_desc(),
+        &AttributeFields {
+            mem_attributes: MemAttributes::CacheableDRAM,
+            acc_perms: AccessPermissions::ReadOnly,
+            execute_never: false,
+        },
+    )?;
+
+    kernel_mmu::kernel_map_pages_at(
+        "Kernel data and bss",
+        &phys_data_page_desc(),
+        &virt_data_page_desc(),
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
@@ -84,14 +140,12 @@
     /// Check alignment of the kernel's virtual memory layout sections.
     #[kernel_test]
     fn virt_mem_layout_sections_are_64KiB_aligned() {
-        const SIXTYFOUR_KIB: usize = 65536;
-
-        for i in LAYOUT.inner().iter() {
-            let start: usize = *(i.virtual_range)().start();
-            let end: usize = *(i.virtual_range)().end() + 1;
+        for i in [virt_stack_page_desc, virt_ro_page_desc, virt_data_page_desc].iter() {
+            let start: usize = i().start_addr().into_usize();
+            let end: usize = i().end_addr().into_usize();

-            assert_eq!(start modulo SIXTYFOUR_KIB, 0);
-            assert_eq!(end modulo SIXTYFOUR_KIB, 0);
+            assert_eq!(start modulo KernelGranule::SIZE, 0);
+            assert_eq!(end modulo KernelGranule::SIZE, 0);
             assert!(end >= start);
         }
     }
@@ -99,17 +153,18 @@
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
+            virt_stack_page_desc().into_usize_range_inclusive(),
+            virt_ro_page_desc().into_usize_range_inclusive(),
+            virt_data_page_desc().into_usize_range_inclusive(),
+        ];
+
+        for (i, first_range) in layout.iter().enumerate() {
+            for second_range in layout.iter().skip(i + 1) {
+                assert!(!first_range.contains(second_range.start()));
+                assert!(!first_range.contains(second_range.end()));
+                assert!(!second_range.contains(first_range.start()));
+                assert!(!second_range.contains(first_range.end()));
             }
         }
     }

diff -uNr 14_exceptions_part2_peripheral_IRQs/src/bsp/raspberrypi/memory.rs 15_virtual_mem_part2_mmio_remap/src/bsp/raspberrypi/memory.rs
--- 14_exceptions_part2_peripheral_IRQs/src/bsp/raspberrypi/memory.rs
+++ 15_virtual_mem_part2_mmio_remap/src/bsp/raspberrypi/memory.rs
@@ -3,9 +3,41 @@
 // Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

 //! BSP Memory Management.
+//!
+//! The physical memory layout after the kernel has been loaded by the Raspberry's firmware, which
+//! copies the binary to 0x8_0000:
+//!
+//! +---------------------------------------------+
+//! |                                             | 0x0
+//! | Unmapped                                    |
+//! |                                             | 0x6_FFFF
+//! +---------------------------------------------+
+//! | BOOT_CORE_STACK_START                       | 0x7_0000
+//! |                                             |            ^
+//! | ...                                         |            | Stack growth direction
+//! |                                             |            |
+//! | BOOT_CORE_STACK_END_INCLUSIVE               | 0x7_FFFF
+//! +---------------------------------------------+
+//! | RO_START == BOOT_CORE_STACK_END             | 0x8_0000
+//! |                                             |
+//! |                                             |
+//! | .text                                       |
+//! | .exception_vectors                          |
+//! | .rodata                                     |
+//! |                                             |
+//! | RO_END_INCLUSIVE                            | 0x8_0000 + __ro_size - 1
+//! +---------------------------------------------+
+//! | RO_END == DATA_START                        | 0x8_0000 + __ro_size
+//! |                                             |
+//! | .data                                       |
+//! | .bss                                        |
+//! |                                             |
+//! | DATA_END_INCLUSIVE                          | 0x8_0000 + __ro_size + __data_size - 1
+//! +---------------------------------------------+

 pub mod mmu;

+use crate::memory::mmu::{Address, Physical, Virtual};
 use core::{cell::UnsafeCell, ops::RangeInclusive};

 //--------------------------------------------------------------------------------------------------
@@ -17,34 +49,39 @@
     static __bss_start: UnsafeCell<u64>;
     static __bss_end_inclusive: UnsafeCell<u64>;
     static __ro_start: UnsafeCell<()>;
-    static __ro_end: UnsafeCell<()>;
+    static __ro_size: UnsafeCell<()>;
+    static __data_size: UnsafeCell<()>;
 }

 //--------------------------------------------------------------------------------------------------
 // Public Definitions
 //--------------------------------------------------------------------------------------------------

-/// The board's memory map.
+/// The board's physical memory map.
 #[rustfmt::skip]
 pub(super) mod map {
-    pub const END_INCLUSIVE:       usize = 0xFFFF_FFFF;
+    use super::*;

-    pub const BOOT_CORE_STACK_END: usize = 0x8_0000;
-
-    pub const GPIO_OFFSET:         usize = 0x0020_0000;
-    pub const UART_OFFSET:         usize = 0x0020_1000;
+    pub const BOOT_CORE_STACK_SIZE:                  usize = 0x1_0000;

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
@@ -52,13 +89,22 @@
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
@@ -71,8 +117,8 @@
 ///
 /// - Value is provided by the linker script and must be trusted as-is.
 #[inline(always)]
-fn ro_start() -> usize {
-    unsafe { __ro_start.get() as usize }
+fn virt_ro_start() -> Address<Virtual> {
+    Address::new(unsafe { __ro_start.get() as usize })
 }

 /// Size of the Read-Only (RO) range of the kernel binary.
@@ -81,8 +127,42 @@
 ///
 /// - Value is provided by the linker script and must be trusted as-is.
 #[inline(always)]
-fn ro_end() -> usize {
-    unsafe { __ro_end.get() as usize }
+fn ro_size() -> usize {
+    unsafe { __ro_size.get() as usize }
+}
+
+/// Start address of the data range.
+#[inline(always)]
+fn virt_data_start() -> Address<Virtual> {
+    virt_ro_start() + ro_size()
+}
+
+/// Size of the data range.
+///
+/// # Safety
+///
+/// - Value is provided by the linker script and must be trusted as-is.
+#[inline(always)]
+fn data_size() -> usize {
+    unsafe { __data_size.get() as usize }
+}
+
+/// Start address of the boot core's stack.
+#[inline(always)]
+fn virt_boot_core_stack_start() -> Address<Virtual> {
+    virt_ro_start() - map::BOOT_CORE_STACK_SIZE
+}
+
+/// Size of the boot core's stack.
+#[inline(always)]
+fn boot_core_stack_size() -> usize {
+    map::BOOT_CORE_STACK_SIZE
+}
+
+/// Exclusive end address of the physical address space.
+#[inline(always)]
+fn phys_addr_space_end() -> Address<Physical> {
+    map::END
 }

 //--------------------------------------------------------------------------------------------------
@@ -91,8 +171,10 @@

 /// Exclusive end address of the boot core's stack.
 #[inline(always)]
-pub fn boot_core_stack_end() -> usize {
-    map::BOOT_CORE_STACK_END
+pub fn phys_boot_core_stack_end() -> Address<Physical> {
+    // The binary is still identity mapped, so we don't need to convert here.
+    let end = virt_boot_core_stack_start().into_usize() + boot_core_stack_size();
+    Address::new(end)
 }

 /// Return the inclusive range spanning the .bss section.

diff -uNr 14_exceptions_part2_peripheral_IRQs/src/bsp/raspberrypi.rs 15_virtual_mem_part2_mmio_remap/src/bsp/raspberrypi.rs
--- 14_exceptions_part2_peripheral_IRQs/src/bsp/raspberrypi.rs
+++ 15_virtual_mem_part2_mmio_remap/src/bsp/raspberrypi.rs
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

diff -uNr 14_exceptions_part2_peripheral_IRQs/src/common.rs 15_virtual_mem_part2_mmio_remap/src/common.rs
--- 14_exceptions_part2_peripheral_IRQs/src/common.rs
+++ 15_virtual_mem_part2_mmio_remap/src/common.rs
@@ -0,0 +1,21 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2020 Andre Richter <andre.o.richter@gmail.com>
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

diff -uNr 14_exceptions_part2_peripheral_IRQs/src/driver.rs 15_virtual_mem_part2_mmio_remap/src/driver.rs
--- 14_exceptions_part2_peripheral_IRQs/src/driver.rs
+++ 15_virtual_mem_part2_mmio_remap/src/driver.rs
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

diff -uNr 14_exceptions_part2_peripheral_IRQs/src/lib.rs 15_virtual_mem_part2_mmio_remap/src/lib.rs
--- 14_exceptions_part2_peripheral_IRQs/src/lib.rs
+++ 15_virtual_mem_part2_mmio_remap/src/lib.rs
@@ -113,6 +113,7 @@

 #![allow(incomplete_features)]
 #![feature(asm)]
+#![feature(const_fn)]
 #![feature(const_generics)]
 #![feature(const_panic)]
 #![feature(core_intrinsics)]
@@ -137,6 +138,7 @@
 mod synchronization;

 pub mod bsp;
+pub mod common;
 pub mod console;
 pub mod cpu;
 pub mod driver;

diff -uNr 14_exceptions_part2_peripheral_IRQs/src/main.rs 15_virtual_mem_part2_mmio_remap/src/main.rs
--- 14_exceptions_part2_peripheral_IRQs/src/main.rs
+++ 15_virtual_mem_part2_mmio_remap/src/main.rs
@@ -26,21 +26,34 @@
 #[no_mangle]
 unsafe fn kernel_init() -> ! {
     use driver::interface::DriverManager;
-    use memory::mmu::interface::MMU;

     exception::handling_init();

-    if let Err(string) = memory::mmu::mmu().init() {
-        panic!("MMU: {}", string);
+    if let Err(string) = memory::mmu::kernel_map_binary_and_enable_mmu() {
+        panic!("Enabling MMU failed: {}", string);
     }
+    // Printing will silently fail fail from here on, because the driver's MMIO is not remapped yet.

-    for i in bsp::driver::driver_manager().all_device_drivers().iter() {
+    // Bring up the drivers needed for printing first.
+    for i in bsp::driver::driver_manager()
+        .early_print_device_drivers()
+        .iter()
+    {
+        // Any encountered errors cannot be printed yet, obviously, so just safely park the CPU.
+        i.init().unwrap_or_else(|_| cpu::wait_forever());
+    }
+    bsp::driver::driver_manager().post_early_print_device_driver_init();
+    // Printing available again from here on.
+
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
@@ -66,8 +79,8 @@

     info!("Booting on: {}", bsp::board_name());

-    info!("MMU online. Special regions:");
-    bsp::memory::mmu::virt_mem_layout().print_layout();
+    info!("MMU online:");
+    memory::mmu::kernel_print_mappings();

     let (_, privilege_level) = exception::current_privilege_level();
     info!("Current privilege level: {}", privilege_level);

diff -uNr 14_exceptions_part2_peripheral_IRQs/src/memory/mmu/mapping_record.rs 15_virtual_mem_part2_mmio_remap/src/memory/mmu/mapping_record.rs
--- 14_exceptions_part2_peripheral_IRQs/src/memory/mmu/mapping_record.rs
+++ 15_virtual_mem_part2_mmio_remap/src/memory/mmu/mapping_record.rs
@@ -0,0 +1,224 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2020 Andre Richter <andre.o.richter@gmail.com>
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
+        phys_pages: &PageSliceDescriptor<Physical>,
+        virt_pages: &PageSliceDescriptor<Virtual>,
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
+        phys_pages: &PageSliceDescriptor<Physical>,
+        virt_pages: &PageSliceDescriptor<Virtual>,
+        attr: &AttributeFields,
+    ) -> Result<(), &'static str> {
+        let x = self.find_next_free()?;
+
+        *x = Some(MappingRecordEntry::new(name, phys_pages, virt_pages, attr));
+        Ok(())
+    }
+
+    pub fn print(&self) {
+        const KIB_RSHIFT: u32 = 10; // log2(1024).
+        const MIB_RSHIFT: u32 = 20; // log2(1024 * 1024).
+
+        info!("      -----------------------------------------------------------------------------------------------------------------");
+        info!(
+            "      {:^24}     {:^24}   {:^7}   {:^9}   {:^35}",
+            "Virtual", "Physical", "Size", "Attr", "Entity"
+        );
+        info!("      -----------------------------------------------------------------------------------------------------------------");
+
+        for i in self
+            .inner
+            .iter()
+            .filter(|x| x.is_some())
+            .map(|x| x.unwrap())
+        {
+            let virt_start = i.virt_start_addr.into_usize();
+            let virt_end_inclusive = virt_start + i.phys_pages.size() - 1;
+            let phys_start = i.phys_pages.start_addr().into_usize();
+            let phys_end_inclusive = i.phys_pages.end_addr_inclusive().into_usize();
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
+                "      {:#011X}..{:#011X} --> {:#011X}..{:#011X} | \
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
+                        "                                                                                  | {}",
+                        additional_user
+                    );
+                }
+            }
+        }
+
+        info!("      -----------------------------------------------------------------------------------------------------------------");
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
+    phys_pages: &PageSliceDescriptor<Physical>,
+    virt_pages: &PageSliceDescriptor<Virtual>,
+    attr: &AttributeFields,
+) -> Result<(), &'static str> {
+    let mut m = &KERNEL_MAPPING_RECORD;
+    m.write(|mr| mr.add(name, phys_pages, virt_pages, attr))
+}
+
+pub fn kernel_find_and_insert_mmio_duplicate(
+    phys_mmio_descriptor: &MMIODescriptor<Physical>,
+    new_user: &'static str,
+) -> Option<Address<Virtual>> {
+    let phys_pages: PageSliceDescriptor<Physical> = phys_mmio_descriptor.clone().into();
+
+    let mut m = &KERNEL_MAPPING_RECORD;
+    m.write(|mr| {
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
+    let mut m = &KERNEL_MAPPING_RECORD;
+    m.read(|mr| mr.print());
+}

diff -uNr 14_exceptions_part2_peripheral_IRQs/src/memory/mmu/types.rs 15_virtual_mem_part2_mmio_remap/src/memory/mmu/types.rs
--- 14_exceptions_part2_peripheral_IRQs/src/memory/mmu/types.rs
+++ 15_virtual_mem_part2_mmio_remap/src/memory/mmu/types.rs
@@ -0,0 +1,283 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2020 Andre Richter <andre.o.richter@gmail.com>
+
+//! Memory Management Unit Types.
+
+use crate::{bsp, common};
+use core::{convert::From, marker::PhantomData, ops::RangeInclusive};
+
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+use super::interface::TranslationGranule;
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
+    _address_type: PhantomData<ATYPE>,
+}
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
+pub struct MMIODescriptor<ATYPE: AddressType> {
+    start_addr: Address<ATYPE>,
+    size: usize,
+}
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+impl AddressType for Physical {}
+impl AddressType for Virtual {}
+
+//------------------------------------------------------------------------------
+// Address
+//------------------------------------------------------------------------------
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
+    /// Return a non-mutable slice of Pages.
+    ///
+    /// # Safety
+    ///
+    /// - Same as applies for `core::slice::from_raw_parts`.
+    pub unsafe fn as_slice(&self) -> &[Page<ATYPE>] {
+        core::slice::from_raw_parts(self.first_page_ptr(), self.num_pages)
+    }
+
+    /// Return the inclusive address range of the slice.
+    pub fn into_usize_range_inclusive(self) -> RangeInclusive<usize> {
+        RangeInclusive::new(
+            self.start_addr().into_usize(),
+            self.end_addr_inclusive().into_usize(),
+        )
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
+impl<ATYPE: AddressType> From<MMIODescriptor<ATYPE>> for PageSliceDescriptor<ATYPE> {
+    fn from(desc: MMIODescriptor<ATYPE>) -> Self {
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
+impl<ATYPE: AddressType> MMIODescriptor<ATYPE> {
+    /// Create an instance.
+    pub const fn new(start_addr: Address<ATYPE>, size: usize) -> Self {
+        assert!(size > 0);
+
+        Self { start_addr, size }
+    }
+
+    /// Return the start address.
+    pub const fn start_addr(&self) -> Address<ATYPE> {
+        self.start_addr
+    }
+
+    /// Return the inclusive end address.
+    pub fn end_addr_inclusive(&self) -> Address<ATYPE> {
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

diff -uNr 14_exceptions_part2_peripheral_IRQs/src/memory/mmu.rs 15_virtual_mem_part2_mmio_remap/src/memory/mmu.rs
--- 14_exceptions_part2_peripheral_IRQs/src/memory/mmu.rs
+++ 15_virtual_mem_part2_mmio_remap/src/memory/mmu.rs
@@ -3,23 +3,18 @@
 // Copyright (c) 2020 Andre Richter <andre.o.richter@gmail.com>

 //! Memory Management Unit.
-//!
-//! In order to decouple `BSP` and `arch` parts of the MMU code (to keep them pluggable), this file
-//! provides types for composing an architecture-agnostic description of the kernel 's virtual
-//! memory layout.
-//!
-//! The `BSP` provides such a description through the `bsp::memory::mmu::virt_mem_layout()`
-//! function.
-//!
-//! The `MMU` driver of the `arch` code uses `bsp::memory::mmu::virt_mem_layout()` to compile and
-//! install respective translation tables.

 #[cfg(target_arch = "aarch64")]
 #[path = "../_arch/aarch64/memory/mmu.rs"]
 mod arch_mmu;
 pub use arch_mmu::*;

-use core::{fmt, ops::RangeInclusive};
+mod mapping_record;
+mod types;
+
+use crate::{bsp, synchronization, warn};
+
+pub use types::*;

 //--------------------------------------------------------------------------------------------------
 // Public Definitions
@@ -27,178 +22,229 @@

 /// Memory Management interfaces.
 pub mod interface {
+    use super::*;
+
+    /// Describes the characteristics of a translation granule.
+    #[allow(missing_docs)]
+    pub trait TranslationGranule {
+        const SIZE: usize;
+        const MASK: usize = Self::SIZE - 1;
+        const SHIFT: usize;
+    }
+
+    /// Translation table operations.
+    pub trait TranslationTable {
+        /// Anything that needs to run before any of the other provided functions can be used.
+        ///
+        /// # Safety
+        ///
+        /// - Implementor must ensure that this function can run only once or is harmless if invoked
+        ///   multiple times.
+        unsafe fn init(&mut self);
+
+        /// The translation table's base address to be used for programming the MMU.
+        fn phys_base_address(&self) -> Address<Physical>;
+
+        /// Map the given physical pages to the given virtual pages.
+        ///
+        /// # Safety
+        ///
+        /// - Using wrong attributes can cause multiple issues of different nature in the system.
+        /// - It is not required that the architectural implementation prevents aliasing. That is,
+        ///   mapping to the same physical memory using multiple virtual addresses, which would
+        ///   break Rust's ownership assumptions. This should be protected against in this module
+        ///   (the kernel's generic MMU code).
+        unsafe fn map_pages_at(
+            &mut self,
+            phys_pages: &PageSliceDescriptor<Physical>,
+            virt_pages: &PageSliceDescriptor<Virtual>,
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

     /// MMU functions.
     pub trait MMU {
-        /// Called by the kernel during early init. Supposed to take the translation tables from the
-        /// `BSP`-supplied `virt_mem_layout()` and install/activate them for the respective MMU.
+        /// Turns on the MMU.
         ///
         /// # Safety
         ///
+        /// - Must only be called after the kernel translation tables have been init()'ed.
         /// - Changes the HW's global state.
-        unsafe fn init(&self) -> Result<(), &'static str>;
+        unsafe fn enable(
+            &self,
+            phys_kernel_table_base_addr: Address<Physical>,
+        ) -> Result<(), &'static str>;
     }
 }

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
+//--------------------------------------------------------------------------------------------------
+// Private Code
+//--------------------------------------------------------------------------------------------------
+use interface::{TranslationTable, MMU};
+use synchronization::interface::ReadWriteEx;

-/// Architecture agnostic access permissions.
-#[allow(missing_docs)]
-#[derive(Copy, Clone)]
-pub enum AccessPermissions {
-    ReadOnly,
-    ReadWrite,
-}
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
+    phys_pages: &PageSliceDescriptor<Physical>,
+    virt_pages: &PageSliceDescriptor<Virtual>,
+    attr: &AttributeFields,
+) -> Result<(), &'static str> {
+    arch_mmu::kernel_translation_tables()
+        .write(|tables| tables.map_pages_at(phys_pages, virt_pages, attr))?;

-/// Collection of memory attributes.
-#[allow(missing_docs)]
-#[derive(Copy, Clone)]
-pub struct AttributeFields {
-    pub mem_attributes: MemAttributes,
-    pub acc_perms: AccessPermissions,
-    pub execute_never: bool,
-}
+    if let Err(x) = mapping_record::kernel_add(name, phys_pages, virt_pages, attr) {
+        warn!("{}", x);
+    }

-/// Architecture agnostic descriptor for a memory range.
-#[allow(missing_docs)]
-pub struct TranslationDescriptor {
-    pub name: &'static str,
-    pub virtual_range: fn() -> RangeInclusive<usize>,
-    pub physical_range_translation: Translation,
-    pub attribute_fields: AttributeFields,
+    Ok(())
 }

-/// Type for expressing the kernel's virtual memory layout.
-pub struct KernelVirtualLayout<const NUM_SPECIAL_RANGES: usize> {
-    /// The last (inclusive) address of the address space.
-    max_virt_addr_inclusive: usize,
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+use interface::TranslationGranule;

-    /// Array of descriptors for non-standard (normal cacheable DRAM) memory regions.
-    inner: [TranslationDescriptor; NUM_SPECIAL_RANGES],
+/// Raw mapping of virtual to physical pages in the kernel translation tables.
+///
+/// Prevents mapping into the MMIO range of the tables.
+///
+/// # Safety
+///
+/// - See `kernel_map_pages_at_unchecked()`.
+/// - Does not prevent aliasing. Currently, we have to trust the callers.
+pub unsafe fn kernel_map_pages_at(
+    name: &'static str,
+    phys_pages: &PageSliceDescriptor<Physical>,
+    virt_pages: &PageSliceDescriptor<Virtual>,
+    attr: &AttributeFields,
+) -> Result<(), &'static str> {
+    let is_mmio = arch_mmu::kernel_translation_tables()
+        .read(|tables| tables.is_virt_page_slice_mmio(virt_pages));
+    if is_mmio {
+        return Err("Attempt to manually map into MMIO region");
+    }
+
+    kernel_map_pages_at_unchecked(name, phys_pages, virt_pages, attr)?;
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
+    phys_mmio_descriptor: &MMIODescriptor<Physical>,
+) -> Result<Address<Virtual>, &'static str> {
+    let phys_pages: PageSliceDescriptor<Physical> = phys_mmio_descriptor.clone().into();
+    let offset_into_start_page =
+        phys_mmio_descriptor.start_addr().into_usize() & bsp::memory::mmu::KernelGranule::MASK;
+
+    // Check if an identical page slice has been mapped for another driver. If so, reuse it.
+    let virt_addr = if let Some(addr) =
+        mapping_record::kernel_find_and_insert_mmio_duplicate(phys_mmio_descriptor, name)
+    {
+        addr
+    // Otherwise, allocate a new virtual page slice and map it.
+    } else {
+        let virt_pages: PageSliceDescriptor<Virtual> = arch_mmu::kernel_translation_tables()
+            .write(|tables| tables.next_mmio_virt_page_slice(phys_pages.num_pages()))?;
+
+        kernel_map_pages_at_unchecked(
+            name,
+            &phys_pages,
+            &virt_pages,
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
+/// Map the kernel's binary and enable the MMU.
+///
+/// # Safety
+///
+/// - Crucial function during kernel init. Changes the the complete memory view of the processor.
+pub unsafe fn kernel_map_binary_and_enable_mmu() -> Result<(), &'static str> {
+    let phys_base_addr = arch_mmu::kernel_translation_tables().write(|tables| {
+        tables.init();
+        tables.phys_base_address()
+    });
+
+    bsp::memory::mmu::kernel_map_binary()?;
+    arch_mmu::mmu().enable(phys_base_addr)
+}
+
+/// Human-readable print of all recorded kernel mappings.
+pub fn kernel_print_mappings() {
+    mapping_record::kernel_print()
 }

 //--------------------------------------------------------------------------------------------------
-// Public Code
+// Testing
 //--------------------------------------------------------------------------------------------------

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
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use test_macros::kernel_test;
+
+    /// Sanity checks for the kernel TranslationTable implementation.
+    #[kernel_test]
+    fn translationtable_implementation_sanity() {
+        // Need to take care that `tables` fits into the stack.
+        let mut tables = MinSizeArchTranslationTable::new();
+
+        unsafe { tables.init() };
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
     }
 }

diff -uNr 14_exceptions_part2_peripheral_IRQs/tests/02_exception_sync_page_fault.rs 15_virtual_mem_part2_mmio_remap/tests/02_exception_sync_page_fault.rs
--- 14_exceptions_part2_peripheral_IRQs/tests/02_exception_sync_page_fault.rs
+++ 15_virtual_mem_part2_mmio_remap/tests/02_exception_sync_page_fault.rs
@@ -21,7 +21,7 @@

 #[no_mangle]
 unsafe fn kernel_init() -> ! {
-    use memory::mmu::interface::MMU;
+    use libkernel::driver::interface::DriverManager;

     bsp::console::qemu_bring_up_console();

@@ -30,10 +30,22 @@

     exception::handling_init();

-    if let Err(string) = memory::mmu::mmu().init() {
-        println!("MMU: {}", string);
+    if let Err(string) = memory::mmu::kernel_map_binary_and_enable_mmu() {
+        println!("Enabling MMU failed: {}", string);
         cpu::qemu_exit_failure()
     }
+    // Printing will silently fail fail from here on, because the driver's MMIO is not remapped yet.
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
