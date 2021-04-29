# Tutorial 06 - UART Chainloader

## tl;dr

- Running from an SD card was a nice experience, but it would be extremely tedious to do it for
  every new binary. So let's write a [chainloader].
- This will be the last binary you need to put on the SD card. Each following tutorial will provide
  a `chainboot` target in the `Makefile` that lets you conveniently load the kernel over `UART`.

[chainloader]: https://en.wikipedia.org/wiki/Chain_loading


## Note

Please note that there is stuff going on in this tutorial that is very hard to grasp by only looking
at the source code changes.

The gist of it is that in `boot.s`, we are writing a piece of [position independent code] which
automatically determines where the firmware has loaded the binary (`0x8_0000`), and where it was
linked to (`0x200_0000`, see `link.ld`). The binary then copies itself from loaded to linked address
(aka  "relocating" itself), and then jumps to the relocated version of `_start_rust()`.

Since the chainloader has put itself "out of the way" now, it can now receive another kernel binary
from the `UART` and copy it to the standard load address of the RPi firmware at `0x8_0000`. Finally,
it jumps to `0x8_0000` and the newly loaded binary transparently executes as if it had been loaded
from SD card all along.

Please bear with me until I find the time to write it all down here elaborately. For the time being,
please see this tutorial as an enabler for a convenience feature that allows booting the following
tutorials in a quick manner.

[position independent code]: https://en.wikipedia.org/wiki/Position-independent_code

## Install and test it

Our chainloader is called `MiniLoad` and is inspired by [raspbootin].

