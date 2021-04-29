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
@@ -25,7 +25,6 @@
     READELF_BINARY    = aarch64-none-elf-readelf
     LINKER_FILE       = src/bsp/raspberrypi/link.ld
     RUSTC_MISC_ARGS   = -C target-cpu=cortex-a53
-    CHAINBOOT_DEMO_PAYLOAD = demo_payload_rpi3.img
 else ifeq ($(BSP),rpi4)
     TARGET            = aarch64-unknown-none-softfloat
     KERNEL_BIN        = kernel8.img
@@ -37,7 +36,6 @@
     READELF_BINARY    = aarch64-none-elf-readelf
     LINKER_FILE       = src/bsp/raspberrypi/link.ld
     RUSTC_MISC_ARGS   = -C target-cpu=cortex-a72
-    CHAINBOOT_DEMO_PAYLOAD = demo_payload_rpi4.img
 endif

 # Export for build.rs
@@ -70,7 +68,6 @@
 DOCKER_ARG_DEV       = --privileged -v /dev:/dev

 DOCKER_QEMU  = $(DOCKER_CMD_INTERACT) $(DOCKER_IMAGE)
-DOCKER_TEST  = $(DOCKER_CMD) -t $(DOCKER_ARG_DIR_UTILS) $(DOCKER_IMAGE)
 DOCKER_TOOLS = $(DOCKER_CMD) $(DOCKER_IMAGE)

 # Dockerize commands that require USB device passthrough only on Linux
@@ -80,12 +77,10 @@
     DOCKER_CHAINBOOT = $(DOCKER_CMD_DEV) $(DOCKER_ARG_DIR_UTILS) $(DOCKER_IMAGE)
 endif

-EXEC_QEMU          = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)
-EXEC_MINIPUSH      = ruby ../utils/minipush.rb
-EXEC_QEMU_MINIPUSH = ruby tests/qemu_minipush.rb
+EXEC_QEMU     = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)
+EXEC_MINIPUSH = ruby ../utils/minipush.rb

-.PHONY: all $(KERNEL_ELF) $(KERNEL_BIN) doc qemu qemuasm chainboot clippy clean readelf objdump nm \
-    check
+.PHONY: all $(KERNEL_ELF) $(KERNEL_BIN) doc qemu chainboot clippy clean readelf objdump nm check

 all: $(KERNEL_BIN)

@@ -101,26 +96,16 @@
 	@$(DOC_CMD) --document-private-items --open

 ifeq ($(QEMU_MACHINE_TYPE),)
-qemu test:
+qemu:
 	$(call colorecho, "\n$(QEMU_MISSING_STRING)")
 else
 qemu: $(KERNEL_BIN)
 	$(call colorecho, "\nLaunching QEMU")
 	@$(DOCKER_QEMU) $(EXEC_QEMU) $(QEMU_RELEASE_ARGS) -kernel $(KERNEL_BIN)
-
-qemuasm: $(KERNEL_BIN)
-	$(call colorecho, "\nLaunching QEMU with ASM output")
-	@$(DOCKER_QEMU) $(EXEC_QEMU) $(QEMU_RELEASE_ARGS) -kernel $(KERNEL_BIN) -d in_asm
-
-test: $(KERNEL_BIN)
-	$(call colorecho, "\nTesting chainloading - $(BSP)")
-	@$(DOCKER_TEST) $(EXEC_QEMU_MINIPUSH) $(EXEC_QEMU) $(QEMU_RELEASE_ARGS) \
-                -kernel $(KERNEL_BIN) $(CHAINBOOT_DEMO_PAYLOAD)
-
 endif

-chainboot:
-	@$(DOCKER_CHAINBOOT) $(EXEC_MINIPUSH) $(DEV_SERIAL) $(CHAINBOOT_DEMO_PAYLOAD)
+chainboot: $(KERNEL_BIN)
+	@$(DOCKER_CHAINBOOT) $(EXEC_MINIPUSH) $(DEV_SERIAL) $(KERNEL_BIN)

 clippy:
 	@RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(CLIPPY_CMD)

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
@@ -45,31 +34,20 @@
 	and	x1, x1, _core_id_mask
 	ldr	x2, BOOT_CORE_ID      // provided by bsp/__board_name__/cpu.rs
 	cmp	x1, x2
-	b.ne	2f
-
-	// If execution reaches here, it is the boot core.
+	b.ne	1f

-	// Next, relocate the binary.
-	ADR_REL	x0, __binary_nonzero_start         // The address the binary got loaded to.
-	ADR_ABS	x1, __binary_nonzero_start         // The address the binary was linked to.
-	ADR_ABS	x2, __binary_nonzero_end_exclusive
-
-1:	ldr	x3, [x0], #8
-	str	x3, [x1], #8
-	cmp	x1, x2
-	b.lo	1b
+	// If execution reaches here, it is the boot core. Now, prepare the jump to Rust code.

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
-2:	wfe
-	b	2b
+1:	wfe
+	b	1b

 .size	_start, . - _start
 .type	_start, function

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
@@ -0,0 +1,118 @@
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
+use cortex_a::{barrier, regs::*};
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
@@ -139,25 +139,19 @@
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
@@ -279,7 +279,7 @@
     }

     /// Retrieve a character.
