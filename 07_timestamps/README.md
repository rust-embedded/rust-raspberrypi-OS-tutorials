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

[    0.143123] mingo version 0.7.0
[    0.143323] Booting on: Raspberry Pi 3
[    0.143778] Architectural timer resolution: 52 ns
[    0.144352] Drivers loaded:
[    0.144688]       1. BCM PL011 UART
[    0.145110]       2. BCM GPIO
[W   0.145469] Spin duration smaller than architecturally supported, skipping
[    0.146313] Spinning for 1 second
[    1.146715] Spinning for 1 second
[    2.146938] Spinning for 1 second
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
 edition = "2021"

Binary files 06_uart_chainloader/demo_payload_rpi3.img and 07_timestamps/demo_payload_rpi3.img differ
Binary files 06_uart_chainloader/demo_payload_rpi4.img and 07_timestamps/demo_payload_rpi4.img differ

diff -uNr 06_uart_chainloader/Makefile 07_timestamps/Makefile
--- 06_uart_chainloader/Makefile
+++ 07_timestamps/Makefile
@@ -24,29 +24,27 @@
 QEMU_MISSING_STRING = "This board is not yet supported for QEMU."

 ifeq ($(BSP),rpi3)
-    TARGET                 = aarch64-unknown-none-softfloat
-    KERNEL_BIN             = kernel8.img
-    QEMU_BINARY            = qemu-system-aarch64
-    QEMU_MACHINE_TYPE      = raspi3
-    QEMU_RELEASE_ARGS      = -serial stdio -display none
-    OBJDUMP_BINARY         = aarch64-none-elf-objdump
-    NM_BINARY              = aarch64-none-elf-nm
-    READELF_BINARY         = aarch64-none-elf-readelf
-    LD_SCRIPT_PATH         = $(shell pwd)/src/bsp/raspberrypi
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
+    LD_SCRIPT_PATH    = $(shell pwd)/src/bsp/raspberrypi
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
-    LD_SCRIPT_PATH         = $(shell pwd)/src/bsp/raspberrypi
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
+    LD_SCRIPT_PATH    = $(shell pwd)/src/bsp/raspberrypi
+    RUSTC_MISC_ARGS   = -C target-cpu=cortex-a72
 endif

 # Export for build.rs.
@@ -92,7 +90,7 @@
     -O binary

 EXEC_QEMU          = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)
-EXEC_TEST_MINIPUSH = ruby tests/chainboot_test.rb
+EXEC_TEST_DISPATCH = ruby ../common/tests/dispatch.rb
 EXEC_MINIPUSH      = ruby ../common/serial/minipush.rb

 ##------------------------------------------------------------------------------
@@ -162,7 +160,7 @@
 ##------------------------------------------------------------------------------
 ifeq ($(QEMU_MACHINE_TYPE),) # QEMU is not supported for the board.

-qemu qemuasm:
+qemu:
 	$(call color_header, "$(QEMU_MISSING_STRING)")

 else # QEMU is supported.
@@ -171,17 +169,13 @@
 	$(call color_header, "Launching QEMU")
 	@$(DOCKER_QEMU) $(EXEC_QEMU) $(QEMU_RELEASE_ARGS) -kernel $(KERNEL_BIN)

-qemuasm: $(KERNEL_BIN)
-	$(call color_header, "Launching QEMU with ASM output")
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
@@ -238,8 +232,7 @@
 ##------------------------------------------------------------------------------
 test_boot: $(KERNEL_BIN)
 	$(call color_header, "Boot test - $(BSP)")
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
 //--------------------------------------------------------------------------------------------------
 // Public Code
 //--------------------------------------------------------------------------------------------------