You can try it with this tutorial already:
1. Depending on your target hardware, run:`make` or `BSP=rpi4 make`.
1. Copy `kernel8.img` to the SD card and put the SD card back into your RPi.
1. Run `make chainboot` or `BSP=rpi4 make chainboot`.
1. Connect the USB serial to your host PC.
    - Wiring diagram at [top-level README](../README.md#-usb-serial-output).
    - Make sure that you **DID NOT** connect the power pin of the USB serial. Only RX/TX and GND.
1. Connect the RPi to the (USB) power cable.
1. Observe the loader fetching a kernel over `UART`:

> ❗ **NOTE**: `make chainboot` assumes a default serial device name of `/dev/ttyUSB0`. Depending on
> your host operating system, the device name might differ. For example, on `macOS`, it might be
> something like `/dev/tty.usbserial-0001`. In this case, please give the name explicitly:


```console
$ DEV_SERIAL=/dev/tty.usbserial-0001 make chainboot
```

[raspbootin]: https://github.com/mrvn/raspbootin

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
[MP] ⏩ Pushing 6 KiB ==========================================🦀 100% 0 KiB/s Time: 00:00:00
[ML] Loaded! Executing the payload now

[0] mingo version 0.5.0
[1] Booting on: Raspberry Pi 3
[2] Drivers loaded:
      1. BCM GPIO
      2. BCM PL011 UART
[3] Chars written: 117
[4] Echoing input now
```

In this tutorial, a version of the kernel from the previous tutorial is loaded for demo purposes. In
subsequent tutorials, it will be the working directory's kernel.

## Test it

The `Makefile` in this tutorial has an additional target, `qemuasm`, that lets you nicely observe
how the kernel, after relocating itself, jumps the load address region (`0x80_XXX`) to the relocated
code at (`0x0200_0XXX`):

```console
$ make qemuasm
[...]
N:
0x00080030:  58000140  ldr      x0, #0x80058
0x00080034:  9100001f  mov      sp, x0
0x00080038:  58000141  ldr      x1, #0x80060
0x0008003c:  d61f0020  br       x1

----------------
IN:
0x02000070:  9400044c  bl       #0x20011a0

----------------
IN:
0x020011a0:  90000008  adrp     x8, #0x2001000
0x020011a4:  90000009  adrp     x9, #0x2001000
0x020011a8:  f9446508  ldr      x8, [x8, #0x8c8]
0x020011ac:  f9446929  ldr      x9, [x9, #0x8d0]
0x020011b0:  eb08013f  cmp      x9, x8
0x020011b4:  54000109  b.ls     #0x20011d4
[...]
```

## Diff to previous
```diff

diff -uNr 05_drivers_gpio_uart/Cargo.toml 06_uart_chainloader/Cargo.toml
--- 05_drivers_gpio_uart/Cargo.toml
+++ 06_uart_chainloader/Cargo.toml
@@ -1,6 +1,6 @@
 [package]
 name = "mingo"
-version = "0.5.0"
+version = "0.6.0"
 authors = ["Andre Richter <andre.o.richter@gmail.com>"]
 edition = "2018"

Binary files 05_drivers_gpio_uart/demo_payload_rpi3.img and 06_uart_chainloader/demo_payload_rpi3.img differ
Binary files 05_drivers_gpio_uart/demo_payload_rpi4.img and 06_uart_chainloader/demo_payload_rpi4.img differ

diff -uNr 05_drivers_gpio_uart/Makefile 06_uart_chainloader/Makefile
--- 05_drivers_gpio_uart/Makefile
+++ 06_uart_chainloader/Makefile
@@ -25,6 +25,7 @@
     READELF_BINARY    = aarch64-none-elf-readelf
     LINKER_FILE       = src/bsp/raspberrypi/link.ld
     RUSTC_MISC_ARGS   = -C target-cpu=cortex-a53
+    CHAINBOOT_DEMO_PAYLOAD = demo_payload_rpi3.img
 else ifeq ($(BSP),rpi4)
     TARGET            = aarch64-unknown-none-softfloat
     KERNEL_BIN        = kernel8.img
@@ -36,6 +37,7 @@
     READELF_BINARY    = aarch64-none-elf-readelf
     LINKER_FILE       = src/bsp/raspberrypi/link.ld
     RUSTC_MISC_ARGS   = -C target-cpu=cortex-a72
+    CHAINBOOT_DEMO_PAYLOAD = demo_payload_rpi4.img
 endif

 # Export for build.rs
@@ -68,19 +70,22 @@
 DOCKER_ARG_DEV       = --privileged -v /dev:/dev

 DOCKER_QEMU  = $(DOCKER_CMD_INTERACT) $(DOCKER_IMAGE)
+DOCKER_TEST  = $(DOCKER_CMD) -t $(DOCKER_ARG_DIR_UTILS) $(DOCKER_IMAGE)
 DOCKER_TOOLS = $(DOCKER_CMD) $(DOCKER_IMAGE)

 # Dockerize commands that require USB device passthrough only on Linux
 ifeq ($(UNAME_S),Linux)
     DOCKER_CMD_DEV = $(DOCKER_CMD_INTERACT) $(DOCKER_ARG_DEV)

-    DOCKER_MINITERM = $(DOCKER_CMD_DEV) $(DOCKER_ARG_DIR_UTILS) $(DOCKER_IMAGE)
+    DOCKER_CHAINBOOT = $(DOCKER_CMD_DEV) $(DOCKER_ARG_DIR_UTILS) $(DOCKER_IMAGE)
 endif

-EXEC_QEMU     = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)
-EXEC_MINITERM = ruby ../utils/miniterm.rb
+EXEC_QEMU          = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)
+EXEC_MINIPUSH      = ruby ../utils/minipush.rb
+EXEC_QEMU_MINIPUSH = ruby tests/qemu_minipush.rb

-.PHONY: all $(KERNEL_ELF) $(KERNEL_BIN) doc qemu miniterm clippy clean readelf objdump nm check
+.PHONY: all $(KERNEL_ELF) $(KERNEL_BIN) doc qemu qemuasm chainboot clippy clean readelf objdump nm \
+    check

 all: $(KERNEL_BIN)

@@ -96,16 +101,26 @@
 	@$(DOC_CMD) --document-private-items --open

 ifeq ($(QEMU_MACHINE_TYPE),)
-qemu:
+qemu test:
 	$(call colorecho, "\n$(QEMU_MISSING_STRING)")
 else
 qemu: $(KERNEL_BIN)
 	$(call colorecho, "\nLaunching QEMU")
 	@$(DOCKER_QEMU) $(EXEC_QEMU) $(QEMU_RELEASE_ARGS) -kernel $(KERNEL_BIN)
+
+qemuasm: $(KERNEL_BIN)
+	$(call colorecho, "\nLaunching QEMU with ASM output")
+	@$(DOCKER_QEMU) $(EXEC_QEMU) $(QEMU_RELEASE_ARGS) -kernel $(KERNEL_BIN) -d in_asm
+
+test: $(KERNEL_BIN)
+	$(call colorecho, "\nTesting chainloading - $(BSP)")
+	@$(DOCKER_TEST) $(EXEC_QEMU_MINIPUSH) $(EXEC_QEMU) $(QEMU_RELEASE_ARGS) \
+                -kernel $(KERNEL_BIN) $(CHAINBOOT_DEMO_PAYLOAD)
+
 endif

-miniterm:
-	@$(DOCKER_MINITERM) $(EXEC_MINITERM) $(DEV_SERIAL)
+chainboot:
+	@$(DOCKER_CHAINBOOT) $(EXEC_MINIPUSH) $(DEV_SERIAL) $(CHAINBOOT_DEMO_PAYLOAD)

 clippy:
 	@RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(CLIPPY_CMD)

diff -uNr 05_drivers_gpio_uart/src/_arch/aarch64/cpu/boot.s 06_uart_chainloader/src/_arch/aarch64/cpu/boot.s
--- 05_drivers_gpio_uart/src/_arch/aarch64/cpu/boot.s
+++ 06_uart_chainloader/src/_arch/aarch64/cpu/boot.s
@@ -18,6 +18,17 @@
 	add	\register, \register, #:lo12:\symbol
 .endm

+// Load the address of a symbol into a register, absolute.
+//
+// # Resources
+//
+// - https://sourceware.org/binutils/docs-2.36/as/AArch64_002dRelocations.html
+.macro ADR_ABS register, symbol
+	movz	\register, #:abs_g2:\symbol
+	movk	\register, #:abs_g1_nc:\symbol
+	movk	\register, #:abs_g0_nc:\symbol
+.endm
+
 .equ _core_id_mask, 0b11

 //--------------------------------------------------------------------------------------------------
@@ -34,20 +45,31 @@
 	and	x1, x1, _core_id_mask
 	ldr	x2, BOOT_CORE_ID      // provided by bsp/__board_name__/cpu.rs
 	cmp	x1, x2
-	b.ne	1f
+	b.ne	2f
+
+	// If execution reaches here, it is the boot core.

-	// If execution reaches here, it is the boot core. Now, prepare the jump to Rust code.
+	// Next, relocate the binary.
+	ADR_REL	x0, __binary_nonzero_start         // The address the binary got loaded to.
+	ADR_ABS	x1, __binary_nonzero_start         // The address the binary was linked to.
+	ADR_ABS	x2, __binary_nonzero_end_exclusive
+
+1:	ldr	x3, [x0], #8
+	str	x3, [x1], #8
+	cmp	x1, x2
+	b.lo	1b

 	// Set the stack pointer.
-	ADR_REL	x0, __boot_core_stack_end_exclusive
+	ADR_ABS	x0, __boot_core_stack_end_exclusive
 	mov	sp, x0

-	// Jump to Rust code.
-	b	_start_rust
+	// Jump to the relocated Rust code.
+	ADR_ABS	x1, _start_rust
+	br	x1

 	// Infinitely wait for events (aka "park the core").
-1:	wfe
-	b	1b
+2:	wfe
+	b	2b

 .size	_start, . - _start
 .type	_start, function

diff -uNr 05_drivers_gpio_uart/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs 06_uart_chainloader/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs
--- 05_drivers_gpio_uart/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs
+++ 06_uart_chainloader/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs
@@ -144,7 +144,7 @@
         // Make an educated guess for a good delay value (Sequence described in the BCM2837
         // peripherals PDF).
         //
-        // - According to Wikipedia, the fastest Pi3 clocks around 1.4 GHz.
+        // - According to Wikipedia, the fastest RPi4 clocks around 1.5 GHz.
         // - The Linux 2837 GPIO driver waits 1 µs between the steps.
         //
         // So lets try to be on the safe side and default to 2000 cycles, which would equal 1 µs

diff -uNr 05_drivers_gpio_uart/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs 06_uart_chainloader/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
--- 05_drivers_gpio_uart/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
+++ 06_uart_chainloader/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
@@ -279,7 +279,7 @@
     }

     /// Retrieve a character.