-    fn read_char(&mut self, blocking_mode: BlockingMode) -> Option<char> {
+    fn read_char_converting(&mut self, blocking_mode: BlockingMode) -> Option<char> {
         // If RX FIFO is empty,
         if self.registers.FR.matches_all(FR::RXFE::SET) {
             // immediately return in non-blocking mode.
@@ -294,7 +294,12 @@
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
@@ -374,14 +379,14 @@
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
@@ -44,12 +42,8 @@
     ***********************************************************************************************/
     .data : { *(.data*) } :segment_rw

-    /* Fill up to 8 byte, b/c relocating the binary is done in u64 chunks */
-    . = ALIGN(8);
-    __binary_nonzero_end_exclusive = .;
-
     /* Section is zeroed in u64 chunks, align start and end to 8 bytes */
-    .bss :
+    .bss : ALIGN(8)
     {
         __bss_start = .;
         *(.bss*);

diff -uNr 06_uart_chainloader/src/bsp/raspberrypi/memory.rs 07_timestamps/src/bsp/raspberrypi/memory.rs
--- 06_uart_chainloader/src/bsp/raspberrypi/memory.rs
+++ 07_timestamps/src/bsp/raspberrypi/memory.rs
@@ -23,10 +23,9 @@
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
@@ -53,13 +52,7 @@
 // Public Code
 //--------------------------------------------------------------------------------------------------

-/// The address on which the Raspberry firmware loads every binary by default.
-#[inline(always)]
-pub fn board_default_load_addr() -> *const u64 {
-    map::BOARD_DEFAULT_LOAD_ADDRESS as _
-}
-
-/// Return the inclusive range spanning the relocated .bss section.
+/// Return the inclusive range spanning the .bss section.
 ///
 /// # Safety
 ///

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
@@ -107,7 +107,6 @@
 //! [`runtime_init::runtime_init()`]: runtime_init/fn.runtime_init.html

 #![allow(clippy::upper_case_acronyms)]
-#![feature(asm)]
 #![feature(const_fn_fn_ptr_basics)]
 #![feature(format_args_nl)]
 #![feature(global_asm)]
@@ -125,6 +124,7 @@
 mod print;
 mod runtime_init;
 mod synchronization;
+mod time;

 /// Early init code.
 ///
@@ -147,56 +147,38 @@
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

diff -uNr 06_uart_chainloader/tests/qemu_minipush.rb 07_timestamps/tests/qemu_minipush.rb
--- 06_uart_chainloader/tests/qemu_minipush.rb
+++ 07_timestamps/tests/qemu_minipush.rb
@@ -1,80 +0,0 @@
-# frozen_string_literal: true
-
-# SPDX-License-Identifier: MIT OR Apache-2.0
-#
-# Copyright (c) 2020-2021 Andre Richter <andre.o.richter@gmail.com>
-
-require_relative '../../utils/minipush'
-require 'expect'
-require 'timeout'
-
-# Match for the last print that 'demo_payload_rpiX.img' produces.
-EXPECTED_PRINT = 'Echoing input now'
-
-# The main class
-class QEMUMiniPush < MiniPush
-    TIMEOUT_SECS = 3
-
-    # override
-    def initialize(qemu_cmd, binary_image_path)
-        super(nil, binary_image_path)
-
-        @qemu_cmd = qemu_cmd
-    end
-
-    private
-
-    def quit_qemu_graceful
-        Timeout.timeout(5) do
-            pid = @target_serial.pid
-            Process.kill('TERM', pid)
-            Process.wait(pid)
-        end
-    end
-
-    # override
-    def open_serial
-        @target_serial = IO.popen(@qemu_cmd, 'r+', err: '/dev/null')
-
-        # Ensure all output is immediately flushed to the device.
-        @target_serial.sync = true
-
-        puts "[#{@name_short}] ✅ Serial connected"
-    end
-
-    # override
-    def terminal
-        result = @target_serial.expect(EXPECTED_PRINT, TIMEOUT_SECS)
-        exit(1) if result.nil?
-
-        puts result
-
-        quit_qemu_graceful
-    end
-
-    # override
-    def connetion_reset; end
-
-    # override
-    def handle_reconnect(error)
-        handle_unexpected(error)
-    end
-end
-
-##--------------------------------------------------------------------------------------------------
-## Execution starts here
-##--------------------------------------------------------------------------------------------------
-puts
-puts 'QEMUMiniPush 1.0'.cyan
-puts
-
-# CTRL + C handler. Only here to suppress Ruby's default exception print.
-trap('INT') do
-    # The `ensure` block from `QEMUMiniPush::run` will run after exit, restoring console state.
-    exit
-end
-
-binary_image_path = ARGV.pop
-qemu_cmd = ARGV.join(' ')
-
-QEMUMiniPush.new(qemu_cmd, binary_image_path).run

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
