# Tutorial 08 - Timestamps

## tl;dr

- We add abstractions for the architectural timer, implement it for `aarch64` and use it to annotate
  prints with timestamps.
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
[MP] ⏩ Pushing 11 KiB =========================================🦀 100% 0 KiB/s Time: 00:00:00
[ML] Loaded! Executing the payload now

[    0.543941] Booting on: Raspberry Pi 3
[    0.545059] Architectural timer resolution: 52 ns
[    0.547358] Drivers loaded:
[    0.548703]       1. BCM GPIO
[    0.550135]       2. BCM PL011 UART
[W   0.551828] Spin duration smaller than architecturally supported, skipping
[    0.555212] Spinning for 1 second
[    1.556818] Spinning for 1 second
[    2.557690] Spinning for 1 second
```

## Diff to previous
```diff
Binary files 07_uart_chainloader/demo_payload_rpi3.img and 08_timestamps/demo_payload_rpi3.img differ
Binary files 07_uart_chainloader/demo_payload_rpi4.img and 08_timestamps/demo_payload_rpi4.img differ

diff -uNr 07_uart_chainloader/Makefile 08_timestamps/Makefile
--- 07_uart_chainloader/Makefile
+++ 08_timestamps/Makefile
@@ -21,8 +21,7 @@
     OBJDUMP_BINARY    = aarch64-none-elf-objdump
     NM_BINARY         = aarch64-none-elf-nm
     LINKER_FILE       = src/bsp/raspberrypi/link.ld
-    RUSTC_MISC_ARGS   = -C target-cpu=cortex-a53 -C relocation-model=pic
-    CHAINBOOT_DEMO_PAYLOAD = demo_payload_rpi3.img
+    RUSTC_MISC_ARGS   = -C target-cpu=cortex-a53
 else ifeq ($(BSP),rpi4)
     TARGET            = aarch64-unknown-none-softfloat
     KERNEL_BIN        = kernel8.img
@@ -32,8 +31,7 @@
     OBJDUMP_BINARY    = aarch64-none-elf-objdump
     NM_BINARY         = aarch64-none-elf-nm
     LINKER_FILE       = src/bsp/raspberrypi/link.ld
-    RUSTC_MISC_ARGS   = -C target-cpu=cortex-a72 -C relocation-model=pic
-    CHAINBOOT_DEMO_PAYLOAD = demo_payload_rpi4.img
+    RUSTC_MISC_ARGS   = -C target-cpu=cortex-a72
 endif

 # Export for build.rs
@@ -75,8 +73,7 @@
 EXEC_QEMU     = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)
 EXEC_MINIPUSH = ruby ../utils/minipush.rb

-.PHONY: all $(KERNEL_ELF) $(KERNEL_BIN) doc qemu qemuasm chainboot clippy clean readelf objdump nm \
-    check
+.PHONY: all $(KERNEL_ELF) $(KERNEL_BIN) doc qemu chainboot clippy clean readelf objdump nm check

 all: $(KERNEL_BIN)

@@ -90,18 +87,15 @@
 	$(DOC_CMD) --document-private-items --open

 ifeq ($(QEMU_MACHINE_TYPE),)
-qemu qemuasm:
+qemu:
 	@echo "This board is not yet supported for QEMU."
 else
 qemu: $(KERNEL_BIN)
 	@$(DOCKER_QEMU) $(EXEC_QEMU) $(QEMU_RELEASE_ARGS) -kernel $(KERNEL_BIN)
-
-qemuasm: $(KERNEL_BIN)
-	@$(DOCKER_QEMU) $(EXEC_QEMU) $(QEMU_RELEASE_ARGS) -kernel $(KERNEL_BIN) -d in_asm
 endif

-chainboot:
-	@$(DOCKER_CHAINBOOT) $(EXEC_MINIPUSH) $(DEV_SERIAL) $(CHAINBOOT_DEMO_PAYLOAD)
+chainboot: $(KERNEL_BIN)
+	@$(DOCKER_CHAINBOOT) $(EXEC_MINIPUSH) $(DEV_SERIAL) $(KERNEL_BIN)

 clippy:
 	RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(CLIPPY_CMD)
