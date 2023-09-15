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
linked to (`0x200_0000`, see `kernel.ld`). The binary then copies itself from loaded to linked
address (aka  "relocating" itself), and then jumps to the relocated version of `_start_rust()`.

Since the chainloader has put itself "out of the way" now, it can now receive another kernel binary
from the `UART` and copy it to the standard load address of the RPi firmware at `0x8_0000`. Finally,
it jumps to `0x8_0000` and the newly loaded binary transparently executes as if it had been loaded
from SD card all along.

Please bear with me until I find the time to write it all down here elaborately. For the time being,
please see this tutorial as an enabler for a convenience feature that allows booting the following
tutorials in a quick manner. _For those keen to get a deeper understanding, it could make sense to
skip forward to [Chapter 15](../15_virtual_mem_part3_precomputed_tables) and read the first half of
the README, where `Load Address != Link Address` is discussed_.

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
[MP] ⏩ Pushing 7 KiB ==========================================🦀 100% 0 KiB/s Time: 00:00:00
[ML] Loaded! Executing the payload now

[0] mingo version 0.5.0
[1] Booting on: Raspberry Pi 3
[2] Drivers loaded:
      1. BCM PL011 UART
      2. BCM GPIO
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
 edition = "2021"

Binary files 05_drivers_gpio_uart/demo_payload_rpi3.img and 06_uart_chainloader/demo_payload_rpi3.img differ
Binary files 05_drivers_gpio_uart/demo_payload_rpi4.img and 06_uart_chainloader/demo_payload_rpi4.img differ

diff -uNr 05_drivers_gpio_uart/Makefile 06_uart_chainloader/Makefile
--- 05_drivers_gpio_uart/Makefile
+++ 06_uart_chainloader/Makefile
@@ -24,27 +24,29 @@
 QEMU_MISSING_STRING = "This board is not yet supported for QEMU."

 ifeq ($(BSP),rpi3)
-    TARGET            = aarch64-unknown-none-softfloat
-    KERNEL_BIN        = kernel8.img
-    QEMU_BINARY       = qemu-system-aarch64
-    QEMU_MACHINE_TYPE = raspi3
-    QEMU_RELEASE_ARGS = -serial stdio -display none
-    OBJDUMP_BINARY    = aarch64-none-elf-objdump
-    NM_BINARY         = aarch64-none-elf-nm
-    READELF_BINARY    = aarch64-none-elf-readelf
-    LD_SCRIPT_PATH    = $(shell pwd)/src/bsp/raspberrypi
-    RUSTC_MISC_ARGS   = -C target-cpu=cortex-a53
+    TARGET                 = aarch64-unknown-none-softfloat
+    KERNEL_BIN             = kernel8.img
+    QEMU_BINARY            = qemu-system-aarch64
+    QEMU_MACHINE_TYPE      = raspi3
+    QEMU_RELEASE_ARGS      = -serial stdio -display none
+    OBJDUMP_BINARY         = aarch64-none-elf-objdump
+    NM_BINARY              = aarch64-none-elf-nm
+    READELF_BINARY         = aarch64-none-elf-readelf
+    LD_SCRIPT_PATH         = $(shell pwd)/src/bsp/raspberrypi
+    RUSTC_MISC_ARGS        = -C target-cpu=cortex-a53
+    CHAINBOOT_DEMO_PAYLOAD = demo_payload_rpi3.img
 else ifeq ($(BSP),rpi4)
