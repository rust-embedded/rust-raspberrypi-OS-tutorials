# Tutorial 19 - Kernel Heap

## tl;dr

- A global heap for the kernel is added, which enables runtime dynamic memory allocation (`Box`,
  `Vec`, etc.).
- Heap memory management is using a `linked list allocator`.
- A `debug!` printing macro is added that is only effective when `make` is invoked with
  `DEBUG_PRINTS=y`.

## Table of Contents

- [Introduction](#introduction)
- [Implementation](#implementation)
  - [Debug Prints](#debug-prints)
  - [Pre-UART Console Output](#pre-uart-console-output)
- [Test it](#test-it)
- [Diff to previous](#diff-to-previous)

## Introduction

The kernel is finally in a good place to add dynamic memory management. The entire kernel runs in
the higher half of the address space by now, and it has decent backtracing support, which can be
leveraged to get rich tracing/debugging support for heap allocations.

Although it is a vital part of implementing a heap, this tutorial will **not** cover
`allocation/deallocation` of heap memory. Instead, we will re-use [@phil-opp]'s excellent
[`linked_list_allocator`]. The reason is that while dynamic memory allocation algorithms are an
interesting topic, there would not be much added benefit in implementing a `linked list allocator`
of our own, since it would turn out very similar to what Philipp and the other contributors have
implemented already. So we might just re-use that, even more so because it can be plugged seamlessly
into our kernel. [@phil-opp] has also written two great articles on [Heap Allocation] and [Allocator
Designs]. I really recommend to read those now before continuing with this tutorial.

[@phil-opp]: https://github.com/phil-opp
[`linked_list_allocator`]: https://crates.io/crates/linked_list_allocator
[Heap Allocation]: https://os.phil-opp.com/heap-allocation/
[Allocator Designs]: https://os.phil-opp.com/allocator-designs/

That being said, what this tutorial text will cover is supporting changes for _enabling_ the
linked_list_allocator, and changes to kernel code leveraging the heap.

## Implementation

First of all, we need to reserve some DRAM for the heap. Traditionally, this is done in the `linker
script`. We place it after the `.data` section and before the `MMIO remap` section.

```ld.s
    __data_end_exclusive = .;

    /***********************************************************************************************
    * Heap
    ***********************************************************************************************/
    __heap_start = .;
    .heap (NOLOAD) :
    {
        . += 16 * 1024 * 1024;
    } :segment_heap
    __heap_end_exclusive = .;

    ASSERT((. & PAGE_MASK) == 0, "Heap is not page aligned")

    /***********************************************************************************************
    * MMIO Remap Reserved
    ***********************************************************************************************/
    __mmio_remap_start = .;
```

In the Rust code, the heap properties can now be queried using the added BSP-function
`bsp::memory::mmu::virt_heap_region()`. The heap allocator itself is added in
`src/memory/heap_alloc.rs`. There, we add the `linked_list_allocator`, wrap it into an
`IRQSafeNullock`, and instantiate it the wrapper in a `static`. This way, global access to the
allocator becomes concurrency-safe:

```rust
use linked_list_allocator::Heap as LinkedListHeap;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// A heap allocator that can be lazyily initialized.
pub struct HeapAllocator {
    inner: IRQSafeNullLock<LinkedListHeap>,
}

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

#[global_allocator]
static KERNEL_HEAP_ALLOCATOR: HeapAllocator = HeapAllocator::new();
```

All that is left to do now is to implement the [`GlobalAlloc`] trait for `HeapAllocator`:

[`GlobalAlloc`]: https://doc.rust-lang.org/stable/core/alloc/trait.GlobalAlloc.html

```rust
unsafe impl GlobalAlloc for HeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let result = KERNEL_HEAP_ALLOCATOR
            .inner
            .lock(|inner| inner.allocate_first_fit(layout).ok());

        match result {
            None => core::ptr::null_mut(),
            Some(allocation) => {
                let ptr = allocation.as_ptr();

                debug_print_alloc_dealloc("Allocation", ptr, layout);

                ptr
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        KERNEL_HEAP_ALLOCATOR
            .inner
            .lock(|inner| inner.deallocate(core::ptr::NonNull::new_unchecked(ptr), layout));

        debug_print_alloc_dealloc("Free", ptr, layout);
    }
}
```

During kernel init, `kernel_init_heap_allocator()` will be called, which basically points the
wrapped allocator to the heap that we defined earlier:

```rust
/// Query the BSP for the heap region and initialize the kernel's heap allocator with it.
pub fn kernel_init_heap_allocator() {
    static INIT_DONE: AtomicBool = AtomicBool::new(false);
    if INIT_DONE.load(Ordering::Relaxed) {
        warn!("Already initialized");
        return;
    }

    let region = bsp::memory::mmu::virt_heap_region();

    KERNEL_HEAP_ALLOCATOR.inner.lock(|inner| unsafe {
        inner.init(region.start_addr().as_usize() as *mut u8, region.size())
    });

    INIT_DONE.store(true, Ordering::Relaxed);
}
```

That's it already! We can now use `Box`, `Vec` and friends 🥳.

### Debug Prints

You might have noticed the `debug_print_alloc_dealloc()` calls in above's snippet. Under the hood,
this function makes use of the `debug!` macro that has been added in this tutorial. This macro will
only print to the console when `make` is invoked with the `ENV` variable `DEBUG_PRINTS` set to
"**y**". As you can see in the following snippet, this enables rich debug output for heap
allocations and deallocations, containing information such as `size`, `start` and `end exclusive`
addresses, as well as a backtrace that shows from where the (de)allocation originated.

```console
$ DEBUG_PRINTS=y make qemu

[...]

<D   0.040505> Kernel Heap: Allocation
      Size:     0x10 (16 Byte)
      Start:    0xffff_ffff_c00a_0010
      End excl: 0xffff_ffff_c00a_0020

      Backtrace:
      ----------------------------------------------------------------------------------------------
          Address            Function containing address
      ----------------------------------------------------------------------------------------------
       1. ffffffffc000cdf8 | <libkernel::bsp::device_driver::bcm::bcm2xxx_pl011_uart::PL011Uart as libkernel::console::interface::Write>::write_fmt
       2. ffffffffc000b4f8 | <libkernel::memory::heap_alloc::HeapAllocator as core::alloc::global::GlobalAlloc>::alloc
       3. ffffffffc000d940 | libkernel::memory::mmu::mapping_record::kernel_add
       4. ffffffffc000adec | libkernel::bsp::raspberrypi::memory::mmu::kernel_add_mapping_records_for_precomputed
       5. ffffffffc00016ac | kernel_init
      ----------------------------------------------------------------------------------------------

[    0.042872] mingo version 0.19.0
[    0.043080] Booting on: Raspberry Pi 3
```

### Pre-UART Console Output

Having a heap allows us to simplify a few modules by switching static-length arrays to the dynamic
`Vec` data structure. Examples are the `interrupt controller drivers` for their handler tables,
`src/memory/mmu/mapping_record.rs` for bookkeeping virtual memory mappings and the `BSP driver
manager` for its instantiated device drivers.

However, many of those allocations happen already **before** the UART driver comes online.
Therefore, a lot of the (de)allocation debug prints would go into the void with the way pre-UART
prints have been handled so far, which is undesirable. To solve this problem, the kernel's initial
(aka pre-UART) console is now not a `NullConsole` anymore, but a `BufferConsole`. The latter owns a
small static array of `chars`, that records any console prints before the actual UART driver comes
online. Once the UART driver is registered in the kernel to become the default console, the first
thing that is done is to print any buffered records of the `BufferConsole`:

```rust
pub fn register_console(new_console: &'static (dyn interface::All + Sync)) {
    CUR_CONSOLE.write(|con| *con = new_console);

    static FIRST_SWITCH: InitStateLock<bool> = InitStateLock::new(true);
    FIRST_SWITCH.write(|first| {
        if *first == true {
            *first = false;

            buffer_console::BUFFER_CONSOLE.dump();
        }
    });
}
```

`BUFFER_CONSOLE.dump()` just drains its buffer to using the newly registered console.

## Test it

If compiled without `DEBUG_PRINTS`, the heap can be observed in the mapping overview and through the
newly added usage statistics:

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
[MP] ⏩ Pushing 320 KiB ======================================🦀 100% 106 KiB/s Time: 00:00:03
[ML] Loaded! Executing the payload now

[    3.572716] mingo version 0.19.0
[    3.572924] Booting on: Raspberry Pi 3
[    3.573379] MMU online:
[    3.573672]       -------------------------------------------------------------------------------------------------------------------------------------------
[    3.575416]                         Virtual                                   Physical               Size       Attr                    Entity
[    3.577160]       -------------------------------------------------------------------------------------------------------------------------------------------
[    3.578905]       0xffff_ffff_c000_0000..0xffff_ffff_c001_ffff --> 0x00_0008_0000..0x00_0009_ffff | 128 KiB | C   RO X  | Kernel code and RO data
[    3.580519]       0xffff_ffff_c002_0000..0xffff_ffff_c009_ffff --> 0x00_000a_0000..0x00_0011_ffff | 512 KiB | C   RW XN | Kernel data and bss
[    3.582089]       0xffff_ffff_c00a_0000..0xffff_ffff_c109_ffff --> 0x00_0012_0000..0x00_0111_ffff |  16 MiB | C   RW XN | Kernel heap
[    3.583573]       0xffff_ffff_c10a_0000..0xffff_ffff_c10a_ffff --> 0x00_3f20_0000..0x00_3f20_ffff |  64 KiB | Dev RW XN | BCM PL011 UART
[    3.585090]                                                                                                             | BCM GPIO
[    3.586542]       0xffff_ffff_c10b_0000..0xffff_ffff_c10b_ffff --> 0x00_3f00_0000..0x00_3f00_ffff |  64 KiB | Dev RW XN | BCM Interrupt Controller
[    3.588167]       0xffff_ffff_c18b_0000..0xffff_ffff_c192_ffff --> 0x00_0000_0000..0x00_0007_ffff | 512 KiB | C   RW XN | Kernel boot-core stack
[    3.589770]       -------------------------------------------------------------------------------------------------------------------------------------------
[    3.591515] Current privilege level: EL1

[...]

[    3.597624] Kernel heap:
[    3.597928]       Used: 2512 Byte (3 KiB)
[    3.598415]       Free: 16774704 Byte (16 MiB)
[    3.598957] Echoing input now
```

## Diff to previous
```diff

diff -uNr 18_backtrace/kernel/Cargo.toml 19_kernel_heap/kernel/Cargo.toml
--- 18_backtrace/kernel/Cargo.toml
+++ 19_kernel_heap/kernel/Cargo.toml
@@ -1,11 +1,12 @@
 [package]
 name = "mingo"
-version = "0.18.0"
+version = "0.19.0"
 authors = ["Andre Richter <andre.o.richter@gmail.com>"]
 edition = "2021"

 [features]
 default = []
+debug_prints = []
 bsp_rpi3 = ["tock-registers"]
 bsp_rpi4 = ["tock-registers"]
 test_build = ["qemu-exit"]
@@ -17,6 +18,7 @@
 [dependencies]
 test-types = { path = "../libraries/test-types" }
 debug-symbol-types = { path = "../libraries/debug-symbol-types" }
+linked_list_allocator = { version = "0.10.x", default-features = false, features = ["const_mut_refs"] }

 # Optional dependencies
 tock-registers = { version = "0.8.x", default-features = false, features = ["register_types"], optional = true }

diff -uNr 18_backtrace/kernel/src/bsp/device_driver/arm/gicv2.rs 19_kernel_heap/kernel/src/bsp/device_driver/arm/gicv2.rs
--- 18_backtrace/kernel/src/bsp/device_driver/arm/gicv2.rs
+++ 19_kernel_heap/kernel/src/bsp/device_driver/arm/gicv2.rs
@@ -86,13 +86,13 @@
     synchronization,
     synchronization::InitStateLock,
 };
+use alloc::vec::Vec;

 //--------------------------------------------------------------------------------------------------
 // Private Definitions
 //--------------------------------------------------------------------------------------------------

-type HandlerTable = [Option<exception::asynchronous::IRQHandlerDescriptor<IRQNumber>>;
-    IRQNumber::MAX_INCLUSIVE + 1];
+type HandlerTable = Vec<Option<exception::asynchronous::IRQHandlerDescriptor<IRQNumber>>>;

 //--------------------------------------------------------------------------------------------------
 // Public Definitions
@@ -118,7 +118,7 @@
 //--------------------------------------------------------------------------------------------------

 impl GICv2 {
-    const MAX_IRQ_NUMBER: usize = 300; // Normally 1019, but keep it lower to save some space.
+    const MAX_IRQ_NUMBER: usize = 1019;

     pub const COMPATIBLE: &'static str = "GICv2 (ARM Generic Interrupt Controller v2)";

@@ -134,7 +134,7 @@
         Self {
             gicd: gicd::GICD::new(gicd_mmio_start_addr),
             gicc: gicc::GICC::new(gicc_mmio_start_addr),
-            handler_table: InitStateLock::new([None; IRQNumber::MAX_INCLUSIVE + 1]),
+            handler_table: InitStateLock::new(Vec::new()),
         }
     }
 }
@@ -152,6 +152,9 @@
     }

     unsafe fn init(&self) -> Result<(), &'static str> {
+        self.handler_table
+            .write(|table| table.resize(IRQNumber::MAX_INCLUSIVE + 1, None));
+
         if bsp::cpu::BOOT_CORE_ID == cpu::smp::core_id() {
             self.gicd.boot_core_init();
         }

diff -uNr 18_backtrace/kernel/src/bsp/device_driver/bcm/bcm2xxx_interrupt_controller/peripheral_ic.rs 19_kernel_heap/kernel/src/bsp/device_driver/bcm/bcm2xxx_interrupt_controller/peripheral_ic.rs
--- 18_backtrace/kernel/src/bsp/device_driver/bcm/bcm2xxx_interrupt_controller/peripheral_ic.rs
+++ 19_kernel_heap/kernel/src/bsp/device_driver/bcm/bcm2xxx_interrupt_controller/peripheral_ic.rs
@@ -16,6 +16,7 @@
     synchronization,
     synchronization::{IRQSafeNullLock, InitStateLock},
 };
+use alloc::vec::Vec;
 use tock_registers::{
     interfaces::{Readable, Writeable},
     register_structs,
@@ -52,8 +53,7 @@
 /// Abstraction for the ReadOnly parts of the associated MMIO registers.
 type ReadOnlyRegisters = MMIODerefWrapper<RORegisterBlock>;

-type HandlerTable = [Option<exception::asynchronous::IRQHandlerDescriptor<PeripheralIRQ>>;
-    PeripheralIRQ::MAX_INCLUSIVE + 1];
+type HandlerTable = Vec<Option<exception::asynchronous::IRQHandlerDescriptor<PeripheralIRQ>>>;

 //--------------------------------------------------------------------------------------------------
 // Public Definitions
@@ -85,10 +85,16 @@
         Self {
             wo_registers: IRQSafeNullLock::new(WriteOnlyRegisters::new(mmio_start_addr)),
             ro_registers: ReadOnlyRegisters::new(mmio_start_addr),
-            handler_table: InitStateLock::new([None; PeripheralIRQ::MAX_INCLUSIVE + 1]),
+            handler_table: InitStateLock::new(Vec::new()),
         }
     }

+    /// Called by the kernel to bring up the device.
+    pub fn init(&self) {
+        self.handler_table
+            .write(|table| table.resize(PeripheralIRQ::MAX_INCLUSIVE + 1, None));
+    }
+
     /// Query the list of pending IRQs.
     fn pending_irqs(&self) -> PendingIRQs {
         let pending_mask: u64 = (u64::from(self.ro_registers.PENDING_2.get()) << 32)

diff -uNr 18_backtrace/kernel/src/bsp/device_driver/bcm/bcm2xxx_interrupt_controller.rs 19_kernel_heap/kernel/src/bsp/device_driver/bcm/bcm2xxx_interrupt_controller.rs
--- 18_backtrace/kernel/src/bsp/device_driver/bcm/bcm2xxx_interrupt_controller.rs
+++ 19_kernel_heap/kernel/src/bsp/device_driver/bcm/bcm2xxx_interrupt_controller.rs
@@ -109,6 +109,12 @@
     fn compatible(&self) -> &'static str {
         Self::COMPATIBLE
     }
+
+    unsafe fn init(&self) -> Result<(), &'static str> {
+        self.periph.init();
+
+        Ok(())
+    }
 }

 impl exception::asynchronous::interface::IRQManager for InterruptController {

diff -uNr 18_backtrace/kernel/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs 19_kernel_heap/kernel/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
--- 18_backtrace/kernel/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
+++ 19_kernel_heap/kernel/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
@@ -327,6 +327,13 @@
         self.chars_written += 1;
     }

