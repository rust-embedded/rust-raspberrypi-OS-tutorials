# Tutorial 07 - UART Chainloader

## tl;dr

Running from an SD card was a nice experience, but it would be extremely tedious
to do it for every new binary. Let's write a [chainloader] using [position
independent code]. This will be the last binary you need to put on the SD card
for quite some time. Each following tutorial will provide a `chainboot` target in
the `Makefile` that lets you conveniently load the kernel over `UART`.

Our chainloader is called `MiniLoad` and is inspired by [raspbootin].

[chainloader]: https://en.wikipedia.org/wiki/Chain_loading
[position independent code]: https://en.wikipedia.org/wiki/Position-independent_code
[raspbootin]: https://github.com/mrvn/raspbootin

You can try it with this tutorial already:
1. Depending on your target hardware:`make` or `BSP=rpi4 make`.
2. Copy `kernel8.img` to the SD card.
3. Execute `make chainboot` or `BSP=rpi4 make chainboot`.
4. Now plug in the USB Serial.
5. Observe the loader fetching a kernel over `UART`:

```console
» make chainboot
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
      1. GPIO
      2. PL011Uart
[2] Chars written: 84
[3] Echoing input now
```

In this tutorial, a version of the kernel from the previous tutorial is loaded
for demo purposes. In subsequent tuts, it will be the working directory's
kernel.

## Test it

The `Makefile` in this tutorial has an additional target, `qemuasm`, that lets
you nicely observe the jump from the loaded address (`0x80_XXX`) to the
relocated code at (`0x3EFF_0XXX`):

```console
make qemuasm
[...]
IN:
0x000809fc:  d0000008  adrp     x8, #0x82000
0x00080a00:  52800020  movz     w0, #0x1
0x00080a04:  f9408908  ldr      x8, [x8, #0x110]
0x00080a08:  d63f0100  blr      x8

----------------
IN:
0x3eff0528:  d0000008  adrp     x8, #0x3eff2000
0x3eff052c:  d0000009  adrp     x9, #0x3eff2000
0x3eff0530:  f9411508  ldr      x8, [x8, #0x228]
0x3eff0534:  f9411929  ldr      x9, [x9, #0x230]
0x3eff0538:  eb08013f  cmp      x9, x8
0x3eff053c:  540000c2  b.hs     #0x3eff0554
[...]
```

