# Tutorial 07 - Timestamps

## tl;dr

- We add abstractions for timer hardware, and implement them for the ARM architectural timer in
  `_arch/aarch64`.
- The new timer functions are used to annotate UART prints with timestamps, and to get rid of the
  cycle-based delays in the `GPIO` device driver, which boosts accuracy.
- A `warn!()` macro is added.

## Test it

Check it out via chainboot (added in previous tutorial):
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
[MP] ⏩ Pushing 12 KiB =========================================🦀 100% 0 KiB/s Time: 00:00:00
[ML] Loaded! Executing the payload now

[    0.140431] mingo version 0.7.0
[    0.140630] Booting on: Raspberry Pi 3
[    0.141085] Architectural timer resolution: 52 ns
[    0.141660] Drivers loaded:
[    0.141995]       1. BCM GPIO
[    0.142353]       2. BCM PL011 UART
[W   0.142777] Spin duration smaller than architecturally supported, skipping
[    0.143621] Spinning for 1 second
[    1.144023] Spinning for 1 second
[    2.144245] Spinning for 1 second
```

## Diff to previous
```diff

diff -uNr 06_uart_chainloader/Cargo.toml 07_timestamps/Cargo.toml
--- 06_uart_chainloader/Cargo.toml
+++ 07_timestamps/Cargo.toml
@@ -1,6 +1,6 @@
 [package]
 name = "mingo"
-version = "0.6.0"
+version = "0.7.0"
 authors = ["Andre Richter <andre.o.richter@gmail.com>"]
 edition = "2018"

Binary files 06_uart_chainloader/demo_payload_rpi3.img and 07_timestamps/demo_payload_rpi3.img differ
Binary files 06_uart_chainloader/demo_payload_rpi4.img and 07_timestamps/demo_payload_rpi4.img differ

diff -uNr 06_uart_chainloader/Makefile 07_timestamps/Makefile
--- 06_uart_chainloader/Makefile
+++ 07_timestamps/Makefile
@@ -22,29 +22,27 @@

 # BSP-specific arguments.
 ifeq ($(BSP),rpi3)
-    TARGET                 = aarch64-unknown-none-softfloat
-    KERNEL_BIN             = kernel8.img
-    QEMU_BINARY            = qemu-system-aarch64
-    QEMU_MACHINE_TYPE      = raspi3
-    QEMU_RELEASE_ARGS      = -serial stdio -display none
-    OBJDUMP_BINARY         = aarch64-none-elf-objdump
-    NM_BINARY              = aarch64-none-elf-nm
-    READELF_BINARY         = aarch64-none-elf-readelf
-    LINKER_FILE            = src/bsp/raspberrypi/link.ld
-    RUSTC_MISC_ARGS        = -C target-cpu=cortex-a53
-    CHAINBOOT_DEMO_PAYLOAD = demo_payload_rpi3.img
+    TARGET            = aarch64-unknown-none-softfloat
+    KERNEL_BIN        = kernel8.img
+    QEMU_BINARY       = qemu-system-aarch64
+    QEMU_MACHINE_TYPE = raspi3
+    QEMU_RELEASE_ARGS = -serial stdio -display none
+    OBJDUMP_BINARY    = aarch64-none-elf-objdump
+    NM_BINARY         = aarch64-none-elf-nm
+    READELF_BINARY    = aarch64-none-elf-readelf
+    LINKER_FILE       = src/bsp/raspberrypi/link.ld
+    RUSTC_MISC_ARGS   = -C target-cpu=cortex-a53
 else ifeq ($(BSP),rpi4)