@@ -48,35 +37,31 @@
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
+	// Read the CPU's timer counter frequency and store it in ARCH_TIMER_COUNTER_FREQUENCY.
+	// Abort if the frequency read back as 0.
+	ADR_REL	x1, ARCH_TIMER_COUNTER_FREQUENCY // provided by aarch64/time.rs
+	mrs	x2, CNTFRQ_EL0
+	cmp	x2, xzr
+	b.eq	.L_parking_loop
+	str	w2, [x1]
+
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
@@ -0,0 +1,162 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
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
+use crate::warn;
+use aarch64_cpu::{asm::barrier, registers::*};
+use core::{
+    num::{NonZeroU128, NonZeroU32, NonZeroU64},
+    ops::{Add, Div},
+    time::Duration,
+};
+use tock_registers::interfaces::Readable;
+
+//--------------------------------------------------------------------------------------------------
+// Private Definitions
+//--------------------------------------------------------------------------------------------------
+
+const NANOSEC_PER_SEC: NonZeroU64 = NonZeroU64::new(1_000_000_000).unwrap();
+
+#[derive(Copy, Clone, PartialOrd, PartialEq)]
+struct GenericTimerCounterValue(u64);
+
+//--------------------------------------------------------------------------------------------------
+// Global instances
+//--------------------------------------------------------------------------------------------------
+
+/// Boot assembly code overwrites this value with the value of CNTFRQ_EL0 before any Rust code is
+/// executed. This given value here is just a (safe) dummy.
+#[no_mangle]
+static ARCH_TIMER_COUNTER_FREQUENCY: NonZeroU32 = NonZeroU32::MIN;
+
+//--------------------------------------------------------------------------------------------------
+// Private Code
+//--------------------------------------------------------------------------------------------------
+
+fn arch_timer_counter_frequency() -> NonZeroU32 {
+    // Read volatile is needed here to prevent the compiler from optimizing
+    // ARCH_TIMER_COUNTER_FREQUENCY away.
+    //
+    // This is safe, because all the safety requirements as stated in read_volatile()'s
+    // documentation are fulfilled.
+    unsafe { core::ptr::read_volatile(&ARCH_TIMER_COUNTER_FREQUENCY) }
+}
+
+impl GenericTimerCounterValue {
+    pub const MAX: Self = GenericTimerCounterValue(u64::MAX);
+}
+
+impl Add for GenericTimerCounterValue {
+    type Output = Self;
+
+    fn add(self, other: Self) -> Self {
+        GenericTimerCounterValue(self.0.wrapping_add(other.0))
+    }
+}
+
+impl From<GenericTimerCounterValue> for Duration {
+    fn from(counter_value: GenericTimerCounterValue) -> Self {
+        if counter_value.0 == 0 {
+            return Duration::ZERO;
+        }
+
+        let frequency: NonZeroU64 = arch_timer_counter_frequency().into();
+
+        // Div<NonZeroU64> implementation for u64 cannot panic.
+        let secs = counter_value.0.div(frequency);
+
+        // This is safe, because frequency can never be greater than u32::MAX, which means the
+        // largest theoretical value for sub_second_counter_value is (u32::MAX - 1). Therefore,
+        // (sub_second_counter_value * NANOSEC_PER_SEC) cannot overflow an u64.
+        //
+        // The subsequent division ensures the result fits into u32, since the max result is smaller
+        // than NANOSEC_PER_SEC. Therefore, just cast it to u32 using `as`.
+        let sub_second_counter_value = counter_value.0 modulo frequency;
+        let nanos = unsafe { sub_second_counter_value.unchecked_mul(u64::from(NANOSEC_PER_SEC)) }
+            .div(frequency) as u32;
+
+        Duration::new(secs, nanos)
+    }
+}
+
+fn max_duration() -> Duration {
+    Duration::from(GenericTimerCounterValue::MAX)
+}
+
+impl TryFrom<Duration> for GenericTimerCounterValue {
+    type Error = &'static str;
+
+    fn try_from(duration: Duration) -> Result<Self, Self::Error> {
+        if duration < resolution() {
+            return Ok(GenericTimerCounterValue(0));
+        }
+
+        if duration > max_duration() {
+            return Err("Conversion error. Duration too big");
+        }
+
+        let frequency: u128 = u32::from(arch_timer_counter_frequency()) as u128;
+        let duration: u128 = duration.as_nanos();
+
+        // This is safe, because frequency can never be greater than u32::MAX, and
+        // (Duration::MAX.as_nanos() * u32::MAX) < u128::MAX.
+        let counter_value =
+            unsafe { duration.unchecked_mul(frequency) }.div(NonZeroU128::from(NANOSEC_PER_SEC));
+
+        // Since we checked above that we are <= max_duration(), just cast to u64.
+        Ok(GenericTimerCounterValue(counter_value as u64))
+    }
+}
+
+#[inline(always)]
+fn read_cntpct() -> GenericTimerCounterValue {
+    // Prevent that the counter is read ahead of time due to out-of-order execution.
+    barrier::isb(barrier::SY);
+    let cnt = CNTPCT_EL0.get();
+
+    GenericTimerCounterValue(cnt)
+}
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+/// The timer's resolution.
+pub fn resolution() -> Duration {
+    Duration::from(GenericTimerCounterValue(1))
+}
+
+/// The uptime since power-on of the device.
+///
+/// This includes time consumed by firmware and bootloaders.
+pub fn uptime() -> Duration {
+    read_cntpct().into()
+}
+
+/// Spin for a given duration.
+pub fn spin_for(duration: Duration) {
+    let curr_counter_value = read_cntpct();
+
+    let counter_value_delta: GenericTimerCounterValue = match duration.try_into() {
+        Err(msg) => {
+            warn!("spin_for: {}. Skipping", msg);
+            return;
+        }
+        Ok(val) => val,
+    };
+    let counter_value_target = curr_counter_value + counter_value_delta;
+
+    // Busy wait.
+    //
+    // Read CNTPCT_EL0 directly to avoid the ISB that is part of [`read_cntpct`].
+    while GenericTimerCounterValue(CNTPCT_EL0.get()) < counter_value_target {}
+}

