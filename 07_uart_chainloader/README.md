# Tutorial 07 - UART Chainloader

## tl;dr

Running from an SD card was a nice experience, but it would be extremely tedious to do it for every
new binary. Let's write a [chainloader] using [position independent code]. This will be the last
binary you need to put on the SD card. Each following tutorial will provide a `chainboot` target in
the `Makefile` that lets you conveniently load the kernel over `UART`.

[chainloader]: https://en.wikipedia.org/wiki/Chain_loading
[position independent code]: https://en.wikipedia.org/wiki/Position-independent_code

## Install and test it

Our chainloader is called `MiniLoad` and is inspired by [raspbootin].

You can try it with this tutorial already:
1. Depending on your target hardware:`make` or `BSP=rpi4 make`.
2. Copy `kernel8.img` to the SD card.
3. Execute `make chainboot` or `BSP=rpi4 make chainboot`.
4. Now plug in the USB Serial.
5. Observe the loader fetching a kernel over `UART`:

> ❗ **NOTE**: By default, `make chainboot` tries to connect to `/dev/ttyUSB0`.
> Should the USB serial on your system have a different name, you have to provide it explicitly. For
> example:
>
> `DEV_SERIAL=/dev/tty.usbserial-0001 make chainboot`

[raspbootin]: https://github.com/mrvn/raspbootin

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
[MP] ⏩ Pushing 7 KiB ==========================================🦀 100% 0 KiB/s Time: 00:00:00
[ML] Loaded! Executing the payload now

[0] Booting on: Raspberry Pi 3
[1] Drivers loaded:
      1. BCM GPIO
      2. BCM PL011 UART