-    TARGET                 = aarch64-unknown-none-softfloat
-    KERNEL_BIN             = kernel8.img
-    QEMU_BINARY            = qemu-system-aarch64
-    QEMU_MACHINE_TYPE      =
-    QEMU_RELEASE_ARGS      = -serial stdio -display none
-    OBJDUMP_BINARY         = aarch64-none-elf-objdump
-    NM_BINARY              = aarch64-none-elf-nm
-    READELF_BINARY         = aarch64-none-elf-readelf
-    LINKER_FILE            = src/bsp/raspberrypi/link.ld
-    RUSTC_MISC_ARGS        = -C target-cpu=cortex-a72
-    CHAINBOOT_DEMO_PAYLOAD = demo_payload_rpi4.img
+    TARGET            = aarch64-unknown-none-softfloat
+    KERNEL_BIN        = kernel8.img
+    QEMU_BINARY       = qemu-system-aarch64
+    QEMU_MACHINE_TYPE =
+    QEMU_RELEASE_ARGS = -serial stdio -display none
+    OBJDUMP_BINARY    = aarch64-none-elf-objdump
+    NM_BINARY         = aarch64-none-elf-nm
+    READELF_BINARY    = aarch64-none-elf-readelf
+    LINKER_FILE       = src/bsp/raspberrypi/link.ld
+    RUSTC_MISC_ARGS   = -C target-cpu=cortex-a72
 endif

 QEMU_MISSING_STRING = "This board is not yet supported for QEMU."
@@ -76,7 +74,7 @@
     -O binary

 EXEC_QEMU          = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)
-EXEC_TEST_MINIPUSH = ruby tests/chainboot_test.rb
+EXEC_TEST_DISPATCH = ruby ../common/tests/dispatch.rb
 EXEC_MINIPUSH      = ruby ../common/serial/minipush.rb

 ##------------------------------------------------------------------------------
@@ -133,7 +131,7 @@
 ##------------------------------------------------------------------------------
 ifeq ($(QEMU_MACHINE_TYPE),) # QEMU is not supported for the board.

-qemu qemuasm:
+qemu:
 	$(call colorecho, "\n$(QEMU_MISSING_STRING)")

 else # QEMU is supported.
@@ -142,17 +140,13 @@
 	$(call colorecho, "\nLaunching QEMU")
 	@$(DOCKER_QEMU) $(EXEC_QEMU) $(QEMU_RELEASE_ARGS) -kernel $(KERNEL_BIN)

-qemuasm: $(KERNEL_BIN)
-	$(call colorecho, "\nLaunching QEMU with ASM output")
-	@$(DOCKER_QEMU) $(EXEC_QEMU) $(QEMU_RELEASE_ARGS) -kernel $(KERNEL_BIN) -d in_asm
-
 endif

 ##------------------------------------------------------------------------------
 ## Push the kernel to the real HW target
 ##------------------------------------------------------------------------------
 chainboot: $(KERNEL_BIN)
-	@$(DOCKER_CHAINBOOT) $(EXEC_MINIPUSH) $(DEV_SERIAL) $(CHAINBOOT_DEMO_PAYLOAD)
+	@$(DOCKER_CHAINBOOT) $(EXEC_MINIPUSH) $(DEV_SERIAL) $(KERNEL_BIN)

 ##------------------------------------------------------------------------------
 ## Run clippy
@@ -216,8 +210,7 @@
 ##------------------------------------------------------------------------------
 test_boot: $(KERNEL_BIN)
 	$(call colorecho, "\nBoot test - $(BSP)")
-	@$(DOCKER_TEST) $(EXEC_TEST_MINIPUSH) $(EXEC_QEMU) $(QEMU_RELEASE_ARGS) \
-		-kernel $(KERNEL_BIN) $(CHAINBOOT_DEMO_PAYLOAD)
+	@$(DOCKER_TEST) $(EXEC_TEST_DISPATCH) $(EXEC_QEMU) $(QEMU_RELEASE_ARGS) -kernel $(KERNEL_BIN)

 test: test_boot


diff -uNr 06_uart_chainloader/src/_arch/aarch64/cpu/boot.s 07_timestamps/src/_arch/aarch64/cpu/boot.s
--- 06_uart_chainloader/src/_arch/aarch64/cpu/boot.s
+++ 07_timestamps/src/_arch/aarch64/cpu/boot.s
@@ -18,17 +18,6 @@
 	add	\register, \register, #:lo12:\symbol
 .endm