diff -uNr 06_uart_chainloader/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs 07_timestamps/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs
--- 06_uart_chainloader/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs
+++ 07_timestamps/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs
@@ -140,25 +140,19 @@
     /// Disable pull-up/down on pins 14 and 15.
     #[cfg(feature = "bsp_rpi3")]
     fn disable_pud_14_15_bcm2837(&mut self) {
-        use crate::cpu;
+        use crate::time;
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
@@ -275,7 +275,7 @@
     }

     /// Retrieve a character.
-    fn read_char(&mut self, blocking_mode: BlockingMode) -> Option<char> {
+    fn read_char_converting(&mut self, blocking_mode: BlockingMode) -> Option<char> {
         // If RX FIFO is empty,
         if self.registers.FR.matches_all(FR::RXFE::SET) {
             // immediately return in non-blocking mode.
@@ -290,7 +290,12 @@
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
@@ -376,14 +381,14 @@
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

diff -uNr 06_uart_chainloader/src/bsp/raspberrypi/driver.rs 07_timestamps/src/bsp/raspberrypi/driver.rs
--- 06_uart_chainloader/src/bsp/raspberrypi/driver.rs
+++ 07_timestamps/src/bsp/raspberrypi/driver.rs
@@ -57,6 +57,17 @@
 /// # Safety
 ///
 /// See child function calls.
+///
+/// # Note
+///
+/// Using atomics here relieves us from needing to use `unsafe` for the static variable.
+///
+/// On `AArch64`, which is the only implemented architecture at the time of writing this,
+/// [`AtomicBool::load`] and [`AtomicBool::store`] are lowered to ordinary load and store
+/// instructions. They are therefore safe to use even with MMU + caching deactivated.
+///
+/// [`AtomicBool::load`]: core::sync::atomic::AtomicBool::load
+/// [`AtomicBool::store`]: core::sync::atomic::AtomicBool::store
 pub unsafe fn init() -> Result<(), &'static str> {
     static INIT_DONE: AtomicBool = AtomicBool::new(false);
     if INIT_DONE.load(Ordering::Relaxed) {

diff -uNr 06_uart_chainloader/src/bsp/raspberrypi/kernel.ld 07_timestamps/src/bsp/raspberrypi/kernel.ld
--- 06_uart_chainloader/src/bsp/raspberrypi/kernel.ld
+++ 07_timestamps/src/bsp/raspberrypi/kernel.ld
@@ -3,6 +3,8 @@
  * Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
  */

+__rpi_phys_dram_start_addr = 0;
+
 /* The physical address at which the the kernel binary will be loaded by the Raspberry's firmware */
 __rpi_phys_binary_load_addr = 0x80000;

@@ -26,8 +28,7 @@

 SECTIONS
 {
-    /* Set the link address to 32 MiB */
-    . = 0x2000000;
+    . =  __rpi_phys_dram_start_addr;

     /***********************************************************************************************
     * Boot Core Stack
@@ -44,7 +45,6 @@
     /***********************************************************************************************
     * Code + RO Data + Global Offset Table
     ***********************************************************************************************/
-    __binary_nonzero_start = .;
     .text :
     {
         KEEP(*(.text._start))
@@ -60,10 +60,6 @@
     ***********************************************************************************************/
     .data : { *(.data*) } :segment_data

-    /* Fill up to 8 byte, b/c relocating the binary is done in u64 chunks */
-    . = ALIGN(8);
-    __binary_nonzero_end_exclusive = .;
-
     /* Section is zeroed in pairs of u64. Align start and end to 16 bytes */
     .bss (NOLOAD) : ALIGN(16)
     {

diff -uNr 06_uart_chainloader/src/bsp/raspberrypi/memory.rs 07_timestamps/src/bsp/raspberrypi/memory.rs
--- 06_uart_chainloader/src/bsp/raspberrypi/memory.rs
+++ 07_timestamps/src/bsp/raspberrypi/memory.rs
@@ -11,7 +11,6 @@
 /// The board's physical memory map.
 #[rustfmt::skip]
 pub(super) mod map {
-    pub const BOARD_DEFAULT_LOAD_ADDRESS: usize =        0x8_0000;

     pub const GPIO_OFFSET:         usize = 0x0020_0000;
     pub const UART_OFFSET:         usize = 0x0020_1000;
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

diff -uNr 06_uart_chainloader/src/driver.rs 07_timestamps/src/driver.rs
--- 06_uart_chainloader/src/driver.rs
+++ 07_timestamps/src/driver.rs
@@ -4,7 +4,10 @@

 //! Driver support.

-use crate::synchronization::{interface::Mutex, NullLock};
+use crate::{
+    info,
+    synchronization::{interface::Mutex, NullLock},
+};

 //--------------------------------------------------------------------------------------------------
 // Private Definitions
@@ -151,4 +154,14 @@
             }
         });
     }
+
+    /// Enumerate all registered device drivers.
+    pub fn enumerate(&self) {
+        let mut i: usize = 1;
+        self.for_each_descriptor(|descriptor| {
+            info!("      {}. {}", i, descriptor.device_driver.compatible());
+
+            i += 1;
+        });
+    }
 }

diff -uNr 06_uart_chainloader/src/main.rs 07_timestamps/src/main.rs
--- 06_uart_chainloader/src/main.rs
+++ 07_timestamps/src/main.rs
@@ -108,9 +108,12 @@

 #![allow(clippy::upper_case_acronyms)]
 #![feature(asm_const)]
+#![feature(const_option)]
 #![feature(format_args_nl)]
+#![feature(nonzero_min_max)]
 #![feature(panic_info_message)]
 #![feature(trait_alias)]
+#![feature(unchecked_math)]
 #![no_main]
 #![no_std]

@@ -121,6 +124,7 @@
 mod panic_wait;
 mod print;
 mod synchronization;
+mod time;

 /// Early init code.
 ///
@@ -142,55 +146,30 @@
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
-    use console::console;
-
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
-    }
+    use core::time::Duration;

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
+    driver::driver_manager().enumerate();
+
+    // Test a failing timer case.
+    time::time_manager().spin_for(Duration::from_nanos(1));
+
+    loop {
+        info!("Spinning for 1 second");
+        time::time_manager().spin_for(Duration::from_secs(1));
     }
-
-    println!("[ML] Loaded! Executing the payload now\n");
-    console().flush();
-
-    // Use black magic to create a function pointer.
-    let kernel: fn() -> ! = unsafe { core::mem::transmute(kernel_addr) };
-
-    // Jump to loaded kernel!
-    kernel()
 }

diff -uNr 06_uart_chainloader/src/panic_wait.rs 07_timestamps/src/panic_wait.rs
--- 06_uart_chainloader/src/panic_wait.rs
+++ 07_timestamps/src/panic_wait.rs
@@ -45,15 +45,18 @@
     // Protect against panic infinite loops if any of the following code panics itself.
     panic_prevent_reenter();

+    let timestamp = crate::time::time_manager().uptime();
     let (location, line, column) = match info.location() {
         Some(loc) => (loc.file(), loc.line(), loc.column()),
         _ => ("???", 0, 0),
     };

     println!(
-        "Kernel panic!\n\n\
+        "[  {:>3}.{:06}] Kernel panic!\n\n\
         Panic location:\n      File '{}', line {}, column {}\n\n\
         {}",
+        timestamp.as_secs(),
+        timestamp.subsec_micros(),
         location,
         line,
         column,

diff -uNr 06_uart_chainloader/src/print.rs 07_timestamps/src/print.rs
--- 06_uart_chainloader/src/print.rs
+++ 07_timestamps/src/print.rs
@@ -34,3 +34,51 @@
         $crate::print::_print(format_args_nl!($($arg)*));
     })
 }