-    TARGET            = aarch64-unknown-none-softfloat
-    KERNEL_BIN        = kernel8.img
-    QEMU_BINARY       = qemu-system-aarch64
-    QEMU_MACHINE_TYPE =
-    QEMU_RELEASE_ARGS = -serial stdio -display none
-    OBJDUMP_BINARY    = aarch64-none-elf-objdump
-    NM_BINARY         = aarch64-none-elf-nm
-    READELF_BINARY    = aarch64-none-elf-readelf
-    LD_SCRIPT_PATH    = $(shell pwd)/src/bsp/raspberrypi
-    RUSTC_MISC_ARGS   = -C target-cpu=cortex-a72
+    TARGET                 = aarch64-unknown-none-softfloat
+    KERNEL_BIN             = kernel8.img
+    QEMU_BINARY            = qemu-system-aarch64
+    QEMU_MACHINE_TYPE      =
+    QEMU_RELEASE_ARGS      = -serial stdio -display none
+    OBJDUMP_BINARY         = aarch64-none-elf-objdump
+    NM_BINARY              = aarch64-none-elf-nm
+    READELF_BINARY         = aarch64-none-elf-readelf
+    LD_SCRIPT_PATH         = $(shell pwd)/src/bsp/raspberrypi
+    RUSTC_MISC_ARGS        = -C target-cpu=cortex-a72
+    CHAINBOOT_DEMO_PAYLOAD = demo_payload_rpi4.img
 endif

 # Export for build.rs.
@@ -90,8 +92,8 @@
     -O binary

 EXEC_QEMU          = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)
-EXEC_TEST_DISPATCH = ruby ../common/tests/dispatch.rb
-EXEC_MINITERM      = ruby ../common/serial/miniterm.rb
+EXEC_TEST_MINIPUSH = ruby tests/chainboot_test.rb
+EXEC_MINIPUSH      = ruby ../common/serial/minipush.rb

 ##------------------------------------------------------------------------------
 ## Dockerization
@@ -110,7 +112,7 @@
 ifeq ($(shell uname -s),Linux)
     DOCKER_CMD_DEV = $(DOCKER_CMD_INTERACT) $(DOCKER_ARG_DEV)

-    DOCKER_MINITERM = $(DOCKER_CMD_DEV) $(DOCKER_ARG_DIR_COMMON) $(DOCKER_IMAGE)
+    DOCKER_CHAINBOOT = $(DOCKER_CMD_DEV) $(DOCKER_ARG_DIR_COMMON) $(DOCKER_IMAGE)
 endif


@@ -118,7 +120,7 @@
 ##--------------------------------------------------------------------------------------------------
 ## Targets
 ##--------------------------------------------------------------------------------------------------
-.PHONY: all doc qemu miniterm clippy clean readelf objdump nm check
+.PHONY: all doc qemu chainboot clippy clean readelf objdump nm check

 all: $(KERNEL_BIN)

@@ -160,7 +162,7 @@
 ##------------------------------------------------------------------------------
 ifeq ($(QEMU_MACHINE_TYPE),) # QEMU is not supported for the board.

-qemu:
+qemu qemuasm:
 	$(call color_header, "$(QEMU_MISSING_STRING)")

 else # QEMU is supported.
@@ -169,13 +171,17 @@
 	$(call color_header, "Launching QEMU")
 	@$(DOCKER_QEMU) $(EXEC_QEMU) $(QEMU_RELEASE_ARGS) -kernel $(KERNEL_BIN)

+qemuasm: $(KERNEL_BIN)
+	$(call color_header, "Launching QEMU with ASM output")
+	@$(DOCKER_QEMU) $(EXEC_QEMU) $(QEMU_RELEASE_ARGS) -kernel $(KERNEL_BIN) -d in_asm
+
 endif
 ##------------------------------------------------------------------------------
-## Connect to the target's serial
+## Push the kernel to the real HW target
 ##------------------------------------------------------------------------------
-miniterm:
-	@$(DOCKER_MINITERM) $(EXEC_MINITERM) $(DEV_SERIAL)
+chainboot: $(KERNEL_BIN)
+	@$(DOCKER_CHAINBOOT) $(EXEC_MINIPUSH) $(DEV_SERIAL) $(CHAINBOOT_DEMO_PAYLOAD)

 ##------------------------------------------------------------------------------
 ## Run clippy