[2] Chars written: 93
[3] Echoing input now
```

In this tutorial, a version of the kernel from the previous tutorial is loaded
for demo purposes. In subsequent tuts, it will be the working directory's
kernel.

## Test it

The `Makefile` in this tutorial has an additional target, `qemuasm`, that lets
you nicely observe the jump from the loaded address (`0x80_XXX`) to the
relocated code at (`0x0200_0XXX`):

```console
$ make qemuasm
[...]
IN:
0x0008098c:  b0000008  adrp     x8, #0x81000
0x00080990:  b0000000  adrp     x0, #0x81000
0x00080994:  912a8000  add      x0, x0, #0xaa0
0x00080998:  f9471908  ldr      x8, [x8, #0xe30]
0x0008099c:  d63f0100  blr      x8

----------------
IN:
0x02000b1c:  b0000008  adrp     x8, #0x2001000
0x02000b20:  b0000009  adrp     x9, #0x2001000
0x02000b24:  f9475d08  ldr      x8, [x8, #0xeb8]
0x02000b28:  f9476129  ldr      x9, [x9, #0xec0]
0x02000b2c:  eb08013f  cmp      x9, x8
0x02000b30:  540000c2  b.hs     #0x2000b48
[...]
```

## Diff to previous
```diff
Binary files 06_drivers_gpio_uart/demo_payload_rpi3.img and 07_uart_chainloader/demo_payload_rpi3.img differ
Binary files 06_drivers_gpio_uart/demo_payload_rpi4.img and 07_uart_chainloader/demo_payload_rpi4.img differ

diff -uNr 06_drivers_gpio_uart/Makefile 07_uart_chainloader/Makefile
--- 06_drivers_gpio_uart/Makefile
+++ 07_uart_chainloader/Makefile
@@ -5,6 +5,12 @@
 # Default to the RPi3
 BSP ?= rpi3

+# Default to a serial device name that is common in Linux.
+DEV_SERIAL ?= /dev/ttyUSB0
+
+# Query the host system's kernel name
+UNAME_S = $(shell uname -s)
+
 # BSP-specific arguments
 ifeq ($(BSP),rpi3)
     TARGET            = aarch64-unknown-none-softfloat
@@ -13,7 +19,8 @@
     QEMU_MACHINE_TYPE = raspi3
     QEMU_RELEASE_ARGS = -serial stdio -display none
     LINKER_FILE       = src/bsp/raspberrypi/link.ld
-    RUSTC_MISC_ARGS   = -C target-cpu=cortex-a53
+    RUSTC_MISC_ARGS   = -C target-cpu=cortex-a53 -C relocation-model=pic
+    CHAINBOOT_DEMO_PAYLOAD = demo_payload_rpi3.img
 else ifeq ($(BSP),rpi4)
     TARGET            = aarch64-unknown-none-softfloat
     KERNEL_BIN        = kernel8.img
@@ -21,7 +28,8 @@
     QEMU_MACHINE_TYPE =
     QEMU_RELEASE_ARGS = -serial stdio -display none
     LINKER_FILE       = src/bsp/raspberrypi/link.ld
-    RUSTC_MISC_ARGS   = -C target-cpu=cortex-a72
+    RUSTC_MISC_ARGS   = -C target-cpu=cortex-a72 -C relocation-model=pic
+    CHAINBOOT_DEMO_PAYLOAD = demo_payload_rpi4.img
 endif

 # Export for build.rs
@@ -46,12 +54,23 @@

 DOCKER_IMAGE         = rustembedded/osdev-utils
 DOCKER_CMD           = docker run -it --rm -v $(shell pwd):/work/tutorial -w /work/tutorial
+DOCKER_ARG_DIR_UTILS = -v $(shell pwd)/../utils:/work/utils
+DOCKER_ARG_DEV       = --privileged -v /dev:/dev

 DOCKER_QEMU = $(DOCKER_CMD) $(DOCKER_IMAGE)

-EXEC_QEMU = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)
+# Dockerize commands that require USB device passthrough only on Linux
+ifeq ($(UNAME_S),Linux)
+    DOCKER_CMD_DEV = $(DOCKER_CMD) $(DOCKER_ARG_DEV)
+
+    DOCKER_CHAINBOOT = $(DOCKER_CMD_DEV) $(DOCKER_ARG_DIR_UTILS) $(DOCKER_IMAGE)
+endif

-.PHONY: all $(KERNEL_ELF) $(KERNEL_BIN) doc qemu clippy clean readelf objdump nm check
+EXEC_QEMU     = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)
+EXEC_MINIPUSH = ruby ../utils/minipush.rb
+
+.PHONY: all $(KERNEL_ELF) $(KERNEL_BIN) doc qemu qemuasm chainboot clippy clean readelf objdump nm \
+    check

 all: $(KERNEL_BIN)

@@ -65,13 +84,19 @@
 	$(DOC_CMD) --document-private-items --open

 ifeq ($(QEMU_MACHINE_TYPE),)
-qemu:
+qemu qemuasm:
 	@echo "This board is not yet supported for QEMU."
 else
 qemu: $(KERNEL_BIN)
 	@$(DOCKER_QEMU) $(EXEC_QEMU) $(QEMU_RELEASE_ARGS) -kernel $(KERNEL_BIN)
+
+qemuasm: $(KERNEL_BIN)
+	@$(DOCKER_QEMU) $(EXEC_QEMU) $(QEMU_RELEASE_ARGS) -kernel $(KERNEL_BIN) -d in_asm
 endif

+chainboot:
+	@$(DOCKER_CHAINBOOT) $(EXEC_MINIPUSH) $(DEV_SERIAL) $(CHAINBOOT_DEMO_PAYLOAD)
+
 clippy:
 	RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(CLIPPY_CMD)


diff -uNr 06_drivers_gpio_uart/src/_arch/aarch64/cpu.rs 07_uart_chainloader/src/_arch/aarch64/cpu.rs
--- 06_drivers_gpio_uart/src/_arch/aarch64/cpu.rs
+++ 07_uart_chainloader/src/_arch/aarch64/cpu.rs
@@ -21,12 +21,12 @@
 #[naked]
 #[no_mangle]
 pub unsafe extern "C" fn _start() -> ! {
-    use crate::runtime_init;
+    use crate::relocate;

     // Expect the boot core to start in EL2.
     if bsp::cpu::BOOT_CORE_ID == cpu::smp::core_id() {
         SP.set(bsp::cpu::BOOT_CORE_STACK_START);
-        runtime_init::runtime_init()
+        relocate::relocate_self::<u64>()
     } else {
         // If not core0, infinitely wait for events.
         wait_forever()

diff -uNr 06_drivers_gpio_uart/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs 07_uart_chainloader/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
--- 06_drivers_gpio_uart/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
+++ 07_uart_chainloader/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
@@ -267,6 +267,16 @@
         let mut r = &self.inner;
         r.lock(|inner| fmt::Write::write_fmt(inner, args))
     }
+
+    fn flush(&self) {
+        // Spin until TX FIFO empty is set.
+        let mut r = &self.inner;
+        r.lock(|inner| {
+            while !inner.registers.FR.matches_all(FR::TXFE::SET) {
+                cpu::nop();
+            }
+        });
+    }
 }

 impl console::interface::Read for PL011Uart {
@@ -278,18 +288,21 @@
                 cpu::nop();
             }

-            // Read one character.
-            let mut ret = inner.registers.DR.get() as u8 as char;
-
-            // Convert carrige return to newline.
-            if ret == '\r' {
-                ret = '\n'
-            }
-
             // Update statistics.
             inner.chars_read += 1;

-            ret
+            // Read one character.
+            inner.registers.DR.get() as u8 as char
+        })
+    }
+
+    fn clear(&self) {
+        let mut r = &self.inner;
+        r.lock(|inner| {
+            // Read from the RX FIFO until it is indicating empty.
+            while !inner.registers.FR.matches_all(FR::RXFE::SET) {
+                inner.registers.DR.get();
+            }
         })
     }
 }