+
+/// Prints an info, with a newline.
+#[macro_export]
+macro_rules! info {
+    ($string:expr) => ({
+        let timestamp = $crate::time::time_manager().uptime();
+
+        $crate::print::_print(format_args_nl!(
+            concat!("[  {:>3}.{:06}] ", $string),
+            timestamp.as_secs(),
+            timestamp.subsec_micros(),
+        ));
+    });
+    ($format_string:expr, $($arg:tt)*) => ({
+        let timestamp = $crate::time::time_manager().uptime();
+
+        $crate::print::_print(format_args_nl!(
+            concat!("[  {:>3}.{:06}] ", $format_string),
+            timestamp.as_secs(),
+            timestamp.subsec_micros(),
+            $($arg)*
+        ));
+    })
+}
+
+/// Prints a warning, with a newline.
+#[macro_export]
+macro_rules! warn {
+    ($string:expr) => ({
+        let timestamp = $crate::time::time_manager().uptime();
+
+        $crate::print::_print(format_args_nl!(
+            concat!("[W {:>3}.{:06}] ", $string),
+            timestamp.as_secs(),
+            timestamp.subsec_micros(),
+        ));
+    });
+    ($format_string:expr, $($arg:tt)*) => ({
+        let timestamp = $crate::time::time_manager().uptime();
+
+        $crate::print::_print(format_args_nl!(
+            concat!("[W {:>3}.{:06}] ", $format_string),
+            timestamp.as_secs(),
+            timestamp.subsec_micros(),
+            $($arg)*
+        ));
+    })
+}