@@ -113,10 +107,7 @@
 	readelf --headers $(KERNEL_ELF)

 objdump: $(KERNEL_ELF)
-	@$(DOCKER_ELFTOOLS) $(OBJDUMP_BINARY) --disassemble --demangle \
-                --section .text \
-                --section .got  \
-                $(KERNEL_ELF) | rustfilt
+	@$(DOCKER_ELFTOOLS) $(OBJDUMP_BINARY) --disassemble --demangle $(KERNEL_ELF) | rustfilt

 nm: $(KERNEL_ELF)
 	@$(DOCKER_ELFTOOLS) $(NM_BINARY) --demangle --print-size $(KERNEL_ELF) | sort | rustfilt

diff -uNr 07_uart_chainloader/src/_arch/aarch64/cpu.rs 08_timestamps/src/_arch/aarch64/cpu.rs
--- 07_uart_chainloader/src/_arch/aarch64/cpu.rs
+++ 08_timestamps/src/_arch/aarch64/cpu.rs
@@ -22,11 +22,11 @@
 ///   actually set (`SP.set()`).
 #[no_mangle]
 pub unsafe fn _start() -> ! {
-    use crate::relocate;
+    use crate::runtime_init;

     if bsp::cpu::BOOT_CORE_ID == cpu::smp::core_id() {
         SP.set(bsp::memory::boot_core_stack_end() as u64);
-        relocate::relocate_self()
+        runtime_init::runtime_init()
     } else {
         // If not core0, infinitely wait for events.
         wait_forever()
@@ -39,15 +39,6 @@

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
@@ -55,19 +46,3 @@
         asm::wfe()
     }
 }
-
-/// Branch to a raw integer value.
-///
-/// # Safety
-///
-/// - This is highly unsafe. Use with care.
-#[inline(always)]
-pub unsafe fn branch_to_raw_addr(addr: usize) -> ! {
-    asm!(
-        "blr {destination:x}",
-        destination = in(reg) addr,
-        options(nomem, nostack)
-    );
-
-    core::intrinsics::unreachable()
-}

diff -uNr 07_uart_chainloader/src/_arch/aarch64/time.rs 08_timestamps/src/_arch/aarch64/time.rs
--- 07_uart_chainloader/src/_arch/aarch64/time.rs
+++ 08_timestamps/src/_arch/aarch64/time.rs
@@ -0,0 +1,98 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
+
+//! Architectural timer primitives.
+
+use crate::{time, warn};
+use core::time::Duration;
+use cortex_a::regs::*;
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
+        let frq: u64 = CNTFRQ_EL0.get() as u64;
+        let current_count: u64 = CNTPCT_EL0.get() * NS_PER_S;
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
+        let frq = CNTFRQ_EL0.get() as u64;
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

diff -uNr 07_uart_chainloader/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs 08_timestamps/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs
--- 07_uart_chainloader/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs
+++ 08_timestamps/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs
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
-        // - According to Wikipedia, the fastest Pi3 clocks around 1.4 GHz.
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

diff -uNr 07_uart_chainloader/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs 08_timestamps/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
--- 07_uart_chainloader/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
+++ 08_timestamps/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
@@ -289,11 +289,18 @@
                 cpu::nop();
             }

+            // Read one character.
+            let mut ret = inner.registers.DR.get() as u8 as char;
+
+            // Convert carrige return to newline.
+            if ret == '\r' {
+                ret = '\n'
+            }
+
             // Update statistics.
             inner.chars_read += 1;

-            // Read one character.
-            inner.registers.DR.get() as u8 as char
+            ret
         })
     }