@@ -232,7 +238,8 @@
 ##------------------------------------------------------------------------------
 test_boot: $(KERNEL_BIN)
 	$(call color_header, "Boot test - $(BSP)")
-	@$(DOCKER_TEST) $(EXEC_TEST_DISPATCH) $(EXEC_QEMU) $(QEMU_RELEASE_ARGS) -kernel $(KERNEL_BIN)
+	@$(DOCKER_TEST) $(EXEC_TEST_MINIPUSH) $(EXEC_QEMU) $(QEMU_RELEASE_ARGS) \
+		-kernel $(KERNEL_BIN) $(CHAINBOOT_DEMO_PAYLOAD)

 test: test_boot


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
 //--------------------------------------------------------------------------------------------------
 // Public Code
 //--------------------------------------------------------------------------------------------------
@@ -37,23 +48,35 @@
 	// If execution reaches here, it is the boot core.

 	// Initialize DRAM.
-	ADR_REL	x0, __bss_start
-	ADR_REL x1, __bss_end_exclusive
+	ADR_ABS	x0, __bss_start
+	ADR_ABS x1, __bss_end_exclusive

 .L_bss_init_loop:
 	cmp	x0, x1
-	b.eq	.L_prepare_rust
+	b.eq	.L_relocate_binary
 	stp	xzr, xzr, [x0], #16
 	b	.L_bss_init_loop

+	// Next, relocate the binary.
+.L_relocate_binary:
+	ADR_REL	x0, __binary_nonzero_start         // The address the binary got loaded to.
+	ADR_ABS	x1, __binary_nonzero_start         // The address the binary was linked to.
+	ADR_ABS	x2, __binary_nonzero_end_exclusive
+
+.L_copy_loop:
+	ldr	x3, [x0], #8
+	str	x3, [x1], #8
+	cmp	x1, x2
+	b.lo	.L_copy_loop
+
 	// Prepare the jump to Rust code.
-.L_prepare_rust:
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
 .L_parking_loop:

diff -uNr 05_drivers_gpio_uart/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs 06_uart_chainloader/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
--- 05_drivers_gpio_uart/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
+++ 06_uart_chainloader/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
@@ -275,7 +275,7 @@
     }

     /// Retrieve a character.