diff -uNr 06_uart_chainloader/src/time.rs 07_timestamps/src/time.rs
--- 06_uart_chainloader/src/time.rs
+++ 07_timestamps/src/time.rs
@@ -0,0 +1,57 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2020-2023 Andre Richter <andre.o.richter@gmail.com>
+
+//! Timer primitives.
+
+#[cfg(target_arch = "aarch64")]
+#[path = "_arch/aarch64/time.rs"]
+mod arch_time;
+
+use core::time::Duration;
+
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
+/// Provides time management functions.
+pub struct TimeManager;
+
+//--------------------------------------------------------------------------------------------------
+// Global instances
+//--------------------------------------------------------------------------------------------------
+
+static TIME_MANAGER: TimeManager = TimeManager::new();
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+/// Return a reference to the global TimeManager.
+pub fn time_manager() -> &'static TimeManager {
+    &TIME_MANAGER
+}
+
+impl TimeManager {
+    /// Create an instance.
+    pub const fn new() -> Self {
+        Self
+    }
+
+    /// The timer's resolution.
+    pub fn resolution(&self) -> Duration {
+        arch_time::resolution()
+    }
+
+    /// The uptime since power-on of the device.
+    ///
+    /// This includes time consumed by firmware and bootloaders.
+    pub fn uptime(&self) -> Duration {
+        arch_time::uptime()
+    }
+
+    /// Spin for a given duration.
+    pub fn spin_for(&self, duration: Duration) {
+        arch_time::spin_for(duration)
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
@@ -1,78 +0,0 @@
-# frozen_string_literal: true
-
-# SPDX-License-Identifier: MIT OR Apache-2.0
-#
-# Copyright (c) 2020-2023 Andre Richter <andre.o.richter@gmail.com>
-
-require_relative '../../common/serial/minipush'
-require_relative '../../common/tests/boot_test'
-require 'pty'
-
-# Match for the last print that 'demo_payload_rpiX.img' produces.
-EXPECTED_PRINT = 'Echoing input now'
-
-# Wait for request to power the target.
-class PowerTargetRequestTest < SubtestBase
-    MINIPUSH_POWER_TARGET_REQUEST = 'Please power the target now'
-
-    def initialize(qemu_cmd, pty_main)
-        super()
-        @qemu_cmd = qemu_cmd
-        @pty_main = pty_main
-    end
-
-    def name
-        'Waiting for request to power target'
-    end
-
-    def run(qemu_out, _qemu_in)
-        expect_or_raise(qemu_out, MINIPUSH_POWER_TARGET_REQUEST)
-
-        # Now is the time to start QEMU with the chainloader binary. QEMU's virtual tty connects to
-        # the MiniPush instance spawned on pty_main, so that the two processes talk to each other.
-        Process.spawn(@qemu_cmd, in: @pty_main, out: @pty_main, err: '/dev/null')
-    end
-end
-
-# Extend BootTest so that it listens on the output of a MiniPush instance, which is itself connected
-# to a QEMU instance instead of a real HW.
-class ChainbootTest < BootTest
-    MINIPUSH = '../common/serial/minipush.rb'
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
-    def setup
-        pty_main, pty_secondary = PTY.open
-        mp_out, _mp_in = PTY.spawn("ruby #{MINIPUSH} #{pty_secondary.path} #{@payload_path}")
-
-        # The subtests (from this class and the parents) listen on @qemu_out_wrapped. Hence, point
-        # it to MiniPush's output.
-        @qemu_out_wrapped = PTYLoggerWrapper.new(mp_out, "\r\n")
-
-        # Important: Run this subtest before the one in the parent class.
-        @console_subtests.prepend(PowerTargetRequestTest.new(@qemu_cmd, pty_main))
-    end
-
-    # override
-    def finish
-        super()
-        @test_output.map! { |x| x.gsub(/.*\r/, '  ') }
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
