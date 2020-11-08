# Tutorial 07 - UART Chainloader

## tl;dr

- Running from an SD card was a nice experience, but it would be extremely tedious to do it for
  every new binary. Let's write a [chainloader] using [position independent code].
- This will be the last binary you need to put on the SD card. Each following tutorial will provide
  a `chainboot` target in the `Makefile` that lets you conveniently load the kernel over `UART`.

[chainloader]: https://en.wikipedia.org/wiki/Chain_loading
[position independent code]: https://en.wikipedia.org/wiki/Position-independent_code

## Note

Please note that there is a lot of stuff going on in this tutorial that is very hard to grasp by
only looking at the source code changes.

Please bear with me until I find the time to write it all down here elaborately. For the time being,
please see this tutorial as an enabler for a convenience feature that allows booting the following
tutorials in a quick manner.

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
@@ -21,7 +21,8 @@
     OBJDUMP_BINARY    = aarch64-none-elf-objdump
     NM_BINARY         = aarch64-none-elf-nm
     LINKER_FILE       = src/bsp/raspberrypi/link.ld
-    RUSTC_MISC_ARGS   = -C target-cpu=cortex-a53
+    RUSTC_MISC_ARGS   = -C target-cpu=cortex-a53 -C relocation-model=pic
+    CHAINBOOT_DEMO_PAYLOAD = demo_payload_rpi3.img
 else ifeq ($(BSP),rpi4)
     TARGET            = aarch64-unknown-none-softfloat
     KERNEL_BIN        = kernel8.img
@@ -31,7 +32,8 @@
     OBJDUMP_BINARY    = aarch64-none-elf-objdump
     NM_BINARY         = aarch64-none-elf-nm
     LINKER_FILE       = src/bsp/raspberrypi/link.ld
-    RUSTC_MISC_ARGS   = -C target-cpu=cortex-a72
+    RUSTC_MISC_ARGS   = -C target-cpu=cortex-a72 -C relocation-model=pic
+    CHAINBOOT_DEMO_PAYLOAD = demo_payload_rpi4.img
 endif

 # Export for build.rs
@@ -67,13 +69,14 @@
 ifeq ($(UNAME_S),Linux)
     DOCKER_CMD_DEV = $(DOCKER_CMD_INTERACT) $(DOCKER_ARG_DEV)

-    DOCKER_MINITERM = $(DOCKER_CMD_DEV) $(DOCKER_ARG_DIR_UTILS) $(DOCKER_IMAGE)
+    DOCKER_CHAINBOOT = $(DOCKER_CMD_DEV) $(DOCKER_ARG_DIR_UTILS) $(DOCKER_IMAGE)
 endif

 EXEC_QEMU     = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)
-EXEC_MINITERM = ruby ../utils/miniterm.rb
+EXEC_MINIPUSH = ruby ../utils/minipush.rb

-.PHONY: all $(KERNEL_ELF) $(KERNEL_BIN) doc qemu miniterm clippy clean readelf objdump nm check
+.PHONY: all $(KERNEL_ELF) $(KERNEL_BIN) doc qemu qemuasm chainboot clippy clean readelf objdump nm \
+    check

 all: $(KERNEL_BIN)

@@ -87,15 +90,18 @@
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

-miniterm:
-	@$(DOCKER_MINITERM) $(EXEC_MINITERM) $(DEV_SERIAL)
+chainboot:
+	@$(DOCKER_CHAINBOOT) $(EXEC_MINIPUSH) $(DEV_SERIAL) $(CHAINBOOT_DEMO_PAYLOAD)

 clippy:
 	RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(CLIPPY_CMD)
@@ -107,7 +113,10 @@
 	readelf --headers $(KERNEL_ELF)

 objdump: $(KERNEL_ELF)
-	@$(DOCKER_ELFTOOLS) $(OBJDUMP_BINARY) --disassemble --demangle $(KERNEL_ELF)
+	@$(DOCKER_ELFTOOLS) $(OBJDUMP_BINARY) --disassemble --demangle \
+                --section .text \
+                --section .got  \
+                $(KERNEL_ELF)

 nm: $(KERNEL_ELF)
 	@$(DOCKER_ELFTOOLS) $(NM_BINARY) --demangle --print-size $(KERNEL_ELF) | sort

diff -uNr 06_drivers_gpio_uart/src/_arch/aarch64/cpu.rs 07_uart_chainloader/src/_arch/aarch64/cpu.rs
--- 06_drivers_gpio_uart/src/_arch/aarch64/cpu.rs
+++ 07_uart_chainloader/src/_arch/aarch64/cpu.rs
@@ -21,12 +21,12 @@
 #[naked]
 #[no_mangle]
 pub unsafe fn _start() -> ! {
-    use crate::runtime_init;
+    use crate::relocate;

     // Expect the boot core to start in EL2.
     if bsp::cpu::BOOT_CORE_ID == cpu::smp::core_id() {
         SP.set(bsp::memory::boot_core_stack_end() as u64);
-        runtime_init::runtime_init()
+        relocate::relocate_self()
     } else {
         // If not core0, infinitely wait for events.
         wait_forever()
@@ -54,3 +54,19 @@
         asm::wfe()
     }
 }