diff -uNr 06_drivers_gpio_uart/src/bsp/raspberrypi/cpu.rs 07_uart_chainloader/src/bsp/raspberrypi/cpu.rs
--- 06_drivers_gpio_uart/src/bsp/raspberrypi/cpu.rs
+++ 07_uart_chainloader/src/bsp/raspberrypi/cpu.rs
@@ -13,3 +13,6 @@

 /// The early boot core's stack address.
 pub const BOOT_CORE_STACK_START: u64 = 0x80_000;
+
+/// The address on which the Raspberry firmware loads every binary by default.
+pub const BOARD_DEFAULT_LOAD_ADDRESS: usize = 0x80_000;

diff -uNr 06_drivers_gpio_uart/src/bsp/raspberrypi/link.ld 07_uart_chainloader/src/bsp/raspberrypi/link.ld
--- 06_drivers_gpio_uart/src/bsp/raspberrypi/link.ld
+++ 07_uart_chainloader/src/bsp/raspberrypi/link.ld
@@ -5,9 +5,10 @@

 SECTIONS
 {
-    /* Set current address to the value from which the RPi starts execution */
-    . = 0x80000;
+    /* Set the link address to 32 MiB */
+    . = 0x2000000;

+    __binary_start = .;
     .text :
     {
         *(.text._start) *(.text*)
@@ -32,5 +33,14 @@
         __bss_end = .;
     }

+    .got :
+    {
+        *(.got*)
+    }
+
+    /* Fill up to 8 byte, b/c relocating the binary is done in u64 chunks */
+    . = ALIGN(8);
+    __binary_end = .;
+
     /DISCARD/ : { *(.comment*) }
 }

diff -uNr 06_drivers_gpio_uart/src/console.rs 07_uart_chainloader/src/console.rs
--- 06_drivers_gpio_uart/src/console.rs
+++ 07_uart_chainloader/src/console.rs
@@ -19,6 +19,10 @@

         /// Write a Rust format string.
         fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result;
+
+        /// Block execution until the last character has been physically put on the TX wire
+        /// (draining TX buffers/FIFOs, if any).
+        fn flush(&self);
     }

     /// Console read functions.
@@ -27,6 +31,9 @@
         fn read_char(&self) -> char {
             ' '
         }