+    /// Send a slice of characters.
+    fn write_array(&mut self, a: &[char]) {
+        for c in a {
+            self.write_char(*c);
+        }
+    }
+
     /// Block execution until the last buffered character has been physically put on the TX wire.
     fn flush(&self) {
         // Spin until the busy bit is cleared.
@@ -443,6 +450,10 @@
         self.inner.lock(|inner| inner.write_char(c));
     }

+    fn write_array(&self, a: &[char]) {
+        self.inner.lock(|inner| inner.write_array(a));
+    }
+
     fn write_fmt(&self, args: core::fmt::Arguments) -> fmt::Result {
         // Fully qualified syntax for the call to `core::fmt::Write::write_fmt()` to increase
         // readability.

diff -uNr 18_backtrace/kernel/src/bsp/raspberrypi/kernel.ld 19_kernel_heap/kernel/src/bsp/raspberrypi/kernel.ld
--- 18_backtrace/kernel/src/bsp/raspberrypi/kernel.ld
+++ 19_kernel_heap/kernel/src/bsp/raspberrypi/kernel.ld
@@ -35,6 +35,7 @@
 {
     segment_code            PT_LOAD FLAGS(5);
     segment_data            PT_LOAD FLAGS(6);
+    segment_heap            PT_LOAD FLAGS(6);
     segment_boot_core_stack PT_LOAD FLAGS(6);
 }

@@ -84,6 +85,18 @@
     __data_end_exclusive = .;

     /***********************************************************************************************
+    * Heap
+    ***********************************************************************************************/
+    __heap_start = .;
+    .heap (NOLOAD) :
+    {
+        . += 16 * 1024 * 1024;
+    } :segment_heap
+    __heap_end_exclusive = .;
+
+    ASSERT((. & PAGE_MASK) == 0, "Heap is not page aligned")
+
+    /***********************************************************************************************
     * MMIO Remap Reserved
     ***********************************************************************************************/
     __mmio_remap_start = .;

diff -uNr 18_backtrace/kernel/src/bsp/raspberrypi/memory/mmu.rs 19_kernel_heap/kernel/src/bsp/raspberrypi/memory/mmu.rs
--- 18_backtrace/kernel/src/bsp/raspberrypi/memory/mmu.rs
+++ 19_kernel_heap/kernel/src/bsp/raspberrypi/memory/mmu.rs
@@ -122,6 +122,16 @@
     MemoryRegion::new(start_page_addr, end_exclusive_page_addr)
 }