+
+/// Branch to a raw integer value.
+///
+/// # Safety
+///
+/// - This is highly unsafe. Use with care.
+#[inline(always)]
+pub unsafe fn branch_to_raw_addr(addr: usize) -> ! {
+    asm!(
+        "blr {destination:x}",
+        destination = in(reg) addr,
+        options(nomem, nostack)
+    );
+
+    core::intrinsics::unreachable()
+}

diff -uNr 06_drivers_gpio_uart/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs 07_uart_chainloader/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
--- 06_drivers_gpio_uart/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
+++ 07_uart_chainloader/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
@@ -271,6 +271,16 @@
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
@@ -282,18 +292,21 @@
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

diff -uNr 06_drivers_gpio_uart/src/bsp/raspberrypi/link.ld 07_uart_chainloader/src/bsp/raspberrypi/link.ld
--- 06_drivers_gpio_uart/src/bsp/raspberrypi/link.ld
+++ 07_uart_chainloader/src/bsp/raspberrypi/link.ld
@@ -5,12 +5,15 @@

 SECTIONS
 {
-    /* Set current address to the value from which the RPi starts execution */
-    . = 0x80000;
+    /* Set the link address to 32 MiB */
+    . = 0x2000000;

+    __binary_start = .;
     .text :
     {
-        *(.text._start) *(.text*)
+        *(.text._start)
+        KEEP(*(.text.runtime_init))
+        *(.text*);
     }

     .rodata :
@@ -35,5 +38,16 @@
         __bss_end_inclusive = . - 8;
     }

+    .got :
+    {
+        *(.got*)
+    }
+
+    /* Fill up to 8 byte, b/c relocating the binary is done in u64 chunks */
+    . = ALIGN(8);
+    __binary_end_inclusive = . - 8;
+
+    __runtime_init_reloc = runtime_init;
+
     /DISCARD/ : { *(.comment*) }
 }

diff -uNr 06_drivers_gpio_uart/src/bsp/raspberrypi/memory.rs 07_uart_chainloader/src/bsp/raspberrypi/memory.rs
--- 06_drivers_gpio_uart/src/bsp/raspberrypi/memory.rs
+++ 07_uart_chainloader/src/bsp/raspberrypi/memory.rs
@@ -12,6 +12,9 @@

 // Symbols from the linker script.
 extern "Rust" {
+    static __binary_start: UnsafeCell<u64>;
+    static __binary_end_inclusive: UnsafeCell<u64>;
+    static __runtime_init_reloc: UnsafeCell<u64>;
     static __bss_start: UnsafeCell<u64>;
     static __bss_end_inclusive: UnsafeCell<u64>;
 }
@@ -23,10 +26,12 @@
 /// The board's memory map.
 #[rustfmt::skip]
 pub(super) mod map {
-    pub const BOOT_CORE_STACK_END: usize = 0x8_0000;
+    pub const BOOT_CORE_STACK_END:        usize =        0x8_0000;

-    pub const GPIO_OFFSET:         usize = 0x0020_0000;
-    pub const UART_OFFSET:         usize = 0x0020_1000;
+    pub const BOARD_DEFAULT_LOAD_ADDRESS: usize =        0x8_0000;
+
+    pub const GPIO_OFFSET:                usize =        0x0020_0000;
+    pub const UART_OFFSET:                usize =        0x0020_1000;

     /// Physical devices.
     #[cfg(feature = "bsp_rpi3")]
@@ -59,13 +64,35 @@
     map::BOOT_CORE_STACK_END
 }

-/// Return the inclusive range spanning the .bss section.
+/// The address on which the Raspberry firmware loads every binary by default.
+#[inline(always)]
+pub fn board_default_load_addr() -> *const u64 {
+    map::BOARD_DEFAULT_LOAD_ADDRESS as _
+}
+
+/// Return the inclusive range spanning the relocated kernel binary.
+///
+/// # Safety
+///
+/// - Values are provided by the linker script and must be trusted as-is.
+/// - The linker-provided addresses must be u64 aligned.
+pub fn relocated_binary_range_inclusive() -> RangeInclusive<*mut u64> {
+    unsafe { RangeInclusive::new(__binary_start.get(), __binary_end_inclusive.get()) }
+}
+
+/// The relocated address of function `runtime_init()`.
+#[inline(always)]
+pub fn relocated_runtime_init_addr() -> *const u64 {
+    unsafe { __runtime_init_reloc.get() as _ }
+}
+
+/// Return the inclusive range spanning the relocated .bss section.
 ///
 /// # Safety
 ///
 /// - Values are provided by the linker script and must be trusted as-is.
 /// - The linker-provided addresses must be u64 aligned.