diff -uNr 07_uart_chainloader/src/bsp/raspberrypi/link.ld 08_timestamps/src/bsp/raspberrypi/link.ld
--- 07_uart_chainloader/src/bsp/raspberrypi/link.ld
+++ 08_timestamps/src/bsp/raspberrypi/link.ld
@@ -5,15 +5,12 @@

 SECTIONS
 {
-    /* Set the link address to 32 MiB */
-    . = 0x2000000;
+    /* Set current address to the value from which the RPi starts execution */
+    . = 0x80000;

-    __binary_start = .;
     .text :
     {
-        *(.text._start)
-        KEEP(*(.text.runtime_init))
-        *(.text*);
+        *(.text._start) *(.text*)
     }

     .rodata :
@@ -38,16 +35,5 @@
         __bss_end_inclusive = . - 8;
     }

-    .got :
-    {
-        *(.got*)
-    }
-
-    /* Fill up to 8 byte, b/c relocating the binary is done in u64 chunks */
-    . = ALIGN(8);
-    __binary_end_inclusive = . - 8;
-
-    __runtime_init_reloc = runtime_init;
-
     /DISCARD/ : { *(.comment*) }
 }

diff -uNr 07_uart_chainloader/src/bsp/raspberrypi/memory.rs 08_timestamps/src/bsp/raspberrypi/memory.rs
--- 07_uart_chainloader/src/bsp/raspberrypi/memory.rs
+++ 08_timestamps/src/bsp/raspberrypi/memory.rs
@@ -12,9 +12,6 @@

 // Symbols from the linker script.
 extern "Rust" {
-    static __binary_start: UnsafeCell<u64>;
-    static __binary_end_inclusive: UnsafeCell<u64>;
-    static __runtime_init_reloc: UnsafeCell<u64>;
     static __bss_start: UnsafeCell<u64>;
     static __bss_end_inclusive: UnsafeCell<u64>;
 }
@@ -26,12 +23,10 @@
 /// The board's memory map.
 #[rustfmt::skip]
 pub(super) mod map {
-    pub const BOOT_CORE_STACK_END:        usize =        0x8_0000;
+    pub const BOOT_CORE_STACK_END: usize = 0x8_0000;

-    pub const BOARD_DEFAULT_LOAD_ADDRESS: usize =        0x8_0000;
-
-    pub const GPIO_OFFSET:                usize =        0x0020_0000;
-    pub const UART_OFFSET:                usize =        0x0020_1000;
+    pub const GPIO_OFFSET:         usize = 0x0020_0000;
+    pub const UART_OFFSET:         usize = 0x0020_1000;

     /// Physical devices.
     #[cfg(feature = "bsp_rpi3")]
@@ -64,35 +59,13 @@
     map::BOOT_CORE_STACK_END
 }

-/// The address on which the Raspberry firmware loads every binary by default.
-#[inline(always)]
-pub fn board_default_load_addr() -> *const u64 {
-    map::BOARD_DEFAULT_LOAD_ADDRESS as _
-}
-
-/// Return the inclusive range spanning the relocated kernel binary.
-///
-/// # Safety
-///
-/// - Values are provided by the linker script and must be trusted as-is.
-/// - The linker-provided addresses must be u64 aligned.
-pub fn relocated_binary_range_inclusive() -> RangeInclusive<*mut u64> {
-    unsafe { RangeInclusive::new(__binary_start.get(), __binary_end_inclusive.get()) }
-}
-
-/// The relocated address of function `runtime_init()`.
-#[inline(always)]
-pub fn relocated_runtime_init_addr() -> *const u64 {
-    unsafe { __runtime_init_reloc.get() as _ }
-}
-
-/// Return the inclusive range spanning the relocated .bss section.
+/// Return the inclusive range spanning the .bss section.
 ///
 /// # Safety
 ///
 /// - Values are provided by the linker script and must be trusted as-is.
 /// - The linker-provided addresses must be u64 aligned.