+/// The heap pages.
+pub fn virt_heap_region() -> MemoryRegion<Virtual> {
+    let num_pages = size_to_num_pages(super::heap_size());
+
+    let start_page_addr = super::virt_heap_start();
+    let end_exclusive_page_addr = start_page_addr.checked_offset(num_pages as isize).unwrap();
+
+    MemoryRegion::new(start_page_addr, end_exclusive_page_addr)
+}
+
 /// The boot core stack pages.
 pub fn virt_boot_core_stack_region() -> MemoryRegion<Virtual> {
     let num_pages = size_to_num_pages(super::boot_core_stack_size());
@@ -169,6 +179,14 @@
         &kernel_page_attributes(virt_data_region.start_page_addr()),
     );

+    let virt_heap_region = virt_heap_region();
+    generic_mmu::kernel_add_mapping_record(
+        "Kernel heap",
+        &virt_heap_region,
+        &kernel_virt_to_phys_region(virt_heap_region),
+        &kernel_page_attributes(virt_heap_region.start_page_addr()),
+    );
+
     let virt_boot_core_stack_region = virt_boot_core_stack_region();
     generic_mmu::kernel_add_mapping_record(
         "Kernel boot-core stack",

diff -uNr 18_backtrace/kernel/src/bsp/raspberrypi/memory.rs 19_kernel_heap/kernel/src/bsp/raspberrypi/memory.rs
--- 18_backtrace/kernel/src/bsp/raspberrypi/memory.rs
+++ 19_kernel_heap/kernel/src/bsp/raspberrypi/memory.rs
@@ -28,7 +28,11 @@
 //! | .bss                                  |
 //! |                                       |
 //! +---------------------------------------+
-//! |                                       | data_end_exclusive
+//! |                                       | heap_start == data_end_exclusive
+//! | .heap                                 |
+//! |                                       |
+//! +---------------------------------------+
+//! |                                       | heap_end_exclusive
 //! |                                       |
 //!
 //!
@@ -50,7 +54,11 @@
 //! | .bss                                  |
 //! |                                       |
 //! +---------------------------------------+
-//! |                                       |  mmio_remap_start == data_end_exclusive
+//! |                                       | heap_start == data_end_exclusive
+//! | .heap                                 |
+//! |                                       |
+//! +---------------------------------------+
+//! |                                       |  mmio_remap_start == heap_end_exclusive
 //! | VA region for MMIO remapping          |
 //! |                                       |
 //! +---------------------------------------+
@@ -83,6 +91,9 @@
     static __data_start: UnsafeCell<()>;
     static __data_end_exclusive: UnsafeCell<()>;

+    static __heap_start: UnsafeCell<()>;
+    static __heap_end_exclusive: UnsafeCell<()>;
+
     static __mmio_remap_start: UnsafeCell<()>;
     static __mmio_remap_end_exclusive: UnsafeCell<()>;

@@ -179,6 +190,22 @@
     unsafe { (__data_end_exclusive.get() as usize) - (__data_start.get() as usize) }
 }