+
+        /// Clear RX buffers, if any.
+        fn clear(&self);
     }

     /// Console statistics.

diff -uNr 06_drivers_gpio_uart/src/main.rs 07_uart_chainloader/src/main.rs
--- 06_drivers_gpio_uart/src/main.rs
+++ 07_uart_chainloader/src/main.rs
@@ -108,7 +108,8 @@
 #![no_std]

 // `mod cpu` provides the `_start()` function, the first function to run. `_start()` then calls
-// `runtime_init()`, which jumps to `kernel_init()`.
+// `relocate::relocate_self()`. `relocate::relocate_self()` calls `runtime_init()`, which jumps to
+// `kernel_init()`.

 mod bsp;
 mod console;
@@ -117,6 +118,7 @@
 mod memory;
 mod panic_wait;
 mod print;
+mod relocate;
 mod runtime_init;
 mod synchronization;

@@ -143,35 +145,52 @@

 /// The main function running after the early init.
 fn kernel_main() -> ! {
+    use bsp::console::console;
     use console::interface::All;
-    use driver::interface::DriverManager;

-    // Wait for user to hit Enter.
-    loop {
-        if bsp::console::console().read_char() == '\n' {
-            break;
+    println!(" __  __ _      _ _                 _ ");
+    println!("|  \\/  (_)_ _ (_) |   ___  __ _ __| |");
+    println!("| |\\/| | | ' \\| | |__/ _ \\/ _` / _` |");
+    println!("|_|  |_|_|_||_|_|____\\___/\\__,_\\__,_|");
+    println!();
+    println!("{:^37}", bsp::board_name());
+    println!();
+    println!("[ML] Requesting binary");
+    console().flush();
+
+    // Clear the RX FIFOs, if any, of spurious received characters before starting with the loader
+    // protocol.
+    console().clear();
+
+    // Notify `Minipush` to send the binary.
+    for _ in 0..3 {
+        console().write_char(3 as char);
+    }
+
+    // Read the binary's size.
+    let mut size: u32 = u32::from(console().read_char() as u8);
+    size |= u32::from(console().read_char() as u8) << 8;
+    size |= u32::from(console().read_char() as u8) << 16;
+    size |= u32::from(console().read_char() as u8) << 24;
+
+    // Trust it's not too big.
+    console().write_char('O');
+    console().write_char('K');
+
+    let kernel_addr: *mut u8 = bsp::cpu::BOARD_DEFAULT_LOAD_ADDRESS as *mut u8;
+    unsafe {
+        // Read the kernel byte by byte.
+        for i in 0..size {
+            *kernel_addr.offset(i as isize) = console().read_char() as u8;
         }
     }

-    println!("[0] Booting on: {}", bsp::board_name());
+    println!("[ML] Loaded! Executing the payload now\n");
+    console().flush();

-    println!("[1] Drivers loaded:");
-    for (i, driver) in bsp::driver::driver_manager()
-        .all_device_drivers()
-        .iter()
-        .enumerate()
-    {
-        println!("      {}. {}", i + 1, driver.compatible());
-    }
+    // Use black magic to get a function pointer.
+    let kernel: extern "C" fn() -> ! = unsafe { core::mem::transmute(kernel_addr as *const ()) };

-    println!(
-        "[2] Chars written: {}",
-        bsp::console::console().chars_written()
-    );
-    println!("[3] Echoing input now");
-
-    loop {
-        let c = bsp::console::console().read_char();
-        bsp::console::console().write_char(c);
-    }
+    // Jump to loaded kernel!
+    kernel()
 }

diff -uNr 06_drivers_gpio_uart/src/relocate.rs 07_uart_chainloader/src/relocate.rs
--- 06_drivers_gpio_uart/src/relocate.rs
+++ 07_uart_chainloader/src/relocate.rs
@@ -0,0 +1,52 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
+
+//! Relocation code.
+
+use crate::{bsp, runtime_init};
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+/// Relocates the own binary from `bsp::cpu::BOARD_DEFAULT_LOAD_ADDRESS` to the `__binary_start`
+/// address from the linker script.
+///
+/// # Safety
+///
+/// - Only a single core must be active and running this function.
+/// - Function must not use the `bss` section.
+pub unsafe fn relocate_self<T>() -> ! {
+    extern "C" {
+        static __binary_start: usize;
+        static __binary_end: usize;
+    }
+
+    let binary_start_addr: usize = &__binary_start as *const _ as _;
+    let binary_end_addr: usize = &__binary_end as *const _ as _;
+    let binary_size_in_byte: usize = binary_end_addr - binary_start_addr;
+
+    // Get the relocation destination address from the linker symbol.
+    let mut reloc_dst_addr: *mut T = binary_start_addr as *mut T;
+
+    // The address of where the previous firmware loaded us.
+    let mut src_addr: *const T = bsp::cpu::BOARD_DEFAULT_LOAD_ADDRESS as *const _;
+
+    // Copy the whole binary.
+    //
+    // This is essentially a `memcpy()` optimized for throughput by transferring in chunks of T.
+    let n = binary_size_in_byte / core::mem::size_of::<T>();
+    for _ in 0..n {
+        use core::ptr;
+
+        ptr::write_volatile::<T>(reloc_dst_addr, ptr::read_volatile::<T>(src_addr));
+        reloc_dst_addr = reloc_dst_addr.offset(1);
+        src_addr = src_addr.offset(1);
+    }
+
+    // Call `runtime_init()` through a trait object, causing the jump to use an absolute address to
+    // reach the relocated binary. An elaborate explanation can be found in the `runtime_init.rs`
+    // source comments.
+    runtime_init::get().runtime_init()
+}

diff -uNr 06_drivers_gpio_uart/src/runtime_init.rs 07_uart_chainloader/src/runtime_init.rs
--- 06_drivers_gpio_uart/src/runtime_init.rs
+++ 07_uart_chainloader/src/runtime_init.rs
@@ -8,9 +8,43 @@
 use core::ops::Range;

 //--------------------------------------------------------------------------------------------------
+// Private Definitions
+//--------------------------------------------------------------------------------------------------
+
+struct Traitor;
+
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
+/// We are outsmarting the compiler here by using a trait as a layer of indirection. Because we are
+/// generating PIC code, a static dispatch to `init()` would generate a relative jump from the
+/// callee to `init()`. However, when calling `init()`, code just finished copying the binary to the
+/// actual link-time address, and hence is still running at whatever location the previous loader
+/// has put it. So we do not want a relative jump, because it would not jump to the relocated code.
+///
+/// By indirecting through a trait object, we can make use of the property that vtables store
+/// absolute addresses. So calling `init()` this way will kick execution to the relocated binary.
+pub trait RunTimeInit {
+    /// Equivalent to `crt0` or `c0` code in C/C++ world. Clears the `bss` section, then jumps to
+    /// kernel init code.
+    ///
+    /// # Safety
+    ///
+    /// - Only a single core must be active and running this function.
+    unsafe fn runtime_init(&self) -> ! {
+        zero_bss();
+
+        crate::kernel_init()
+    }
+}
+
+//--------------------------------------------------------------------------------------------------
 // Private Code
 //--------------------------------------------------------------------------------------------------

+impl RunTimeInit for Traitor {}
+
 /// Return the range spanning the .bss section.
 ///
 /// # Safety
@@ -44,14 +78,7 @@
 // Public Code
 //--------------------------------------------------------------------------------------------------

-/// Equivalent to `crt0` or `c0` code in C/C++ world. Clears the `bss` section, then jumps to kernel
-/// init code.
-///
-/// # Safety
-///
-/// - Only a single core must be active and running this function.
-pub unsafe fn runtime_init() -> ! {
-    zero_bss();
-
-    crate::kernel_init()
+/// Give the callee a `RunTimeInit` trait object.
+pub fn get() -> &'static dyn RunTimeInit {
+    &Traitor {}
 }

```