-// Load the address of a symbol into a register, absolute.
-//
-// # Resources
-//
-// - https://sourceware.org/binutils/docs-2.36/as/AArch64_002dRelocations.html
-.macro ADR_ABS register, symbol
-	movz	\register, #:abs_g2:\symbol
-	movk	\register, #:abs_g1_nc:\symbol
-	movk	\register, #:abs_g0_nc:\symbol
-.endm
-
 .equ _core_id_mask, 0b11

 //--------------------------------------------------------------------------------------------------
@@ -50,35 +39,23 @@
 	// If execution reaches here, it is the boot core.

 	// Initialize DRAM.
-	ADR_ABS	x0, __bss_start
-	ADR_ABS x1, __bss_end_exclusive
+	ADR_REL	x0, __bss_start
+	ADR_REL x1, __bss_end_exclusive

 .L_bss_init_loop:
 	cmp	x0, x1
-	b.eq	.L_relocate_binary
+	b.eq	.L_prepare_rust
 	stp	xzr, xzr, [x0], #16
 	b	.L_bss_init_loop

-	// Next, relocate the binary.
-.L_relocate_binary:
-	ADR_REL	x0, __binary_nonzero_start         // The address the binary got loaded to.
-	ADR_ABS	x1, __binary_nonzero_start         // The address the binary was linked to.
-	ADR_ABS	x2, __binary_nonzero_end_exclusive
-
-.L_copy_loop:
-	ldr	x3, [x0], #8
-	str	x3, [x1], #8
-	cmp	x1, x2
-	b.lo	.L_copy_loop
-
 	// Prepare the jump to Rust code.
+.L_prepare_rust:
 	// Set the stack pointer.
-	ADR_ABS	x0, __boot_core_stack_end_exclusive
+	ADR_REL	x0, __boot_core_stack_end_exclusive
 	mov	sp, x0

-	// Jump to the relocated Rust code.
-	ADR_ABS	x1, _start_rust
-	br	x1
+	// Jump to Rust code.
+	b	_start_rust

 	// Infinitely wait for events (aka "park the core").
 .L_parking_loop:

diff -uNr 06_uart_chainloader/src/_arch/aarch64/cpu.rs 07_timestamps/src/_arch/aarch64/cpu.rs
--- 06_uart_chainloader/src/_arch/aarch64/cpu.rs
+++ 07_timestamps/src/_arch/aarch64/cpu.rs
@@ -19,15 +19,6 @@

 pub use asm::nop;