+/// Start page address of the heap segment.
+#[inline(always)]
+fn virt_heap_start() -> PageAddress<Virtual> {
+    PageAddress::from(unsafe { __heap_start.get() as usize })
+}
+
+/// Size of the heap segment.
+///
+/// # Safety
+///
+/// - Value is provided by the linker script and must be trusted as-is.
+#[inline(always)]
+fn heap_size() -> usize {
+    unsafe { (__heap_end_exclusive.get() as usize) - (__heap_start.get() as usize) }
+}
+
 /// Start page address of the MMIO remap reservation.
 ///
 /// # Safety

diff -uNr 18_backtrace/kernel/src/console/buffer_console.rs 19_kernel_heap/kernel/src/console/buffer_console.rs
--- 18_backtrace/kernel/src/console/buffer_console.rs
+++ 19_kernel_heap/kernel/src/console/buffer_console.rs
@@ -0,0 +1,108 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2022-2023 Andre Richter <andre.o.richter@gmail.com>
+
+//! A console that buffers input during the init phase.
+
+use super::interface;
+use crate::{console, info, synchronization, synchronization::InitStateLock};
+use core::fmt;
+
+//--------------------------------------------------------------------------------------------------
+// Private Definitions
+//--------------------------------------------------------------------------------------------------
+
+const BUF_SIZE: usize = 1024 * 64;
+
+pub struct BufferConsoleInner {
+    buf: [char; BUF_SIZE],
+    write_ptr: usize,
+}
+
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
+pub struct BufferConsole {
+    inner: InitStateLock<BufferConsoleInner>,
+}
+
+//--------------------------------------------------------------------------------------------------
+// Global instances
+//--------------------------------------------------------------------------------------------------
+
+pub static BUFFER_CONSOLE: BufferConsole = BufferConsole {
+    inner: InitStateLock::new(BufferConsoleInner {
+        // Use the null character, so this lands in .bss and does not waste space in the binary.
+        buf: ['\0'; BUF_SIZE],
+        write_ptr: 0,
+    }),
+};
+
+//--------------------------------------------------------------------------------------------------
+// Private Code
+//--------------------------------------------------------------------------------------------------
+
+impl BufferConsoleInner {
+    fn write_char(&mut self, c: char) {
+        if self.write_ptr < (BUF_SIZE - 1) {
+            self.buf[self.write_ptr] = c;
+            self.write_ptr += 1;
+        }
+    }
+}
+
+impl fmt::Write for BufferConsoleInner {
+    fn write_str(&mut self, s: &str) -> fmt::Result {
+        for c in s.chars() {
+            self.write_char(c);
+        }
+
+        Ok(())
+    }
+}
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+use synchronization::interface::ReadWriteEx;
+
+impl BufferConsole {
+    /// Dump the buffer.
+    ///
+    /// # Invariant
+    ///
+    /// It is expected that this is only called when self != crate::console::console().
+    pub fn dump(&self) {
+        self.inner.read(|inner| {
+            console::console().write_array(&inner.buf[0..inner.write_ptr]);
+
+            if inner.write_ptr == (BUF_SIZE - 1) {
+                info!("Pre-UART buffer overflowed");
+            } else if inner.write_ptr > 0 {
+                info!("End of pre-UART buffer")
+            }
+        });
+    }
+}
+
+impl interface::Write for BufferConsole {
+    fn write_char(&self, c: char) {
+        self.inner.write(|inner| inner.write_char(c));
+    }
+
+    fn write_array(&self, _a: &[char]) {}
+
+    fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result {
+        self.inner.write(|inner| fmt::Write::write_fmt(inner, args))
+    }
+
+    fn flush(&self) {}
+}
+
+impl interface::Read for BufferConsole {
+    fn clear_rx(&self) {}
+}
+
+impl interface::Statistics for BufferConsole {}
+impl interface::All for BufferConsole {}