-pub fn bss_range_inclusive() -> RangeInclusive<*mut u64> {
+pub fn relocated_bss_range_inclusive() -> RangeInclusive<*mut u64> {
     let range;
     unsafe {
         range = RangeInclusive::new(__bss_start.get(), __bss_end_inclusive.get());

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
@@ -100,6 +100,8 @@
 //! - `crate::memory::*`
 //! - `crate::bsp::memory::*`

+#![feature(asm)]
+#![feature(core_intrinsics)]
 #![feature(format_args_nl)]
 #![feature(naked_functions)]
 #![feature(panic_info_message)]
@@ -108,7 +110,8 @@
 #![no_std]

 // `mod cpu` provides the `_start()` function, the first function to run. `_start()` then calls
-// `runtime_init()`, which jumps to `kernel_init()`.
+// `relocate::relocate_self()`. `relocate::relocate_self()` calls `runtime_init()`, which jumps to
+// `kernel_init()`.

 mod bsp;
 mod console;
@@ -117,6 +120,7 @@
 mod memory;
 mod panic_wait;
 mod print;
+mod relocate;
 mod runtime_init;
 mod synchronization;

@@ -143,35 +147,52 @@

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
+    let kernel_addr: *mut u8 = bsp::memory::board_default_load_addr() as *mut u8;
+    unsafe {
+        // Read the kernel byte by byte.
+        for i in 0..size {
+            core::ptr::write_volatile(kernel_addr.offset(i as isize), console().read_char() as u8)
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
+    let kernel: fn() -> ! = unsafe { core::mem::transmute(kernel_addr) };

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
@@ -0,0 +1,51 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
+
+//! Relocation code.
+
+use crate::{bsp, cpu};
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+/// Relocates the own binary from `bsp::memory::board_default_load_addr()` to the `__binary_start`
+/// address from the linker script.
+///
+/// # Safety
+///
+/// - Only a single core must be active and running this function.
+/// - Function must not use the `bss` section.
+#[inline(never)]
+pub unsafe fn relocate_self() -> ! {
+    let range = bsp::memory::relocated_binary_range_inclusive();
+    let mut relocated_binary_start_addr = *range.start();
+    let relocated_binary_end_addr_inclusive = *range.end();
+
+    // The address of where the previous firmware loaded us.
+    let mut current_binary_start_addr = bsp::memory::board_default_load_addr();
+
+    // Copy the whole binary.
+    while relocated_binary_start_addr <= relocated_binary_end_addr_inclusive {
+        core::ptr::write_volatile(
+            relocated_binary_start_addr,
+            core::ptr::read_volatile(current_binary_start_addr),
+        );
+        relocated_binary_start_addr = relocated_binary_start_addr.offset(1);
+        current_binary_start_addr = current_binary_start_addr.offset(1);
+    }
+
+    // The following function calls form a hack to achieve an "absolute jump" to
+    // `runtime_init::runtime_init()` by forcing an indirection through the global offset table
+    // (GOT), so that execution continues from the relocated binary.
+    //
+    // Without this, the address of `runtime_init()` would be calculated as a relative offset from
+    // the current program counter, since we are compiling as `position independent code`. This
+    // would cause us to keep executing from the address to which the firmware loaded us, instead of
+    // the relocated position.
+    //
+    // There likely is a more elegant way to do this.
+    let relocated_runtime_init_addr = bsp::memory::relocated_runtime_init_addr() as usize;
+    cpu::branch_to_raw_addr(relocated_runtime_init_addr)
+}

diff -uNr 06_drivers_gpio_uart/src/runtime_init.rs 07_uart_chainloader/src/runtime_init.rs
--- 06_drivers_gpio_uart/src/runtime_init.rs
+++ 07_uart_chainloader/src/runtime_init.rs
@@ -17,7 +17,7 @@
 /// - Must only be called pre `kernel_init()`.
 #[inline(always)]
 unsafe fn zero_bss() {
-    memory::zero_volatile(bsp::memory::bss_range_inclusive());
+    memory::zero_volatile(bsp::memory::relocated_bss_range_inclusive());
 }

 //--------------------------------------------------------------------------------------------------
@@ -30,6 +30,7 @@
 /// # Safety
 ///
 /// - Only a single core must be active and running this function.
+#[no_mangle]
 pub unsafe fn runtime_init() -> ! {
     zero_bss();

```