-    fn read_char_converting(&mut self, blocking_mode: BlockingMode) -> Option<char> {
+    fn read_char(&mut self, blocking_mode: BlockingMode) -> Option<char> {
         // If RX FIFO is empty,
         if self.registers.FR.matches_all(FR::RXFE::SET) {
             // immediately return in non-blocking mode.
@@ -290,12 +290,7 @@
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
@@ -381,14 +376,14 @@
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

diff -uNr 05_drivers_gpio_uart/src/bsp/raspberrypi/console.rs 06_uart_chainloader/src/bsp/raspberrypi/console.rs
--- 05_drivers_gpio_uart/src/bsp/raspberrypi/console.rs
+++ 06_uart_chainloader/src/bsp/raspberrypi/console.rs
@@ -1,16 +0,0 @@
-// SPDX-License-Identifier: MIT OR Apache-2.0
-//
-// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
-
-//! BSP console facilities.
-
-use crate::console;
-
-//--------------------------------------------------------------------------------------------------
-// Public Code
-//--------------------------------------------------------------------------------------------------
-
-/// Return a reference to the console.
-pub fn console() -> &'static dyn console::interface::All {
-    &super::driver::PL011_UART
-}

diff -uNr 05_drivers_gpio_uart/src/bsp/raspberrypi/kernel.ld 06_uart_chainloader/src/bsp/raspberrypi/kernel.ld
--- 05_drivers_gpio_uart/src/bsp/raspberrypi/kernel.ld
+++ 06_uart_chainloader/src/bsp/raspberrypi/kernel.ld
@@ -3,8 +3,6 @@
  * Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
  */

-__rpi_phys_dram_start_addr = 0;
-
 /* The physical address at which the the kernel binary will be loaded by the Raspberry's firmware */
 __rpi_phys_binary_load_addr = 0x80000;

@@ -28,7 +26,8 @@

 SECTIONS
 {
-    . =  __rpi_phys_dram_start_addr;
+    /* Set the link address to 32 MiB */
+    . = 0x2000000;

     /***********************************************************************************************
     * Boot Core Stack
@@ -45,6 +44,7 @@
     /***********************************************************************************************
     * Code + RO Data + Global Offset Table
     ***********************************************************************************************/
+    __binary_nonzero_start = .;
     .text :
     {
         KEEP(*(.text._start))
@@ -60,6 +60,10 @@
     ***********************************************************************************************/
     .data : { *(.data*) } :segment_data

+    /* Fill up to 8 byte, b/c relocating the binary is done in u64 chunks */
+    . = ALIGN(8);
+    __binary_nonzero_end_exclusive = .;
+
     /* Section is zeroed in pairs of u64. Align start and end to 16 bytes */
     .bss (NOLOAD) : ALIGN(16)
     {

diff -uNr 05_drivers_gpio_uart/src/bsp/raspberrypi/memory.rs 06_uart_chainloader/src/bsp/raspberrypi/memory.rs
--- 05_drivers_gpio_uart/src/bsp/raspberrypi/memory.rs
+++ 06_uart_chainloader/src/bsp/raspberrypi/memory.rs
@@ -11,6 +11,7 @@
 /// The board's physical memory map.
 #[rustfmt::skip]
 pub(super) mod map {
+    pub const BOARD_DEFAULT_LOAD_ADDRESS: usize =        0x8_0000;

     pub const GPIO_OFFSET:         usize = 0x0020_0000;
     pub const UART_OFFSET:         usize = 0x0020_1000;
@@ -35,3 +36,13 @@
         pub const PL011_UART_START: usize = START + UART_OFFSET;
     }
 }
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+/// The address on which the Raspberry firmware loads every binary by default.
+#[inline(always)]
+pub fn board_default_load_addr() -> *const u64 {
+    map::BOARD_DEFAULT_LOAD_ADDRESS as _
+}

diff -uNr 05_drivers_gpio_uart/src/driver.rs 06_uart_chainloader/src/driver.rs
--- 05_drivers_gpio_uart/src/driver.rs
+++ 06_uart_chainloader/src/driver.rs
@@ -4,10 +4,7 @@

 //! Driver support.

-use crate::{
-    println,
-    synchronization::{interface::Mutex, NullLock},
-};
+use crate::synchronization::{interface::Mutex, NullLock};

 //--------------------------------------------------------------------------------------------------
 // Private Definitions
@@ -154,14 +151,4 @@
             }
         });
     }
-
-    /// Enumerate all registered device drivers.
-    pub fn enumerate(&self) {
-        let mut i: usize = 1;
-        self.for_each_descriptor(|descriptor| {
-            println!("      {}. {}", i, descriptor.device_driver.compatible());
-
-            i += 1;
-        });
-    }
 }

diff -uNr 05_drivers_gpio_uart/src/main.rs 06_uart_chainloader/src/main.rs
--- 05_drivers_gpio_uart/src/main.rs
+++ 06_uart_chainloader/src/main.rs
@@ -142,27 +142,55 @@
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
     use console::console;

-    println!(
-        "[0] {} version {}",
-        env!("CARGO_PKG_NAME"),
-        env!("CARGO_PKG_VERSION")
-    );
-    println!("[1] Booting on: {}", bsp::board_name());
+    println!("{}", MINILOAD_LOGO);
+    println!("{:^37}", bsp::board_name());
+    println!();
+    println!("[ML] Requesting binary");
+    console().flush();

-    println!("[2] Drivers loaded:");
-    driver::driver_manager().enumerate();
+    // Discard any spurious received characters before starting with the loader protocol.
+    console().clear_rx();