diff -uNr 18_backtrace/kernel/src/console/null_console.rs 19_kernel_heap/kernel/src/console/null_console.rs
--- 18_backtrace/kernel/src/console/null_console.rs
+++ 19_kernel_heap/kernel/src/console/null_console.rs
@@ -1,41 +0,0 @@
-// SPDX-License-Identifier: MIT OR Apache-2.0
-//
-// Copyright (c) 2022-2023 Andre Richter <andre.o.richter@gmail.com>
-
-//! Null console.
-
-use super::interface;
-use core::fmt;
-
-//--------------------------------------------------------------------------------------------------
-// Public Definitions
-//--------------------------------------------------------------------------------------------------
-
-pub struct NullConsole;
-
-//--------------------------------------------------------------------------------------------------
-// Global instances
-//--------------------------------------------------------------------------------------------------
-
-pub static NULL_CONSOLE: NullConsole = NullConsole {};
-
-//--------------------------------------------------------------------------------------------------
-// Public Code
-//--------------------------------------------------------------------------------------------------
-
-impl interface::Write for NullConsole {
-    fn write_char(&self, _c: char) {}
-
-    fn write_fmt(&self, _args: fmt::Arguments) -> fmt::Result {
-        fmt::Result::Ok(())
-    }
-
-    fn flush(&self) {}
-}
-
-impl interface::Read for NullConsole {
-    fn clear_rx(&self) {}
-}
-
-impl interface::Statistics for NullConsole {}
-impl interface::All for NullConsole {}

diff -uNr 18_backtrace/kernel/src/console.rs 19_kernel_heap/kernel/src/console.rs
--- 18_backtrace/kernel/src/console.rs
+++ 19_kernel_heap/kernel/src/console.rs
@@ -4,7 +4,7 @@

 //! System console.

-mod null_console;
+mod buffer_console;

 use crate::synchronization;

@@ -21,6 +21,9 @@
         /// Write a single character.
         fn write_char(&self, c: char);

+        /// Write a slice of characters.
+        fn write_array(&self, a: &[char]);
+
         /// Write a Rust format string.
         fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result;

@@ -61,7 +64,7 @@
 //--------------------------------------------------------------------------------------------------

 static CUR_CONSOLE: InitStateLock<&'static (dyn interface::All + Sync)> =
-    InitStateLock::new(&null_console::NULL_CONSOLE);
+    InitStateLock::new(&buffer_console::BUFFER_CONSOLE);

 //--------------------------------------------------------------------------------------------------
 // Public Code
@@ -71,6 +74,15 @@
 /// Register a new console.
 pub fn register_console(new_console: &'static (dyn interface::All + Sync)) {
     CUR_CONSOLE.write(|con| *con = new_console);
+
+    static FIRST_SWITCH: InitStateLock<bool> = InitStateLock::new(true);
+    FIRST_SWITCH.write(|first| {
+        if *first {
+            *first = false;
+
+            buffer_console::BUFFER_CONSOLE.dump();
+        }
+    });
 }

 /// Return a reference to the currently registered console.

diff -uNr 18_backtrace/kernel/src/driver.rs 19_kernel_heap/kernel/src/driver.rs
--- 18_backtrace/kernel/src/driver.rs
+++ 19_kernel_heap/kernel/src/driver.rs
@@ -8,23 +8,10 @@
     exception, info,
     synchronization::{interface::ReadWriteEx, InitStateLock},
 };
+use alloc::vec::Vec;
 use core::fmt;

 //--------------------------------------------------------------------------------------------------
-// Private Definitions
-//--------------------------------------------------------------------------------------------------
-
-const NUM_DRIVERS: usize = 5;
-
-struct DriverManagerInner<T>
-where
-    T: 'static,
-{
-    next_index: usize,
-    descriptors: [Option<DeviceDriverDescriptor<T>>; NUM_DRIVERS],
-}
-
-//--------------------------------------------------------------------------------------------------
 // Public Definitions
 //--------------------------------------------------------------------------------------------------

@@ -68,7 +55,6 @@
 pub type DeviceDriverPostInitCallback = unsafe fn() -> Result<(), &'static str>;

 /// A descriptor for device drivers.
-#[derive(Copy, Clone)]
 pub struct DeviceDriverDescriptor<T>
 where
     T: 'static,
@@ -83,7 +69,7 @@
 where
     T: 'static,
 {
-    inner: InitStateLock<DriverManagerInner<T>>,
+    descriptors: InitStateLock<Vec<DeviceDriverDescriptor<T>>>,
 }

 //--------------------------------------------------------------------------------------------------
@@ -93,23 +79,6 @@
 static DRIVER_MANAGER: DriverManager<exception::asynchronous::IRQNumber> = DriverManager::new();

 //--------------------------------------------------------------------------------------------------
-// Private Code
-//--------------------------------------------------------------------------------------------------
-
-impl<T> DriverManagerInner<T>
-where
-    T: 'static + Copy,
-{
-    /// Create an instance.
-    pub const fn new() -> Self {
-        Self {
-            next_index: 0,
-            descriptors: [None; NUM_DRIVERS],
-        }
-    }
-}
-
-//--------------------------------------------------------------------------------------------------
 // Public Code
 //--------------------------------------------------------------------------------------------------

@@ -135,32 +104,19 @@

 impl<T> DriverManager<T>
 where