-/// Spin for `n` cycles.
-#[cfg(feature = "bsp_rpi3")]
-#[inline(always)]
-pub fn spin_for_cycles(n: usize) {
-    for _ in 0..n {
-        asm::nop();
-    }
-}
-
 /// Pause execution on the core.
 #[inline(always)]
 pub fn wait_forever() -> ! {

diff -uNr 06_uart_chainloader/src/_arch/aarch64/time.rs 07_timestamps/src/_arch/aarch64/time.rs
--- 06_uart_chainloader/src/_arch/aarch64/time.rs
+++ 07_timestamps/src/_arch/aarch64/time.rs
@@ -0,0 +1,119 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
+
+//! Architectural timer primitives.
+//!
+//! # Orientation
+//!
+//! Since arch modules are imported into generic modules using the path attribute, the path of this
+//! file is:
+//!
+//! crate::time::arch_time
+
+use crate::{time, warn};
+use core::time::Duration;
+use cortex_a::{asm::barrier, registers::*};
+use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};
+
+//--------------------------------------------------------------------------------------------------
+// Private Definitions
+//--------------------------------------------------------------------------------------------------
+
+const NS_PER_S: u64 = 1_000_000_000;
+
+/// ARMv8 Generic Timer.
+struct GenericTimer;
+
+//--------------------------------------------------------------------------------------------------
+// Global instances
+//--------------------------------------------------------------------------------------------------
+
+static TIME_MANAGER: GenericTimer = GenericTimer;
+
+//--------------------------------------------------------------------------------------------------
+// Private Code
+//--------------------------------------------------------------------------------------------------
+
+impl GenericTimer {
+    #[inline(always)]
+    fn read_cntpct(&self) -> u64 {
+        // Prevent that the counter is read ahead of time due to out-of-order execution.
+        unsafe { barrier::isb(barrier::SY) };
+        CNTPCT_EL0.get()
+    }
+}
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+/// Return a reference to the time manager.
+pub fn time_manager() -> &'static impl time::interface::TimeManager {
+    &TIME_MANAGER
+}
+
+//------------------------------------------------------------------------------
+// OS Interface Code
+//------------------------------------------------------------------------------
+
+impl time::interface::TimeManager for GenericTimer {
+    fn resolution(&self) -> Duration {
+        Duration::from_nanos(NS_PER_S / (CNTFRQ_EL0.get() as u64))
+    }
+
+    fn uptime(&self) -> Duration {
+        let current_count: u64 = self.read_cntpct() * NS_PER_S;
+        let frq: u64 = CNTFRQ_EL0.get() as u64;
+
+        Duration::from_nanos(current_count / frq)
+    }
+
+    fn spin_for(&self, duration: Duration) {
+        // Instantly return on zero.
+        if duration.as_nanos() == 0 {
+            return;
+        }
+
+        // Calculate the register compare value.
+        let frq = CNTFRQ_EL0.get();
+        let x = match frq.checked_mul(duration.as_nanos() as u64) {
+            None => {
+                warn!("Spin duration too long, skipping");
+                return;
+            }
+            Some(val) => val,
+        };
+        let tval = x / NS_PER_S;
+
+        // Check if it is within supported bounds.
+        let warn: Option<&str> = if tval == 0 {
+            Some("smaller")
+        // The upper 32 bits of CNTP_TVAL_EL0 are reserved.
+        } else if tval > u32::max_value().into() {
+            Some("bigger")
+        } else {
+            None
+        };
+
+        if let Some(w) = warn {
+            warn!(
+                "Spin duration {} than architecturally supported, skipping",
+                w
+            );
+            return;
+        }
+
+        // Set the compare value register.
+        CNTP_TVAL_EL0.set(tval);
+
+        // Kick off the counting.                       // Disable timer interrupt.
+        CNTP_CTL_EL0.modify(CNTP_CTL_EL0::ENABLE::SET + CNTP_CTL_EL0::IMASK::SET);
+
+        // ISTATUS will be '1' when cval ticks have passed. Busy-check it.
+        while !CNTP_CTL_EL0.matches_all(CNTP_CTL_EL0::ISTATUS::SET) {}
+
+        // Disable counting again.
+        CNTP_CTL_EL0.modify(CNTP_CTL_EL0::ENABLE::CLEAR);
+    }
+}

diff -uNr 06_uart_chainloader/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs 07_timestamps/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs
--- 06_uart_chainloader/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs
+++ 07_timestamps/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs
@@ -143,25 +143,19 @@
     /// Disable pull-up/down on pins 14 and 15.
     #[cfg(feature = "bsp_rpi3")]
     fn disable_pud_14_15_bcm2837(&mut self) {
-        use crate::cpu;
+        use crate::{time, time::interface::TimeManager};
+        use core::time::Duration;

-        // Make an educated guess for a good delay value (Sequence described in the BCM2837
-        // peripherals PDF).
-        //
-        // - According to Wikipedia, the fastest RPi4 clocks around 1.5 GHz.
-        // - The Linux 2837 GPIO driver waits 1 µs between the steps.
-        //
-        // So lets try to be on the safe side and default to 2000 cycles, which would equal 1 µs
-        // would the CPU be clocked at 2 GHz.
-        const DELAY: usize = 2000;
+        // The Linux 2837 GPIO driver waits 1 µs between the steps.
+        const DELAY: Duration = Duration::from_micros(1);

         self.registers.GPPUD.write(GPPUD::PUD::Off);
-        cpu::spin_for_cycles(DELAY);
+        time::time_manager().spin_for(DELAY);

         self.registers
             .GPPUDCLK0
             .write(GPPUDCLK0::PUDCLK15::AssertClock + GPPUDCLK0::PUDCLK14::AssertClock);
-        cpu::spin_for_cycles(DELAY);
+        time::time_manager().spin_for(DELAY);

         self.registers.GPPUD.write(GPPUD::PUD::Off);
         self.registers.GPPUDCLK0.set(0);

diff -uNr 06_uart_chainloader/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs 07_timestamps/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
--- 06_uart_chainloader/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
+++ 07_timestamps/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
@@ -284,7 +284,7 @@
     }

     /// Retrieve a character.