-    println!("[3] Chars written: {}", console().chars_written());
-    println!("[4] Echoing input now");
+    // Notify `Minipush` to send the binary.
+    for _ in 0..3 {
+        console().write_char(3 as char);
+    }

-    // Discard any spurious received characters before going into echo mode.
-    console().clear_rx();
-    loop {
-        let c = console().read_char();
-        console().write_char(c);
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
     }
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

diff -uNr 05_drivers_gpio_uart/tests/boot_test_string.rb 06_uart_chainloader/tests/boot_test_string.rb
--- 05_drivers_gpio_uart/tests/boot_test_string.rb
+++ 06_uart_chainloader/tests/boot_test_string.rb
@@ -1,3 +0,0 @@
-# frozen_string_literal: true
-
-EXPECTED_PRINT = 'Echoing input now'

diff -uNr 05_drivers_gpio_uart/tests/chainboot_test.rb 06_uart_chainloader/tests/chainboot_test.rb
--- 05_drivers_gpio_uart/tests/chainboot_test.rb
+++ 06_uart_chainloader/tests/chainboot_test.rb
@@ -0,0 +1,78 @@
+# frozen_string_literal: true
+
+# SPDX-License-Identifier: MIT OR Apache-2.0
+#
+# Copyright (c) 2020-2023 Andre Richter <andre.o.richter@gmail.com>
+
+require_relative '../../common/serial/minipush'
+require_relative '../../common/tests/boot_test'
+require 'pty'
+
+# Match for the last print that 'demo_payload_rpiX.img' produces.
+EXPECTED_PRINT = 'Echoing input now'
+
+# Wait for request to power the target.
+class PowerTargetRequestTest < SubtestBase
+    MINIPUSH_POWER_TARGET_REQUEST = 'Please power the target now'
+
+    def initialize(qemu_cmd, pty_main)
+        super()
+        @qemu_cmd = qemu_cmd
+        @pty_main = pty_main
+    end
+
+    def name
+        'Waiting for request to power target'
+    end
+
+    def run(qemu_out, _qemu_in)
+        expect_or_raise(qemu_out, MINIPUSH_POWER_TARGET_REQUEST)
+
+        # Now is the time to start QEMU with the chainloader binary. QEMU's virtual tty connects to
+        # the MiniPush instance spawned on pty_main, so that the two processes talk to each other.
+        Process.spawn(@qemu_cmd, in: @pty_main, out: @pty_main, err: '/dev/null')
+    end
+end
+
+# Extend BootTest so that it listens on the output of a MiniPush instance, which is itself connected
+# to a QEMU instance instead of a real HW.
+class ChainbootTest < BootTest
+    MINIPUSH = '../common/serial/minipush.rb'
+
+    def initialize(qemu_cmd, payload_path)
+        super(qemu_cmd, EXPECTED_PRINT)
+
+        @test_name = 'Boot test using Minipush'
+
+        @payload_path = payload_path
+    end
+
+    private
+
+    # override
+    def setup
+        pty_main, pty_secondary = PTY.open
+        mp_out, _mp_in = PTY.spawn("ruby #{MINIPUSH} #{pty_secondary.path} #{@payload_path}")
+
+        # The subtests (from this class and the parents) listen on @qemu_out_wrapped. Hence, point
+        # it to MiniPush's output.
+        @qemu_out_wrapped = PTYLoggerWrapper.new(mp_out, "\r\n")
+
+        # Important: Run this subtest before the one in the parent class.
+        @console_subtests.prepend(PowerTargetRequestTest.new(@qemu_cmd, pty_main))
+    end
+
+    # override
+    def finish
+        super()
+        @test_output.map! { |x| x.gsub(/.*\r/, '  ') }
+    end
+end
+
+##--------------------------------------------------------------------------------------------------
+## Execution starts here
+##--------------------------------------------------------------------------------------------------
+payload_path = ARGV.pop
+qemu_cmd = ARGV.join(' ')
+
+ChainbootTest.new(qemu_cmd, payload_path).run

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