-    fn read_char_converting(&mut self, blocking_mode: BlockingMode) -> Option<char> {
+    fn read_char(&mut self, blocking_mode: BlockingMode) -> Option<char> {
         // If RX FIFO is empty,
         if self.registers.FR.matches_all(FR::RXFE::SET) {
             // immediately return in non-blocking mode.
@@ -294,12 +294,7 @@
         }

         // Read one character.
-        let mut ret = self.registers.DR.get() as u8 as char;
-
-        // Convert carrige return to newline.
-        if ret == '\r' {
-            ret = '\n'
-        }
+        let ret = self.registers.DR.get() as u8 as char;

         // Update statistics.
         self.chars_read += 1;
@@ -379,14 +374,14 @@
 impl console::interface::Read for PL011Uart {
     fn read_char(&self) -> char {
         self.inner
-            .lock(|inner| inner.read_char_converting(BlockingMode::Blocking).unwrap())
+            .lock(|inner| inner.read_char(BlockingMode::Blocking).unwrap())
     }

     fn clear_rx(&self) {
         // Read from the RX FIFO until it is indicating empty.
         while self
             .inner
-            .lock(|inner| inner.read_char_converting(BlockingMode::NonBlocking))
+            .lock(|inner| inner.read_char(BlockingMode::NonBlocking))
             .is_some()
         {}
     }

diff -uNr 05_drivers_gpio_uart/src/bsp/raspberrypi/link.ld 06_uart_chainloader/src/bsp/raspberrypi/link.ld
--- 05_drivers_gpio_uart/src/bsp/raspberrypi/link.ld
+++ 06_uart_chainloader/src/bsp/raspberrypi/link.ld
@@ -16,7 +16,8 @@

 SECTIONS
 {
-    . =  __rpi_load_addr;
+    /* Set the link address to 32 MiB */
+    . = 0x2000000;
                                         /*   ^             */
                                         /*   | stack       */
                                         /*   | growth      */
@@ -26,6 +27,7 @@
     /***********************************************************************************************
     * Code + RO Data + Global Offset Table
     ***********************************************************************************************/
+    __binary_nonzero_start = .;
     .text :
     {
         KEEP(*(.text._start))
@@ -42,8 +44,12 @@
     ***********************************************************************************************/
     .data : { *(.data*) } :segment_rw

+    /* Fill up to 8 byte, b/c relocating the binary is done in u64 chunks */
+    . = ALIGN(8);
+    __binary_nonzero_end_exclusive = .;
+
     /* Section is zeroed in u64 chunks, align start and end to 8 bytes */
-    .bss : ALIGN(8)
+    .bss :
     {
         __bss_start = .;
         *(.bss*);

diff -uNr 05_drivers_gpio_uart/src/bsp/raspberrypi/memory.rs 06_uart_chainloader/src/bsp/raspberrypi/memory.rs
--- 05_drivers_gpio_uart/src/bsp/raspberrypi/memory.rs
+++ 06_uart_chainloader/src/bsp/raspberrypi/memory.rs
@@ -23,9 +23,10 @@
 /// The board's physical memory map.
 #[rustfmt::skip]
 pub(super) mod map {
+    pub const BOARD_DEFAULT_LOAD_ADDRESS: usize =        0x8_0000;

-    pub const GPIO_OFFSET:         usize = 0x0020_0000;
-    pub const UART_OFFSET:         usize = 0x0020_1000;
+    pub const GPIO_OFFSET:                usize =        0x0020_0000;
+    pub const UART_OFFSET:                usize =        0x0020_1000;

     /// Physical devices.
     #[cfg(feature = "bsp_rpi3")]
@@ -52,7 +53,13 @@
 // Public Code
 //--------------------------------------------------------------------------------------------------

-/// Return the inclusive range spanning the .bss section.
+/// The address on which the Raspberry firmware loads every binary by default.
+#[inline(always)]
+pub fn board_default_load_addr() -> *const u64 {
+    map::BOARD_DEFAULT_LOAD_ADDRESS as _
+}
+
+/// Return the inclusive range spanning the relocated .bss section.
 ///
 /// # Safety
 ///

diff -uNr 05_drivers_gpio_uart/src/main.rs 06_uart_chainloader/src/main.rs
--- 05_drivers_gpio_uart/src/main.rs
+++ 06_uart_chainloader/src/main.rs
@@ -107,6 +107,7 @@
 //! [`runtime_init::runtime_init()`]: runtime_init/fn.runtime_init.html

 #![allow(clippy::upper_case_acronyms)]
+#![feature(asm)]
 #![feature(const_fn_fn_ptr_basics)]
 #![feature(format_args_nl)]
 #![feature(global_asm)]
@@ -146,38 +147,56 @@
     kernel_main()
 }

+const MINILOAD_LOGO: &str = r#"
+ __  __ _      _ _                 _
+|  \/  (_)_ _ (_) |   ___  __ _ __| |
+| |\/| | | ' \| | |__/ _ \/ _` / _` |
+|_|  |_|_|_||_|_|____\___/\__,_\__,_|
+"#;
+
 /// The main function running after the early init.
 fn kernel_main() -> ! {
     use bsp::console::console;
     use console::interface::All;
-    use driver::interface::DriverManager;
-
-    println!(
-        "[0] {} version {}",
-        env!("CARGO_PKG_NAME"),
-        env!("CARGO_PKG_VERSION")
-    );
-    println!("[1] Booting on: {}", bsp::board_name());
-
-    println!("[2] Drivers loaded:");
-    for (i, driver) in bsp::driver::driver_manager()
-        .all_device_drivers()
-        .iter()
-        .enumerate()
-    {
-        println!("      {}. {}", i + 1, driver.compatible());
-    }

-    println!(
-        "[3] Chars written: {}",
-        bsp::console::console().chars_written()
-    );
-    println!("[4] Echoing input now");
+    println!("{}", MINILOAD_LOGO);
+    println!("{:^37}", bsp::board_name());
+    println!();
+    println!("[ML] Requesting binary");
+    console().flush();

-    // Discard any spurious received characters before going into echo mode.
+    // Discard any spurious received characters before starting with the loader protocol.
     console().clear_rx();
-    loop {
-        let c = bsp::console::console().read_char();
-        bsp::console::console().write_char(c);
+
+    // Notify `Minipush` to send the binary.
+    for _ in 0..3 {
+        console().write_char(3 as char);
     }
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
+    let kernel_addr: *mut u8 = bsp::memory::board_default_load_addr() as *mut u8;
+    unsafe {
+        // Read the kernel byte by byte.
+        for i in 0..size {
+            core::ptr::write_volatile(kernel_addr.offset(i as isize), console().read_char() as u8)
+        }
+    }
+
+    println!("[ML] Loaded! Executing the payload now\n");
+    console().flush();
+
+    // Use black magic to create a function pointer.
+    let kernel: fn() -> ! = unsafe { core::mem::transmute(kernel_addr) };
+
+    // Jump to loaded kernel!
+    kernel()
 }

diff -uNr 05_drivers_gpio_uart/tests/qemu_minipush.rb 06_uart_chainloader/tests/qemu_minipush.rb
--- 05_drivers_gpio_uart/tests/qemu_minipush.rb
+++ 06_uart_chainloader/tests/qemu_minipush.rb
@@ -0,0 +1,80 @@
+# frozen_string_literal: true
+
+# SPDX-License-Identifier: MIT OR Apache-2.0
+#
+# Copyright (c) 2020-2021 Andre Richter <andre.o.richter@gmail.com>
+
+require_relative '../../utils/minipush'
+require 'expect'
+require 'timeout'
+
+# Match for the last print that 'demo_payload_rpiX.img' produces.
+EXPECTED_PRINT = 'Echoing input now'
+
+# The main class
+class QEMUMiniPush < MiniPush
+    TIMEOUT_SECS = 3
+
+    # override
+    def initialize(qemu_cmd, binary_image_path)
+        super(nil, binary_image_path)
+
+        @qemu_cmd = qemu_cmd
+    end
+
+    private
+
+    def quit_qemu_graceful
+        Timeout.timeout(5) do
+            pid = @target_serial.pid
+            Process.kill('TERM', pid)
+            Process.wait(pid)
+        end
+    end
+
+    # override
+    def open_serial
+        @target_serial = IO.popen(@qemu_cmd, 'r+', err: '/dev/null')
+
+        # Ensure all output is immediately flushed to the device.
+        @target_serial.sync = true
+
+        puts "[#{@name_short}] ✅ Serial connected"
+    end
+
+    # override
+    def terminal
+        result = @target_serial.expect(EXPECTED_PRINT, TIMEOUT_SECS)
+        exit(1) if result.nil?
+
+        puts result
+
+        quit_qemu_graceful
+    end
+
+    # override
+    def connetion_reset; end
+
+    # override
+    def handle_reconnect(error)
+        handle_unexpected(error)
+    end
+end
+
+##--------------------------------------------------------------------------------------------------
+## Execution starts here
+##--------------------------------------------------------------------------------------------------
+puts
+puts 'QEMUMiniPush 1.0'.cyan
+puts
+
+# CTRL + C handler. Only here to suppress Ruby's default exception print.
+trap('INT') do
+    # The `ensure` block from `QEMUMiniPush::run` will run after exit, restoring console state.
+    exit
+end
+
+binary_image_path = ARGV.pop
+qemu_cmd = ARGV.join(' ')
+
+QEMUMiniPush.new(qemu_cmd, binary_image_path).run

diff -uNr 05_drivers_gpio_uart/update.sh 06_uart_chainloader/update.sh
--- 05_drivers_gpio_uart/update.sh
+++ 06_uart_chainloader/update.sh
@@ -0,0 +1,8 @@
+#!/usr/bin/env bash
+
+cd ../05_drivers_gpio_uart
+BSP=rpi4 make
+cp kernel8.img ../06_uart_chainloader/demo_payload_rpi4.img
+make
+cp kernel8.img ../06_uart_chainloader/demo_payload_rpi3.img
+rm kernel8.img

```