-pub fn relocated_bss_range_inclusive() -> RangeInclusive<*mut u64> {
+pub fn bss_range_inclusive() -> RangeInclusive<*mut u64> {
     let range;
     unsafe {
         range = RangeInclusive::new(__bss_start.get(), __bss_end_inclusive.get());

diff -uNr 07_uart_chainloader/src/main.rs 08_timestamps/src/main.rs
--- 07_uart_chainloader/src/main.rs
+++ 08_timestamps/src/main.rs
@@ -11,9 +11,11 @@
 //!
 //! - [`bsp::console::console()`] - Returns a reference to the kernel's [console interface].
 //! - [`bsp::driver::driver_manager()`] - Returns a reference to the kernel's [driver interface].
+//! - [`time::time_manager()`] - Returns a reference to the kernel's [timer interface].
 //!
 //! [console interface]: ../libkernel/console/interface/index.html
 //! [driver interface]: ../libkernel/driver/interface/trait.DriverManager.html
+//! [timer interface]: ../libkernel/time/interface/trait.TimeManager.html
 //!
 //! # Code organization and architecture
 //!
@@ -100,9 +102,7 @@
 //! - `crate::memory::*`
 //! - `crate::bsp::memory::*`

-#![feature(asm)]
 #![feature(const_fn_fn_ptr_basics)]
-#![feature(core_intrinsics)]
 #![feature(format_args_nl)]
 #![feature(panic_info_message)]
 #![feature(trait_alias)]
@@ -110,8 +110,7 @@
 #![no_std]

 // `mod cpu` provides the `_start()` function, the first function to run. `_start()` then calls
-// `relocate::relocate_self()`. `relocate::relocate_self()` calls `runtime_init()`, which jumps to
-// `kernel_init()`.
+// `runtime_init()`, which jumps to `kernel_init()`.

 mod bsp;
 mod console;
@@ -120,9 +119,9 @@
 mod memory;
 mod panic_wait;
 mod print;
-mod relocate;
 mod runtime_init;
 mod synchronization;
+mod time;

 /// Early init code.
 ///
@@ -147,52 +146,31 @@

 /// The main function running after the early init.
 fn kernel_main() -> ! {
-    use bsp::console::console;
-    use console::interface::All;
+    use core::time::Duration;
+    use driver::interface::DriverManager;
+    use time::interface::TimeManager;

-    println!(" __  __ _      _ _                 _ ");
-    println!("|  \\/  (_)_ _ (_) |   ___  __ _ __| |");
-    println!("| |\\/| | | ' \\| | |__/ _ \\/ _` / _` |");
-    println!("|_|  |_|_|_||_|_|____\\___/\\__,_\\__,_|");
-    println!();
-    println!("{:^37}", bsp::board_name());
-    println!();
-    println!("[ML] Requesting binary");
-    console().flush();
-
-    // Clear the RX FIFOs, if any, of spurious received characters before starting with the loader
-    // protocol.
-    console().clear();
-
-    // Notify `Minipush` to send the binary.
-    for _ in 0..3 {
-        console().write_char(3 as char);
-    }
+    info!("Booting on: {}", bsp::board_name());

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

-    println!("[ML] Loaded! Executing the payload now\n");
-    console().flush();
+    // Test a failing timer case.
+    time::time_manager().spin_for(Duration::from_nanos(1));

-    // Use black magic to get a function pointer.
-    let kernel: fn() -> ! = unsafe { core::mem::transmute(kernel_addr) };
-
-    // Jump to loaded kernel!
-    kernel()
+    loop {
+        info!("Spinning for 1 second");
+        time::time_manager().spin_for(Duration::from_secs(1));
+    }
 }

diff -uNr 07_uart_chainloader/src/print.rs 08_timestamps/src/print.rs
--- 07_uart_chainloader/src/print.rs
+++ 08_timestamps/src/print.rs
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

diff -uNr 07_uart_chainloader/src/relocate.rs 08_timestamps/src/relocate.rs
--- 07_uart_chainloader/src/relocate.rs
+++ 08_timestamps/src/relocate.rs
@@ -1,51 +0,0 @@
-// SPDX-License-Identifier: MIT OR Apache-2.0
-//
-// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
-
-//! Relocation code.
-
-use crate::{bsp, cpu};
-
-//--------------------------------------------------------------------------------------------------
-// Public Code
-//--------------------------------------------------------------------------------------------------
-
-/// Relocates the own binary from `bsp::memory::board_default_load_addr()` to the `__binary_start`
-/// address from the linker script.
-///
-/// # Safety
-///
-/// - Only a single core must be active and running this function.
-/// - Function must not use the `bss` section.
-#[inline(never)]
-pub unsafe fn relocate_self() -> ! {
-    let range = bsp::memory::relocated_binary_range_inclusive();
-    let mut relocated_binary_start_addr = *range.start();
-    let relocated_binary_end_addr_inclusive = *range.end();
-
-    // The address of where the previous firmware loaded us.
-    let mut current_binary_start_addr = bsp::memory::board_default_load_addr();
-
-    // Copy the whole binary.
-    while relocated_binary_start_addr <= relocated_binary_end_addr_inclusive {
-        core::ptr::write_volatile(
-            relocated_binary_start_addr,
-            core::ptr::read_volatile(current_binary_start_addr),
-        );
-        relocated_binary_start_addr = relocated_binary_start_addr.offset(1);
-        current_binary_start_addr = current_binary_start_addr.offset(1);
-    }
-
-    // The following function calls form a hack to achieve an "absolute jump" to
-    // `runtime_init::runtime_init()` by forcing an indirection through the global offset table
-    // (GOT), so that execution continues from the relocated binary.
-    //
-    // Without this, the address of `runtime_init()` would be calculated as a relative offset from
-    // the current program counter, since we are compiling as `position independent code`. This
-    // would cause us to keep executing from the address to which the firmware loaded us, instead of
-    // the relocated position.
-    //
-    // There likely is a more elegant way to do this.
-    let relocated_runtime_init_addr = bsp::memory::relocated_runtime_init_addr() as usize;
-    cpu::branch_to_raw_addr(relocated_runtime_init_addr)
-}

diff -uNr 07_uart_chainloader/src/runtime_init.rs 08_timestamps/src/runtime_init.rs
--- 07_uart_chainloader/src/runtime_init.rs
+++ 08_timestamps/src/runtime_init.rs
@@ -17,7 +17,7 @@
 /// - Must only be called pre `kernel_init()`.
 #[inline(always)]
 unsafe fn zero_bss() {
-    memory::zero_volatile(bsp::memory::relocated_bss_range_inclusive());
+    memory::zero_volatile(bsp::memory::bss_range_inclusive());
 }

 //--------------------------------------------------------------------------------------------------
@@ -30,7 +30,6 @@
 /// # Safety
 ///
 /// - Only a single core must be active and running this function.
-#[no_mangle]
 pub unsafe fn runtime_init() -> ! {
     zero_bss();


diff -uNr 07_uart_chainloader/src/time.rs 08_timestamps/src/time.rs
--- 07_uart_chainloader/src/time.rs
+++ 08_timestamps/src/time.rs
@@ -0,0 +1,35 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2020-2021 Andre Richter <andre.o.richter@gmail.com>
+
+//! Timer primitives.
+
+#[cfg(target_arch = "aarch64")]
+#[path = "_arch/aarch64/time.rs"]
+mod arch_time;
+pub use arch_time::*;
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
+    ///
+    /// The `BSP` is supposed to supply one global instance.
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

diff -uNr 07_uart_chainloader/update.sh 08_timestamps/update.sh
--- 07_uart_chainloader/update.sh
+++ 08_timestamps/update.sh
@@ -1,8 +0,0 @@
-#!/usr/bin/env bash
-
-cd ../06_drivers_gpio_uart
-BSP=rpi4 make
-cp kernel8.img ../07_uart_chainloader/demo_payload_rpi4.img
-make
-cp kernel8.img ../07_uart_chainloader/demo_payload_rpi3.img
-rm kernel8.img

```