-    fn read_char(&mut self, blocking_mode: BlockingMode) -> Option<char> {
+    fn read_char_converting(&mut self, blocking_mode: BlockingMode) -> Option<char> {
         // If RX FIFO is empty,
         if self.registers.FR.matches_all(FR::RXFE::SET) {
             // immediately return in non-blocking mode.
@@ -299,7 +299,12 @@
         }

         // Read one character.
-        let ret = self.registers.DR.get() as u8 as char;
+        let mut ret = self.registers.DR.get() as u8 as char;
+
+        // Convert carrige return to newline.
+        if ret == '\r' {
+            ret = '\n'
+        }

         // Update statistics.
         self.chars_read += 1;
@@ -379,14 +384,14 @@
 impl console::interface::Read for PL011Uart {
     fn read_char(&self) -> char {
         self.inner
-            .lock(|inner| inner.read_char(BlockingMode::Blocking).unwrap())
+            .lock(|inner| inner.read_char_converting(BlockingMode::Blocking).unwrap())
     }

     fn clear_rx(&self) {
         // Read from the RX FIFO until it is indicating empty.
         while self
             .inner
-            .lock(|inner| inner.read_char(BlockingMode::NonBlocking))
+            .lock(|inner| inner.read_char_converting(BlockingMode::NonBlocking))
             .is_some()
         {}
     }

diff -uNr 06_uart_chainloader/src/bsp/raspberrypi/link.ld 07_timestamps/src/bsp/raspberrypi/link.ld
--- 06_uart_chainloader/src/bsp/raspberrypi/link.ld
+++ 07_timestamps/src/bsp/raspberrypi/link.ld
@@ -16,8 +16,7 @@

 SECTIONS
 {
-    /* Set the link address to 32 MiB */
-    . = 0x2000000;
+    . =  __rpi_load_addr;
                                         /*   ^             */
                                         /*   | stack       */
                                         /*   | growth      */
@@ -27,7 +26,6 @@
     /***********************************************************************************************
     * Code + RO Data + Global Offset Table
     ***********************************************************************************************/
-    __binary_nonzero_start = .;
     .text :
     {
         KEEP(*(.text._start))
@@ -44,10 +42,6 @@
     ***********************************************************************************************/
     .data : { *(.data*) } :segment_rw

-    /* Fill up to 8 byte, b/c relocating the binary is done in u64 chunks */
-    . = ALIGN(8);
-    __binary_nonzero_end_exclusive = .;
-
     /* Section is zeroed in pairs of u64. Align start and end to 16 bytes */
     .bss : ALIGN(16)
     {

diff -uNr 06_uart_chainloader/src/bsp/raspberrypi/memory.rs 07_timestamps/src/bsp/raspberrypi/memory.rs
--- 06_uart_chainloader/src/bsp/raspberrypi/memory.rs
+++ 07_timestamps/src/bsp/raspberrypi/memory.rs
@@ -11,10 +11,9 @@
 /// The board's physical memory map.
 #[rustfmt::skip]
 pub(super) mod map {
-    pub const BOARD_DEFAULT_LOAD_ADDRESS: usize =        0x8_0000;

-    pub const GPIO_OFFSET:                usize =        0x0020_0000;
-    pub const UART_OFFSET:                usize =        0x0020_1000;
+    pub const GPIO_OFFSET:         usize = 0x0020_0000;
+    pub const UART_OFFSET:         usize = 0x0020_1000;

     /// Physical devices.
     #[cfg(feature = "bsp_rpi3")]
@@ -36,13 +35,3 @@
         pub const PL011_UART_START: usize = START + UART_OFFSET;
     }
 }
-
-//--------------------------------------------------------------------------------------------------
-// Public Code
-//--------------------------------------------------------------------------------------------------
-
-/// The address on which the Raspberry firmware loads every binary by default.
-#[inline(always)]
-pub fn board_default_load_addr() -> *const u64 {
-    map::BOARD_DEFAULT_LOAD_ADDRESS as _
-}

diff -uNr 06_uart_chainloader/src/cpu.rs 07_timestamps/src/cpu.rs
--- 06_uart_chainloader/src/cpu.rs
+++ 07_timestamps/src/cpu.rs
@@ -14,6 +14,3 @@
 // Architectural Public Reexports
 //--------------------------------------------------------------------------------------------------
 pub use arch_cpu::{nop, wait_forever};
-
-#[cfg(feature = "bsp_rpi3")]
-pub use arch_cpu::spin_for_cycles;

diff -uNr 06_uart_chainloader/src/main.rs 07_timestamps/src/main.rs
--- 06_uart_chainloader/src/main.rs
+++ 07_timestamps/src/main.rs
@@ -105,7 +105,6 @@
 //! 2. Once finished with architectural setup, the arch code calls `kernel_init()`.

 #![allow(clippy::upper_case_acronyms)]
-#![feature(asm)]
 #![feature(const_fn_fn_ptr_basics)]
 #![feature(format_args_nl)]
 #![feature(global_asm)]
@@ -121,6 +120,7 @@
 mod panic_wait;
 mod print;
 mod synchronization;
+mod time;

 /// Early init code.
 ///
@@ -143,56 +143,38 @@
     kernel_main()
 }

-const MINILOAD_LOGO: &str = r#"
- __  __ _      _ _                 _
-|  \/  (_)_ _ (_) |   ___  __ _ __| |
-| |\/| | | ' \| | |__/ _ \/ _` / _` |
-|_|  |_|_|_||_|_|____\___/\__,_\__,_|
-"#;
-
 /// The main function running after the early init.
 fn kernel_main() -> ! {
-    use bsp::console::console;
-    use console::interface::All;
+    use core::time::Duration;
+    use driver::interface::DriverManager;
+    use time::interface::TimeManager;

-    println!("{}", MINILOAD_LOGO);
-    println!("{:^37}", bsp::board_name());
-    println!();
-    println!("[ML] Requesting binary");
-    console().flush();
-
-    // Discard any spurious received characters before starting with the loader protocol.
-    console().clear_rx();
-
-    // Notify `Minipush` to send the binary.
-    for _ in 0..3 {
-        console().write_char(3 as char);
+    info!(
+        "{} version {}",
+        env!("CARGO_PKG_NAME"),
+        env!("CARGO_PKG_VERSION")
+    );
+    info!("Booting on: {}", bsp::board_name());
+
+    info!(
+        "Architectural timer resolution: {} ns",
+        time::time_manager().resolution().as_nanos()
+    );
+
+    info!("Drivers loaded:");
+    for (i, driver) in bsp::driver::driver_manager()
+        .all_device_drivers()
+        .iter()
+        .enumerate()
+    {
+        info!("      {}. {}", i + 1, driver.compatible());
     }

-    // Read the binary's size.
-    let mut size: u32 = u32::from(console().read_char() as u8);
-    size |= u32::from(console().read_char() as u8) << 8;
-    size |= u32::from(console().read_char() as u8) << 16;
-    size |= u32::from(console().read_char() as u8) << 24;
-
-    // Trust it's not too big.
-    console().write_char('O');
-    console().write_char('K');
-
-    let kernel_addr: *mut u8 = bsp::memory::board_default_load_addr() as *mut u8;
-    unsafe {
-        // Read the kernel byte by byte.
-        for i in 0..size {
-            core::ptr::write_volatile(kernel_addr.offset(i as isize), console().read_char() as u8)
-        }
-    }
+    // Test a failing timer case.
+    time::time_manager().spin_for(Duration::from_nanos(1));

-    println!("[ML] Loaded! Executing the payload now\n");
-    console().flush();
-
-    // Use black magic to create a function pointer.
-    let kernel: fn() -> ! = unsafe { core::mem::transmute(kernel_addr) };
-
-    // Jump to loaded kernel!
-    kernel()
+    loop {
+        info!("Spinning for 1 second");
+        time::time_manager().spin_for(Duration::from_secs(1));
+    }
 }

diff -uNr 06_uart_chainloader/src/print.rs 07_timestamps/src/print.rs
--- 06_uart_chainloader/src/print.rs
+++ 07_timestamps/src/print.rs
@@ -36,3 +36,71 @@
         $crate::print::_print(format_args_nl!($($arg)*));
     })
 }
+
+/// Prints an info, with a newline.
+#[macro_export]
+macro_rules! info {
+    ($string:expr) => ({
+        #[allow(unused_imports)]
+        use crate::time::interface::TimeManager;
+
+        let timestamp = $crate::time::time_manager().uptime();
+        let timestamp_subsec_us = timestamp.subsec_micros();
+
+        $crate::print::_print(format_args_nl!(
+            concat!("[  {:>3}.{:03}{:03}] ", $string),
+            timestamp.as_secs(),
+            timestamp_subsec_us / 1_000,
+            timestamp_subsec_us modulo 1_000
+        ));
+    });
+    ($format_string:expr, $($arg:tt)*) => ({
+        #[allow(unused_imports)]
+        use crate::time::interface::TimeManager;
+
+        let timestamp = $crate::time::time_manager().uptime();
+        let timestamp_subsec_us = timestamp.subsec_micros();
+
+        $crate::print::_print(format_args_nl!(
+            concat!("[  {:>3}.{:03}{:03}] ", $format_string),
+            timestamp.as_secs(),
+            timestamp_subsec_us / 1_000,
+            timestamp_subsec_us modulo 1_000,
+            $($arg)*
+        ));
+    })
+}
+
+/// Prints a warning, with a newline.
+#[macro_export]
+macro_rules! warn {
+    ($string:expr) => ({
+        #[allow(unused_imports)]
+        use crate::time::interface::TimeManager;
+
+        let timestamp = $crate::time::time_manager().uptime();
+        let timestamp_subsec_us = timestamp.subsec_micros();
+
+        $crate::print::_print(format_args_nl!(
+            concat!("[W {:>3}.{:03}{:03}] ", $string),
+            timestamp.as_secs(),
+            timestamp_subsec_us / 1_000,
+            timestamp_subsec_us modulo 1_000
+        ));
+    });
+    ($format_string:expr, $($arg:tt)*) => ({
+        #[allow(unused_imports)]
+        use crate::time::interface::TimeManager;
+
+        let timestamp = $crate::time::time_manager().uptime();
+        let timestamp_subsec_us = timestamp.subsec_micros();
+
+        $crate::print::_print(format_args_nl!(
+            concat!("[W {:>3}.{:03}{:03}] ", $format_string),
+            timestamp.as_secs(),
+            timestamp_subsec_us / 1_000,
+            timestamp_subsec_us modulo 1_000,
+            $($arg)*
+        ));
+    })
+}

diff -uNr 06_uart_chainloader/src/time.rs 07_timestamps/src/time.rs
--- 06_uart_chainloader/src/time.rs
+++ 07_timestamps/src/time.rs
@@ -0,0 +1,37 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2020-2021 Andre Richter <andre.o.richter@gmail.com>
+
+//! Timer primitives.
+
+#[cfg(target_arch = "aarch64")]
+#[path = "_arch/aarch64/time.rs"]
+mod arch_time;
+
+//--------------------------------------------------------------------------------------------------
+// Architectural Public Reexports
+//--------------------------------------------------------------------------------------------------
+pub use arch_time::time_manager;
+
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
+/// Timekeeping interfaces.
+pub mod interface {
+    use core::time::Duration;
+
+    /// Time management functions.
+    pub trait TimeManager {
+        /// The timer's resolution.
+        fn resolution(&self) -> Duration;
+
+        /// The uptime since power-on of the device.
+        ///
+        /// This includes time consumed by firmware and bootloaders.
+        fn uptime(&self) -> Duration;
+
+        /// Spin for a given duration.
+        fn spin_for(&self, duration: Duration);
+    }
+}

diff -uNr 06_uart_chainloader/tests/boot_test_string.rb 07_timestamps/tests/boot_test_string.rb
--- 06_uart_chainloader/tests/boot_test_string.rb
+++ 07_timestamps/tests/boot_test_string.rb
@@ -0,0 +1,3 @@
+# frozen_string_literal: true
+
+EXPECTED_PRINT = 'Spinning for 1 second'

diff -uNr 06_uart_chainloader/tests/chainboot_test.rb 07_timestamps/tests/chainboot_test.rb
--- 06_uart_chainloader/tests/chainboot_test.rb
+++ 07_timestamps/tests/chainboot_test.rb
@@ -1,80 +0,0 @@
-# frozen_string_literal: true
-
-# SPDX-License-Identifier: MIT OR Apache-2.0
-#
-# Copyright (c) 2020-2021 Andre Richter <andre.o.richter@gmail.com>
-
-require_relative '../../common/serial/minipush'
-require_relative '../../common/tests/boot_test'
-require 'pty'
-
-# Match for the last print that 'demo_payload_rpiX.img' produces.
-EXPECTED_PRINT = 'Echoing input now'
-
-# Extend BootTest so that it listens on the output of a MiniPush instance, which is itself connected
-# to a QEMU instance instead of a real HW.
-class ChainbootTest < BootTest
-    MINIPUSH = '../common/serial/minipush.rb'
-    MINIPUSH_POWER_TARGET_REQUEST = 'Please power the target now'
-
-    def initialize(qemu_cmd, payload_path)
-        super(qemu_cmd, EXPECTED_PRINT)
-
-        @test_name = 'Boot test using Minipush'
-
-        @payload_path = payload_path
-    end
-
-    private
-
-    # override
-    def post_process_and_add_output(output)
-        temp = output.join.split("\r\n")
-
-        # Should a line have solo carriage returns, remove any overridden parts of the string.
-        temp.map! { |x| x.gsub(/.*\r/, '') }
-
-        @test_output += temp
-    end
-
-    def wait_for_minipush_power_request(mp_out)
-        output = []
-        Timeout.timeout(MAX_WAIT_SECS) do
-            loop do
-                output << mp_out.gets
-                break if output.last.include?(MINIPUSH_POWER_TARGET_REQUEST)
-            end
-        end
-    rescue Timeout::Error
-        @test_error = 'Timed out waiting for power request'
-    rescue StandardError => e
-        @test_error = e.message
-    ensure
-        post_process_and_add_output(output)
-    end
-
-    # override
-    def setup
-        pty_main, pty_secondary = PTY.open
-        mp_out, _mp_in = PTY.spawn("ruby #{MINIPUSH} #{pty_secondary.path} #{@payload_path}")
-
-        # Wait until MiniPush asks for powering the target.
-        wait_for_minipush_power_request(mp_out)
-
-        # Now is the time to start QEMU with the chainloader binary. QEMU's virtual tty is connected
-        # to the MiniPush instance spawned above, so that the two processes talk to each other.
-        Process.spawn(@qemu_cmd, in: pty_main, out: pty_main)
-
-        # The remainder of the test is done by the parent class' run_concrete_test, which listens on
-        # @qemu_serial. Hence, point it to MiniPush's output.
-        @qemu_serial = mp_out
-    end
-end
-
-##--------------------------------------------------------------------------------------------------
-## Execution starts here
-##--------------------------------------------------------------------------------------------------
-payload_path = ARGV.pop
-qemu_cmd = ARGV.join(' ')
-
-ChainbootTest.new(qemu_cmd, payload_path).run

diff -uNr 06_uart_chainloader/update.sh 07_timestamps/update.sh
--- 06_uart_chainloader/update.sh
+++ 07_timestamps/update.sh
@@ -1,8 +0,0 @@
-#!/usr/bin/env bash
-
-cd ../05_drivers_gpio_uart
-BSP=rpi4 make
-cp kernel8.img ../06_uart_chainloader/demo_payload_rpi4.img
-make
-cp kernel8.img ../06_uart_chainloader/demo_payload_rpi3.img
-rm kernel8.img

```