-    T: fmt::Display + Copy,
+    T: fmt::Display,
 {
     /// Create an instance.
     pub const fn new() -> Self {
         Self {
-            inner: InitStateLock::new(DriverManagerInner::new()),
+            descriptors: InitStateLock::new(Vec::new()),
         }
     }

     /// Register a device driver with the kernel.
     pub fn register_driver(&self, descriptor: DeviceDriverDescriptor<T>) {
-        self.inner.write(|inner| {
-            inner.descriptors[inner.next_index] = Some(descriptor);
-            inner.next_index += 1;
-        })
-    }
-
-    /// Helper for iterating over registered drivers.
-    fn for_each_descriptor<'a>(&'a self, f: impl FnMut(&'a DeviceDriverDescriptor<T>)) {
-        self.inner.read(|inner| {
-            inner
-                .descriptors
-                .iter()
-                .filter_map(|x| x.as_ref())
-                .for_each(f)
-        })
+        self.descriptors
+            .write(|descriptors| descriptors.push(descriptor));
     }

     /// Fully initialize all drivers and their interrupts handlers.
@@ -169,53 +125,54 @@
     ///
     /// - During init, drivers might do stuff with system-wide impact.
     pub unsafe fn init_drivers_and_irqs(&self) {
-        self.for_each_descriptor(|descriptor| {
-            // 1. Initialize driver.
-            if let Err(x) = descriptor.device_driver.init() {
-                panic!(
-                    "Error initializing driver: {}: {}",
-                    descriptor.device_driver.compatible(),
-                    x
-                );
-            }
-
-            // 2. Call corresponding post init callback.
-            if let Some(callback) = &descriptor.post_init_callback {
-                if let Err(x) = callback() {
+        self.descriptors.read(|descriptors| {
+            for descriptor in descriptors {
+                // 1. Initialize driver.
+                if let Err(x) = descriptor.device_driver.init() {
                     panic!(
-                        "Error during driver post-init callback: {}: {}",
+                        "Error initializing driver: {}: {}",
                         descriptor.device_driver.compatible(),
                         x
                     );
                 }
+
+                // 2. Call corresponding post init callback.
+                if let Some(callback) = &descriptor.post_init_callback {
+                    if let Err(x) = callback() {
+                        panic!(
+                            "Error during driver post-init callback: {}: {}",
+                            descriptor.device_driver.compatible(),
+                            x
+                        );
+                    }
+                }
             }
-        });

-        // 3. After all post-init callbacks were done, the interrupt controller should be
-        //    registered and functional. So let drivers register with it now.
-        self.for_each_descriptor(|descriptor| {
-            if let Some(irq_number) = &descriptor.irq_number {
-                if let Err(x) = descriptor
-                    .device_driver
-                    .register_and_enable_irq_handler(irq_number)
-                {
-                    panic!(
-                        "Error during driver interrupt handler registration: {}: {}",
-                        descriptor.device_driver.compatible(),
-                        x
-                    );
+            // 3. After all post-init callbacks were done, the interrupt controller should be
+            //    registered and functional. So let drivers register with it now.
+            for descriptor in descriptors {
+                if let Some(irq_number) = &descriptor.irq_number {
+                    if let Err(x) = descriptor
+                        .device_driver
+                        .register_and_enable_irq_handler(irq_number)
+                    {
+                        panic!(
+                            "Error during driver interrupt handler registration: {}: {}",
+                            descriptor.device_driver.compatible(),
+                            x
+                        );
+                    }
                 }
             }
-        });
+        })
     }

     /// Enumerate all registered device drivers.
     pub fn enumerate(&self) {
-        let mut i: usize = 1;
-        self.for_each_descriptor(|descriptor| {
-            info!("      {}. {}", i, descriptor.device_driver.compatible());
-
-            i += 1;
+        self.descriptors.read(|descriptors| {
+            for (i, desc) in descriptors.iter().enumerate() {
+                info!("      {}. {}", i + 1, desc.device_driver.compatible());
+            }
         });
     }
 }

diff -uNr 18_backtrace/kernel/src/lib.rs 19_kernel_heap/kernel/src/lib.rs
--- 18_backtrace/kernel/src/lib.rs
+++ 19_kernel_heap/kernel/src/lib.rs
@@ -110,6 +110,7 @@

 #![allow(clippy::upper_case_acronyms)]
 #![allow(incomplete_features)]
+#![feature(alloc_error_handler)]
 #![feature(asm_const)]
 #![feature(const_option)]
 #![feature(core_intrinsics)]
@@ -130,6 +131,8 @@
 #![reexport_test_harness_main = "test_main"]
 #![test_runner(crate::test_runner)]

+extern crate alloc;
+
 mod panic_wait;
 mod synchronization;


diff -uNr 18_backtrace/kernel/src/main.rs 19_kernel_heap/kernel/src/main.rs
--- 18_backtrace/kernel/src/main.rs
+++ 19_kernel_heap/kernel/src/main.rs
@@ -13,6 +13,8 @@
 #![no_main]
 #![no_std]

+extern crate alloc;
+
 use libkernel::{bsp, cpu, driver, exception, info, memory, state, time};

 /// Early init code.
@@ -73,6 +75,9 @@
     info!("Registered IRQ handlers:");
     exception::asynchronous::irq_manager().print_handler();

+    info!("Kernel heap:");
+    memory::heap_alloc::kernel_heap_allocator().print_usage();
+
     info!("Echoing input now");
     cpu::wait_forever();
 }

diff -uNr 18_backtrace/kernel/src/memory/heap_alloc.rs 19_kernel_heap/kernel/src/memory/heap_alloc.rs
--- 18_backtrace/kernel/src/memory/heap_alloc.rs
+++ 19_kernel_heap/kernel/src/memory/heap_alloc.rs
@@ -0,0 +1,147 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2022-2023 Andre Richter <andre.o.richter@gmail.com>
+
+//! Heap allocation.
+
+use crate::{
+    backtrace, bsp, common, debug, info,
+    memory::{Address, Virtual},
+    synchronization,
+    synchronization::IRQSafeNullLock,
+    warn,
+};
+use alloc::alloc::{GlobalAlloc, Layout};
+use core::sync::atomic::{AtomicBool, Ordering};
+use linked_list_allocator::Heap as LinkedListHeap;
+
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
+/// A heap allocator that can be lazyily initialized.
+pub struct HeapAllocator {
+    inner: IRQSafeNullLock<LinkedListHeap>,
+}
+
+//--------------------------------------------------------------------------------------------------
+// Global instances
+//--------------------------------------------------------------------------------------------------
+
+#[global_allocator]
+static KERNEL_HEAP_ALLOCATOR: HeapAllocator = HeapAllocator::new();
+
+//--------------------------------------------------------------------------------------------------
+// Private Code
+//--------------------------------------------------------------------------------------------------
+
+#[inline(always)]
+fn debug_print_alloc_dealloc(operation: &'static str, ptr: *mut u8, layout: Layout) {
+    let size = layout.size();
+    let (size_h, size_unit) = common::size_human_readable_ceil(size);
+    let addr = Address::<Virtual>::new(ptr as usize);
+
+    debug!(
+        "Kernel Heap: {}\n      \
+        Size:     {:#x} ({} {})\n      \
+        Start:    {}\n      \
+        End excl: {}\n\n      \
+        {}",
+        operation,
+        size,
+        size_h,
+        size_unit,
+        addr,
+        addr + size,
+        backtrace::Backtrace
+    );
+}
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+use synchronization::interface::Mutex;
+
+#[alloc_error_handler]
+fn alloc_error_handler(layout: Layout) -> ! {
+    panic!("Allocation error: {:?}", layout)
+}
+
+/// Return a reference to the kernel's heap allocator.
+pub fn kernel_heap_allocator() -> &'static HeapAllocator {
+    &KERNEL_HEAP_ALLOCATOR
+}
+
+impl HeapAllocator {
+    /// Create an instance.
+    pub const fn new() -> Self {
+        Self {
+            inner: IRQSafeNullLock::new(LinkedListHeap::empty()),
+        }
+    }
+
+    /// Print the current heap usage.
+    pub fn print_usage(&self) {
+        let (used, free) = KERNEL_HEAP_ALLOCATOR
+            .inner
+            .lock(|inner| (inner.used(), inner.free()));
+
+        if used >= 1024 {
+            let (used_h, used_unit) = common::size_human_readable_ceil(used);
+            info!("      Used: {} Byte ({} {})", used, used_h, used_unit);
+        } else {
+            info!("      Used: {} Byte", used);
+        }
+
+        if free >= 1024 {
+            let (free_h, free_unit) = common::size_human_readable_ceil(free);
+            info!("      Free: {} Byte ({} {})", free, free_h, free_unit);
+        } else {
+            info!("      Free: {} Byte", free);
+        }
+    }
+}
+
+unsafe impl GlobalAlloc for HeapAllocator {
+    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
+        let result = KERNEL_HEAP_ALLOCATOR
+            .inner
+            .lock(|inner| inner.allocate_first_fit(layout).ok());
+
+        match result {
+            None => core::ptr::null_mut(),
+            Some(allocation) => {
+                let ptr = allocation.as_ptr();
+
+                debug_print_alloc_dealloc("Allocation", ptr, layout);
+
+                ptr
+            }
+        }
+    }
+
+    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
+        KERNEL_HEAP_ALLOCATOR
+            .inner
+            .lock(|inner| inner.deallocate(core::ptr::NonNull::new_unchecked(ptr), layout));
+
+        debug_print_alloc_dealloc("Free", ptr, layout);
+    }
+}
+
+/// Query the BSP for the heap region and initialize the kernel's heap allocator with it.
+pub fn kernel_init_heap_allocator() {
+    static INIT_DONE: AtomicBool = AtomicBool::new(false);
+    if INIT_DONE.load(Ordering::Relaxed) {
+        warn!("Already initialized");
+        return;
+    }
+
+    let region = bsp::memory::mmu::virt_heap_region();
+
+    KERNEL_HEAP_ALLOCATOR.inner.lock(|inner| unsafe {
+        inner.init(region.start_addr().as_usize() as *mut u8, region.size())
+    });
+
+    INIT_DONE.store(true, Ordering::Relaxed);
+}

diff -uNr 18_backtrace/kernel/src/memory/mmu/mapping_record.rs 19_kernel_heap/kernel/src/memory/mmu/mapping_record.rs
--- 18_backtrace/kernel/src/memory/mmu/mapping_record.rs
+++ 19_kernel_heap/kernel/src/memory/mmu/mapping_record.rs
@@ -8,7 +8,8 @@
     AccessPermissions, Address, AttributeFields, MMIODescriptor, MemAttributes, MemoryRegion,
     Physical, Virtual,
 };
-use crate::{bsp, common, info, synchronization, synchronization::InitStateLock, warn};
+use crate::{bsp, common, info, synchronization, synchronization::InitStateLock};
+use alloc::{vec, vec::Vec};

 //--------------------------------------------------------------------------------------------------
 // Private Definitions
@@ -16,9 +17,8 @@

 /// Type describing a virtual memory mapping.
 #[allow(missing_docs)]
-#[derive(Copy, Clone)]
 struct MappingRecordEntry {
-    pub users: [Option<&'static str>; 5],
+    pub users: Vec<&'static str>,
     pub phys_start_addr: Address<Physical>,
     pub virt_start_addr: Address<Virtual>,
     pub num_pages: usize,
@@ -26,7 +26,7 @@
 }

 struct MappingRecord {
-    inner: [Option<MappingRecordEntry>; 12],
+    inner: Vec<MappingRecordEntry>,
 }

 //--------------------------------------------------------------------------------------------------
@@ -48,7 +48,7 @@
         attr: &AttributeFields,
     ) -> Self {
         Self {
-            users: [Some(name), None, None, None, None],
+            users: vec![name],
             phys_start_addr: phys_region.start_addr(),
             virt_start_addr: virt_region.start_addr(),
             num_pages: phys_region.num_pages(),
@@ -56,54 +56,28 @@
         }
     }

-    fn find_next_free_user(&mut self) -> Result<&mut Option<&'static str>, &'static str> {
-        if let Some(x) = self.users.iter_mut().find(|x| x.is_none()) {
-            return Ok(x);
-        };
-
-        Err("Storage for user info exhausted")
-    }
-
-    pub fn add_user(&mut self, user: &'static str) -> Result<(), &'static str> {
-        let x = self.find_next_free_user()?;
-        *x = Some(user);
-        Ok(())
+    pub fn add_user(&mut self, user: &'static str) {
+        self.users.push(user);
     }
 }

 impl MappingRecord {
     pub const fn new() -> Self {
-        Self { inner: [None; 12] }
-    }
-
-    fn size(&self) -> usize {
-        self.inner.iter().filter(|x| x.is_some()).count()
+        Self { inner: Vec::new() }
     }

     fn sort(&mut self) {
-        let upper_bound_exclusive = self.size();
-        let entries = &mut self.inner[0..upper_bound_exclusive];
-
-        if !entries.is_sorted_by_key(|item| item.unwrap().virt_start_addr) {
-            entries.sort_unstable_by_key(|item| item.unwrap().virt_start_addr)
+        if !self.inner.is_sorted_by_key(|item| item.virt_start_addr) {
+            self.inner.sort_unstable_by_key(|item| item.virt_start_addr)
         }
     }

-    fn find_next_free(&mut self) -> Result<&mut Option<MappingRecordEntry>, &'static str> {
-        if let Some(x) = self.inner.iter_mut().find(|x| x.is_none()) {
-            return Ok(x);
-        }
-
-        Err("Storage for mapping info exhausted")
-    }
-
     fn find_duplicate(
         &mut self,
         phys_region: &MemoryRegion<Physical>,
     ) -> Option<&mut MappingRecordEntry> {
         self.inner
             .iter_mut()
-            .filter_map(|x| x.as_mut())
             .filter(|x| x.attribute_fields.mem_attributes == MemAttributes::Device)
             .find(|x| {
                 if x.phys_start_addr != phys_region.start_addr() {
@@ -124,10 +98,8 @@
         virt_region: &MemoryRegion<Virtual>,
         phys_region: &MemoryRegion<Physical>,
         attr: &AttributeFields,
-    ) -> Result<(), &'static str> {
-        let x = self.find_next_free()?;
-
-        *x = Some(MappingRecordEntry::new(
+    ) {
+        self.inner.push(MappingRecordEntry::new(
             name,
             virt_region,
             phys_region,
@@ -135,8 +107,6 @@
         ));

         self.sort();
-
-        Ok(())
     }

     pub fn print(&self) {
@@ -147,7 +117,7 @@
         );
         info!("      -------------------------------------------------------------------------------------------------------------------------------------------");

-        for i in self.inner.iter().flatten() {
+        for i in self.inner.iter() {
             let size = i.num_pages * bsp::memory::mmu::KernelGranule::SIZE;
             let virt_start = i.virt_start_addr;
             let virt_end_inclusive = virt_start + (size - 1);
@@ -183,16 +153,14 @@
                 attr,
                 acc_p,
                 xn,
-                i.users[0].unwrap()
+                i.users[0]
             );

-            for k in i.users[1..].iter() {
-                if let Some(additional_user) = *k {
-                    info!(
+            for k in &i.users[1..] {
+                info!(
                         "                                                                                                            | {}",
-                        additional_user
+                        k
                     );
-                }
             }
         }

@@ -211,7 +179,7 @@
     virt_region: &MemoryRegion<Virtual>,
     phys_region: &MemoryRegion<Physical>,
     attr: &AttributeFields,
-) -> Result<(), &'static str> {
+) {
     KERNEL_MAPPING_RECORD.write(|mr| mr.add(name, virt_region, phys_region, attr))
 }

@@ -224,9 +192,7 @@
     KERNEL_MAPPING_RECORD.write(|mr| {
         let dup = mr.find_duplicate(&phys_region)?;

-        if let Err(x) = dup.add_user(new_user) {
-            warn!("{}", x);
-        }
+        dup.add_user(new_user);

         Some(dup.virt_start_addr)
     })

diff -uNr 18_backtrace/kernel/src/memory/mmu.rs 19_kernel_heap/kernel/src/memory/mmu.rs
--- 18_backtrace/kernel/src/memory/mmu.rs
+++ 19_kernel_heap/kernel/src/memory/mmu.rs
@@ -17,7 +17,6 @@
     bsp,
     memory::{Address, Physical, Virtual},
     synchronization::{self, interface::Mutex},
-    warn,
 };
 use core::{fmt, num::NonZeroUsize};

@@ -176,9 +175,7 @@
     phys_region: &MemoryRegion<Physical>,
     attr: &AttributeFields,
 ) {
-    if let Err(x) = mapping_record::kernel_add(name, virt_region, phys_region, attr) {
-        warn!("{}", x);
-    }
+    mapping_record::kernel_add(name, virt_region, phys_region, attr);
 }

 /// MMIO remapping in the kernel translation tables.

diff -uNr 18_backtrace/kernel/src/memory.rs 19_kernel_heap/kernel/src/memory.rs
--- 18_backtrace/kernel/src/memory.rs
+++ 19_kernel_heap/kernel/src/memory.rs
@@ -4,6 +4,7 @@

 //! Memory Management.

+pub mod heap_alloc;
 pub mod mmu;

 use crate::{bsp, common};
@@ -163,6 +164,7 @@
 /// Initialize the memory subsystem.
 pub fn init() {
     mmu::kernel_init_mmio_va_allocator();
+    heap_alloc::kernel_init_heap_allocator();
 }

 //--------------------------------------------------------------------------------------------------

diff -uNr 18_backtrace/kernel/src/print.rs 19_kernel_heap/kernel/src/print.rs
--- 18_backtrace/kernel/src/print.rs
+++ 19_kernel_heap/kernel/src/print.rs
@@ -82,3 +82,31 @@
         ));
     })
 }