## Diff to previous
```diff
Binary files 06_drivers_gpio_uart/demo_payload_rpi3.img and 07_uart_chainloader/demo_payload_rpi3.img differ
Binary files 06_drivers_gpio_uart/demo_payload_rpi4.img and 07_uart_chainloader/demo_payload_rpi4.img differ

diff -uNr 06_drivers_gpio_uart/Makefile 07_uart_chainloader/Makefile
--- 06_drivers_gpio_uart/Makefile
+++ 07_uart_chainloader/Makefile
@@ -7,6 +7,11 @@
 	BSP = rpi3
 endif

+# Default to /dev/ttyUSB0
+ifndef DEV_SERIAL
+	DEV_SERIAL = /dev/ttyUSB0
+endif
+
 # BSP-specific arguments
 ifeq ($(BSP),rpi3)
 	TARGET            = aarch64-unknown-none-softfloat
@@ -15,7 +20,8 @@
 	QEMU_MACHINE_TYPE = raspi3
 	QEMU_RELEASE_ARGS = -serial stdio -display none
 	LINKER_FILE       = src/bsp/rpi/link.ld
-	RUSTC_MISC_ARGS   = -C target-cpu=cortex-a53
+	RUSTC_MISC_ARGS   = -C target-cpu=cortex-a53 -C relocation-model=pic
+	CHAINBOOT_DEMO_PAYLOAD = demo_payload_rpi3.img
 else ifeq ($(BSP),rpi4)
 	TARGET            = aarch64-unknown-none-softfloat
 	OUTPUT            = kernel8.img
@@ -23,7 +29,8 @@
 	# QEMU_MACHINE_TYPE =
 	# QEMU_RELEASE_ARGS = -serial stdio -display none
 	LINKER_FILE       = src/bsp/rpi/link.ld
-	RUSTC_MISC_ARGS   = -C target-cpu=cortex-a72
+	RUSTC_MISC_ARGS   = -C target-cpu=cortex-a72 -C relocation-model=pic
+	CHAINBOOT_DEMO_PAYLOAD = demo_payload_rpi4.img
 endif

 RUSTFLAGS          = -C link-arg=-T$(LINKER_FILE) $(RUSTC_MISC_ARGS)
@@ -46,9 +53,12 @@
 DOCKER_IMAGE         = rustembedded/osdev-utils
 DOCKER_CMD           = docker run -it --rm
 DOCKER_ARG_DIR_TUT   = -v $(shell pwd):/work -w /work
+DOCKER_ARG_DIR_UTILS = -v $(shell pwd)/../utils:/utils
+DOCKER_ARG_TTY       = --privileged -v /dev:/dev
 DOCKER_EXEC_QEMU     = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)
+DOCKER_EXEC_MINIPUSH = ruby /utils/minipush.rb

-.PHONY: all doc qemu clippy clean readelf objdump nm
+.PHONY: all doc qemu qemuasm chainboot clippy clean readelf objdump nm

 all: clean $(OUTPUT)

@@ -66,13 +76,26 @@
 ifeq ($(QEMU_MACHINE_TYPE),)
 qemu:
 	@echo "This board is not yet supported for QEMU."
+
+qemuasm:
+	@echo "This board is not yet supported for QEMU."
 else
 qemu: all
 	@$(DOCKER_CMD) $(DOCKER_ARG_DIR_TUT) $(DOCKER_IMAGE) \
 		$(DOCKER_EXEC_QEMU) $(QEMU_RELEASE_ARGS)     \
 		-kernel $(OUTPUT)
+
+qemuasm: all
+	@$(DOCKER_CMD) $(DOCKER_ARG_DIR_TUT) $(DOCKER_IMAGE) \
+		$(DOCKER_EXEC_QEMU) $(QEMU_RELEASE_ARGS)     \
+		-kernel $(OUTPUT) -d in_asm
 endif

+chainboot:
+	@$(DOCKER_CMD) $(DOCKER_ARG_DIR_TUT) $(DOCKER_ARG_DIR_UTILS) $(DOCKER_ARG_TTY) \
+		$(DOCKER_IMAGE) $(DOCKER_EXEC_MINIPUSH) $(DEV_SERIAL)                  \
+		$(CHAINBOOT_DEMO_PAYLOAD)
+
 clippy:
 	RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" cargo xclippy --target=$(TARGET) --features bsp_$(BSP)


diff -uNr 06_drivers_gpio_uart/src/arch/aarch64.rs 07_uart_chainloader/src/arch/aarch64.rs
--- 06_drivers_gpio_uart/src/arch/aarch64.rs
+++ 07_uart_chainloader/src/arch/aarch64.rs
@@ -22,7 +22,7 @@

     if bsp::BOOT_CORE_ID == MPIDR_EL1.get() & CORE_MASK {
         SP.set(bsp::BOOT_CORE_STACK_START);
-        crate::runtime_init::runtime_init()
+        crate::relocate::relocate_self::<u64>()
     } else {
         // If not core0, infinitely wait for events.
         wait_forever()

diff -uNr 06_drivers_gpio_uart/src/bsp/driver/bcm/bcm2xxx_pl011_uart.rs 07_uart_chainloader/src/bsp/driver/bcm/bcm2xxx_pl011_uart.rs
--- 06_drivers_gpio_uart/src/bsp/driver/bcm/bcm2xxx_pl011_uart.rs
+++ 07_uart_chainloader/src/bsp/driver/bcm/bcm2xxx_pl011_uart.rs
@@ -272,6 +272,16 @@
         let mut r = &self.inner;
         r.lock(|inner| fmt::Write::write_fmt(inner, args))
     }
+
+    fn flush(&self) {
+        let mut r = &self.inner;
+        // Spin until TX FIFO empty is set.
+        r.lock(|inner| {
+            while !inner.FR.matches_all(FR::TXFE::SET) {
+                arch::nop();
+            }
+        });
+    }
 }

 impl interface::console::Read for PL011Uart {
@@ -283,18 +293,21 @@
                 arch::nop();
             }

-            // Read one character.
-            let mut ret = inner.DR.get() as u8 as char;
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
+            inner.DR.get() as u8 as char
+        })
+    }
+
+    fn clear(&self) {
+        let mut r = &self.inner;
+        r.lock(|inner| {
+            // Read from the RX FIFO until it is indicating empty.
+            while !inner.FR.matches_all(FR::RXFE::SET) {
+                inner.DR.get();
+            }
         })
     }
 }

diff -uNr 06_drivers_gpio_uart/src/bsp/rpi/link.ld 07_uart_chainloader/src/bsp/rpi/link.ld
--- 06_drivers_gpio_uart/src/bsp/rpi/link.ld
+++ 07_uart_chainloader/src/bsp/rpi/link.ld
@@ -5,9 +5,10 @@

 SECTIONS
 {
-    /* Set current address to the value from which the RPi starts execution */
-    . = 0x80000;
+    /* Set the link address to the top-most 40 KiB of DRAM (assuming 1GiB) */
+    . = 0x3F000000 - 0x10000;

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

diff -uNr 06_drivers_gpio_uart/src/bsp/rpi.rs 07_uart_chainloader/src/bsp/rpi.rs
--- 06_drivers_gpio_uart/src/bsp/rpi.rs
+++ 07_uart_chainloader/src/bsp/rpi.rs
@@ -16,6 +16,9 @@
 /// The early boot core's stack address.
 pub const BOOT_CORE_STACK_START: u64 = 0x80_000;

+/// The address on which the RPi3 firmware loads every binary by default.
+pub const BOARD_DEFAULT_LOAD_ADDRESS: usize = 0x80_000;
+
 //--------------------------------------------------------------------------------------------------
 // Global BSP driver instances
 //--------------------------------------------------------------------------------------------------

diff -uNr 06_drivers_gpio_uart/src/interface.rs 07_uart_chainloader/src/interface.rs
--- 06_drivers_gpio_uart/src/interface.rs
+++ 07_uart_chainloader/src/interface.rs
@@ -29,6 +29,10 @@

         /// Write a Rust format string.
         fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result;
+
+        /// Block execution until the last character has been physically put on the TX wire
+        /// (draining TX buffers/FIFOs, if any).
+        fn flush(&self);
     }

     /// Console read functions.
@@ -37,6 +41,9 @@
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
@@ -29,7 +29,11 @@
 // the first function to run.
 mod arch;

-// `_start()` then calls `runtime_init()`, which on completion, jumps to `kernel_init()`.
+// `_start()` then calls `relocate::relocate_self()`.
+mod relocate;
+
+// `relocate::relocate_self()` calls `runtime_init()`, which on completion, jumps to
+// `kernel_init()`.
 mod runtime_init;

 // Conditionally includes the selected `BSP` code.
@@ -65,25 +69,49 @@
 fn kernel_main() -> ! {
     use interface::console::All;

-    // UART should be functional now. Wait for user to hit Enter.
-    loop {
-        if bsp::console().read_char() == '\n' {
-            break;
-        }
+    println!(" __  __ _      _ _                 _ ");
+    println!("|  \\/  (_)_ _ (_) |   ___  __ _ __| |");
+    println!("| |\\/| | | ' \\| | |__/ _ \\/ _` / _` |");
+    println!("|_|  |_|_|_||_|_|____\\___/\\__,_\\__,_|");
+    println!();
+    println!("{:^37}", bsp::board_name());
+    println!();
+    println!("[ML] Requesting binary");
+    bsp::console().flush();
+
+    // Clear the RX FIFOs, if any, of spurious received characters before starting with the loader
+    // protocol.
+    bsp::console().clear();
+
+    // Notify `Minipush` to send the binary.
+    for _ in 0..3 {
+        bsp::console().write_char(3 as char);
     }

-    println!("[0] Booting on: {}", bsp::board_name());
-
-    println!("[1] Drivers loaded:");
-    for (i, driver) in bsp::device_drivers().iter().enumerate() {
-        println!("      {}. {}", i + 1, driver.compatible());
+    // Read the binary's size.
+    let mut size: u32 = u32::from(bsp::console().read_char() as u8);
+    size |= u32::from(bsp::console().read_char() as u8) << 8;
+    size |= u32::from(bsp::console().read_char() as u8) << 16;
+    size |= u32::from(bsp::console().read_char() as u8) << 24;
+
+    // Trust it's not too big.
+    bsp::console().write_char('O');
+    bsp::console().write_char('K');
+
+    let kernel_addr: *mut u8 = bsp::BOARD_DEFAULT_LOAD_ADDRESS as *mut u8;
+    unsafe {
+        // Read the kernel byte by byte.
+        for i in 0..size {
+            *kernel_addr.offset(i as isize) = bsp::console().read_char() as u8;
+        }
     }

-    println!("[2] Chars written: {}", bsp::console().chars_written());
-    println!("[3] Echoing input now");
+    println!("[ML] Loaded! Executing the payload now\n");
+    bsp::console().flush();

-    loop {
-        let c = bsp::console().read_char();
-        bsp::console().write_char(c);
-    }
+    // Use black magic to get a function pointer.
+    let kernel: extern "C" fn() -> ! = unsafe { core::mem::transmute(kernel_addr as *const ()) };
+
+    // Jump to loaded kernel!
+    kernel()
 }

diff -uNr 06_drivers_gpio_uart/src/relocate.rs 07_uart_chainloader/src/relocate.rs
--- 06_drivers_gpio_uart/src/relocate.rs
+++ 07_uart_chainloader/src/relocate.rs
@@ -0,0 +1,46 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
+
+//! Relocation code.
+
+/// Relocates the own binary from `bsp::BOARD_DEFAULT_LOAD_ADDRESS` to the `__binary_start` address
+/// from the linker script.
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
+    let mut src_addr: *const T = crate::bsp::BOARD_DEFAULT_LOAD_ADDRESS as *const _;
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
+    // Call `init()` through a trait object, causing the jump to use an absolute address to reach
+    // the relocated binary. An elaborate explanation can be found in the runtime_init.rs source
+    // comments.
+    crate::runtime_init::get().runtime_init()
+}

diff -uNr 06_drivers_gpio_uart/src/runtime_init.rs 07_uart_chainloader/src/runtime_init.rs
--- 06_drivers_gpio_uart/src/runtime_init.rs
+++ 07_uart_chainloader/src/runtime_init.rs
@@ -36,14 +36,32 @@
     memory::zero_volatile(bss_range());
 }

-/// Equivalent to `crt0` or `c0` code in C/C++ world. Clears the `bss` section, then jumps to kernel
-/// init code.
+/// We are outsmarting the compiler here by using a trait as a layer of indirection. Because we are
+/// generating PIC code, a static dispatch to `init()` would generate a relative jump from the
+/// callee to `init()`. However, when calling `init()`, code just finished copying the binary to the
+/// actual link-time address, and hence is still running at whatever location the previous loader
+/// has put it. So we do not want a relative jump, because it would not jump to the relocated code.
 ///
-/// # Safety
-///
-/// - Only a single core must be active and running this function.
-pub unsafe fn runtime_init() -> ! {
-    zero_bss();
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
+struct Traitor;
+impl RunTimeInit for Traitor {}

-    crate::kernel_init()
+/// Give the callee a `RunTimeInit` trait object.
+pub fn get() -> &'static dyn RunTimeInit {
+    &Traitor {}
 }

```