+
+/// Debug print, with a newline.
+#[macro_export]
+macro_rules! debug {
+    ($string:expr) => ({
+        if cfg!(feature = "debug_prints") {
+            let timestamp = $crate::time::time_manager().uptime();
+
+            $crate::print::_print(format_args_nl!(
+                concat!("<[>D {:>3}.{:06}> ", $string),
+                timestamp.as_secs(),
+                timestamp.subsec_micros(),
+            ));
+        }
+    });
+    ($format_string:expr, $($arg:tt)*) => ({
+        if cfg!(feature = "debug_prints") {
+            let timestamp = $crate::time::time_manager().uptime();
+
+            $crate::print::_print(format_args_nl!(
+                concat!("<D {:>3}.{:06}> ", $format_string),
+                timestamp.as_secs(),
+                timestamp.subsec_micros(),
+                $($arg)*
+            ));
+        }
+    })
+}

diff -uNr 18_backtrace/kernel/src/state.rs 19_kernel_heap/kernel/src/state.rs
--- 18_backtrace/kernel/src/state.rs
+++ 19_kernel_heap/kernel/src/state.rs
@@ -52,7 +52,7 @@
     const SINGLE_CORE_MAIN: u8 = 1;
     const MULTI_CORE_MAIN: u8 = 2;

-    /// Create an instance.
+    /// Create a new instance.
     pub const fn new() -> Self {
         Self(AtomicU8::new(Self::INIT))
     }

diff -uNr 18_backtrace/Makefile 19_kernel_heap/Makefile
--- 18_backtrace/Makefile
+++ 19_kernel_heap/Makefile
@@ -16,6 +16,11 @@
 # Default to a serial device name that is common in Linux.
 DEV_SERIAL ?= /dev/ttyUSB0

+# Optional debug prints.
+ifdef DEBUG_PRINTS
+    FEATURES = --features debug_prints
+endif
+
 # Optional integration test name.
 ifdef TEST
     TEST_ARG = --test $(TEST)
@@ -70,7 +75,7 @@
 ##--------------------------------------------------------------------------------------------------
 KERNEL_MANIFEST      = kernel/Cargo.toml
 KERNEL_LINKER_SCRIPT = kernel.ld
-LAST_BUILD_CONFIG    = target/$(BSP).build_config
+LAST_BUILD_CONFIG    = target/$(BSP)_$(DEBUG_PRINTS).build_config

 KERNEL_ELF_RAW      = target/$(TARGET)/release/kernel
 # This parses cargo's dep-info file.
@@ -117,17 +122,17 @@
     -D warnings                   \
     -D missing_docs

-FEATURES      = --features bsp_$(BSP)
+FEATURES     += --features bsp_$(BSP)
 COMPILER_ARGS = --target=$(TARGET) \
     $(FEATURES)                    \
     --release

 # build-std can be skipped for helper commands that do not rely on correct stack frames and other
 # custom compiler options. This results in a huge speedup.
-RUSTC_CMD   = cargo rustc $(COMPILER_ARGS) -Z build-std=core --manifest-path $(KERNEL_MANIFEST)
+RUSTC_CMD   = cargo rustc $(COMPILER_ARGS) -Z build-std=core,alloc --manifest-path $(KERNEL_MANIFEST)
 DOC_CMD     = cargo doc $(COMPILER_ARGS)
 CLIPPY_CMD  = cargo clippy $(COMPILER_ARGS)
-TEST_CMD    = cargo test $(COMPILER_ARGS) -Z build-std=core --manifest-path $(KERNEL_MANIFEST)
+TEST_CMD    = cargo test $(COMPILER_ARGS) -Z build-std=core,alloc --manifest-path $(KERNEL_MANIFEST)
 OBJCOPY_CMD = rust-objcopy \
     --strip-all            \
     -O binary

```
