# Tutorial 08 - Timestamps

## tl;dr

We add abstractions for the architectural timer, implement it for `aarch64` and use it to annotate
prints with timestamps; A `warn!()` macro is added.

## Test it

Check it out via chainboot (added in previous tutorial):
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
[MP] ⏩ Pushing 12 KiB =========================================🦀 100% 0 KiB/s Time: 00:00:00
[ML] Loaded! Executing the payload now

[    0.586140] Booting on: Raspberry Pi 3
[    0.587227] Architectural timer resolution: 52 ns
[    0.589530] Drivers loaded:
[    0.590876]       1. BCM GPIO
[    0.592309]       2. BCM PL011 UART
[W   0.594005] Spin duration smaller than architecturally supported, skipping
[    0.597392] Spinning for 1 second
[    1.599001] Spinning for 1 second
[    2.599872] Spinning for 1 second

```

## Diff to previous
```diff
Binary files 07_uart_chainloader/demo_payload_rpi3.img and 08_timestamps/demo_payload_rpi3.img differ
Binary files 07_uart_chainloader/demo_payload_rpi4.img and 08_timestamps/demo_payload_rpi4.img differ

diff -uNr 07_uart_chainloader/Makefile 08_timestamps/Makefile
--- 07_uart_chainloader/Makefile
+++ 08_timestamps/Makefile
@@ -19,8 +19,7 @@
     QEMU_MACHINE_TYPE = raspi3
     QEMU_RELEASE_ARGS = -serial stdio -display none
     LINKER_FILE       = src/bsp/raspberrypi/link.ld
-    RUSTC_MISC_ARGS   = -C target-cpu=cortex-a53 -C relocation-model=pic
-    CHAINBOOT_DEMO_PAYLOAD = demo_payload_rpi3.img
+    RUSTC_MISC_ARGS   = -C target-cpu=cortex-a53
 else ifeq ($(BSP),rpi4)
     TARGET            = aarch64-unknown-none-softfloat
     KERNEL_BIN        = kernel8.img
@@ -28,8 +27,7 @@
     QEMU_MACHINE_TYPE =
     QEMU_RELEASE_ARGS = -serial stdio -display none
     LINKER_FILE       = src/bsp/raspberrypi/link.ld
-    RUSTC_MISC_ARGS   = -C target-cpu=cortex-a72 -C relocation-model=pic
-    CHAINBOOT_DEMO_PAYLOAD = demo_payload_rpi4.img
+    RUSTC_MISC_ARGS   = -C target-cpu=cortex-a72
 endif

 # Export for build.rs
@@ -69,8 +67,7 @@
 EXEC_QEMU     = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)
 EXEC_MINIPUSH = ruby ../utils/minipush.rb

-.PHONY: all $(KERNEL_ELF) $(KERNEL_BIN) doc qemu qemuasm chainboot clippy clean readelf objdump nm \
-    check
+.PHONY: all $(KERNEL_ELF) $(KERNEL_BIN) doc qemu chainboot clippy clean readelf objdump nm check

 all: $(KERNEL_BIN)

@@ -84,18 +81,15 @@
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

diff -uNr 07_uart_chainloader/src/_arch/aarch64/cpu.rs 08_timestamps/src/_arch/aarch64/cpu.rs
--- 07_uart_chainloader/src/_arch/aarch64/cpu.rs
+++ 08_timestamps/src/_arch/aarch64/cpu.rs
@@ -21,12 +21,12 @@
 #[naked]
 #[no_mangle]
 pub unsafe extern "C" fn _start() -> ! {
-    use crate::relocate;
+    use crate::runtime_init;

     // Expect the boot core to start in EL2.
     if bsp::cpu::BOOT_CORE_ID == cpu::smp::core_id() {
         SP.set(bsp::cpu::BOOT_CORE_STACK_START);
-        relocate::relocate_self::<u64>()
+        runtime_init::runtime_init()
     } else {
         // If not core0, infinitely wait for events.
         wait_forever()

diff -uNr 07_uart_chainloader/src/_arch/aarch64/time.rs 08_timestamps/src/_arch/aarch64/time.rs
--- 07_uart_chainloader/src/_arch/aarch64/time.rs
+++ 08_timestamps/src/_arch/aarch64/time.rs
@@ -0,0 +1,101 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
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
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
+/// ARMv8 Generic Timer.
+pub struct GenericTimer;
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
+        CNTP_TVAL_EL0.set(tval as u32);
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

diff -uNr 07_uart_chainloader/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs 08_timestamps/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
--- 07_uart_chainloader/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
+++ 08_timestamps/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
@@ -288,11 +288,18 @@
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


diff -uNr 07_uart_chainloader/src/bsp/raspberrypi/cpu.rs 08_timestamps/src/bsp/raspberrypi/cpu.rs
--- 07_uart_chainloader/src/bsp/raspberrypi/cpu.rs
+++ 08_timestamps/src/bsp/raspberrypi/cpu.rs
@@ -13,6 +13,3 @@

 /// The early boot core's stack address.
 pub const BOOT_CORE_STACK_START: u64 = 0x80_000;
-
-/// The address on which the Raspberry firmware loads every binary by default.
-pub const BOARD_DEFAULT_LOAD_ADDRESS: usize = 0x80_000;

diff -uNr 07_uart_chainloader/src/bsp/raspberrypi/link.ld 08_timestamps/src/bsp/raspberrypi/link.ld
--- 07_uart_chainloader/src/bsp/raspberrypi/link.ld
+++ 08_timestamps/src/bsp/raspberrypi/link.ld
@@ -5,10 +5,9 @@

 SECTIONS
 {
-    /* Set the link address to 32 MiB */
-    . = 0x2000000;
+    /* Set current address to the value from which the RPi starts execution */
+    . = 0x80000;

-    __binary_start = .;
     .text :
     {
         *(.text._start) *(.text*)
@@ -33,14 +32,5 @@
         __bss_end = .;
     }

-    .got :
-    {
-        *(.got*)
-    }
-
-    /* Fill up to 8 byte, b/c relocating the binary is done in u64 chunks */
-    . = ALIGN(8);
-    __binary_end = .;
-
     /DISCARD/ : { *(.comment*) }
 }

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
@@ -108,8 +110,7 @@
 #![no_std]

 // `mod cpu` provides the `_start()` function, the first function to run. `_start()` then calls
-// `relocate::relocate_self()`. `relocate::relocate_self()` calls `runtime_init()`, which jumps to
-// `kernel_init()`.
+// `runtime_init()`, which jumps to `kernel_init()`.

 mod bsp;
 mod console;
@@ -118,9 +119,9 @@
 mod memory;
 mod panic_wait;
 mod print;
-mod relocate;
 mod runtime_init;
 mod synchronization;
+mod time;

 /// Early init code.
 ///
@@ -145,52 +146,31 @@

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
-    let kernel_addr: *mut u8 = bsp::cpu::BOARD_DEFAULT_LOAD_ADDRESS as *mut u8;
-    unsafe {
-        // Read the kernel byte by byte.
-        for i in 0..size {
-            *kernel_addr.offset(i as isize) = console().read_char() as u8;
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
-    let kernel: extern "C" fn() -> ! = unsafe { core::mem::transmute(kernel_addr as *const ()) };
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
@@ -40,3 +40,71 @@
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
@@ -1,52 +0,0 @@
-// SPDX-License-Identifier: MIT OR Apache-2.0
-//
-// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
-
-//! Relocation code.
-
-use crate::{bsp, runtime_init};
-
-//--------------------------------------------------------------------------------------------------
-// Public Code
-//--------------------------------------------------------------------------------------------------
-
-/// Relocates the own binary from `bsp::cpu::BOARD_DEFAULT_LOAD_ADDRESS` to the `__binary_start`
-/// address from the linker script.
-///
-/// # Safety
-///
-/// - Only a single core must be active and running this function.
-/// - Function must not use the `bss` section.
-pub unsafe fn relocate_self<T>() -> ! {
-    extern "C" {
-        static __binary_start: usize;
-        static __binary_end: usize;
-    }
-
-    let binary_start_addr: usize = &__binary_start as *const _ as _;
-    let binary_end_addr: usize = &__binary_end as *const _ as _;
-    let binary_size_in_byte: usize = binary_end_addr - binary_start_addr;
-
-    // Get the relocation destination address from the linker symbol.
-    let mut reloc_dst_addr: *mut T = binary_start_addr as *mut T;
-
-    // The address of where the previous firmware loaded us.
-    let mut src_addr: *const T = bsp::cpu::BOARD_DEFAULT_LOAD_ADDRESS as *const _;
-
-    // Copy the whole binary.
-    //
-    // This is essentially a `memcpy()` optimized for throughput by transferring in chunks of T.
-    let n = binary_size_in_byte / core::mem::size_of::<T>();
-    for _ in 0..n {
-        use core::ptr;
-
-        ptr::write_volatile::<T>(reloc_dst_addr, ptr::read_volatile::<T>(src_addr));
-        reloc_dst_addr = reloc_dst_addr.offset(1);
-        src_addr = src_addr.offset(1);
-    }
-
-    // Call `runtime_init()` through a trait object, causing the jump to use an absolute address to
-    // reach the relocated binary. An elaborate explanation can be found in the `runtime_init.rs`
-    // source comments.
-    runtime_init::get().runtime_init()
-}

diff -uNr 07_uart_chainloader/src/runtime_init.rs 08_timestamps/src/runtime_init.rs
--- 07_uart_chainloader/src/runtime_init.rs
+++ 08_timestamps/src/runtime_init.rs
@@ -8,43 +8,9 @@
 use core::ops::Range;

 //--------------------------------------------------------------------------------------------------
-// Private Definitions
-//--------------------------------------------------------------------------------------------------
-
-struct Traitor;
-
-//--------------------------------------------------------------------------------------------------
-// Public Definitions
-//--------------------------------------------------------------------------------------------------
-
-/// We are outsmarting the compiler here by using a trait as a layer of indirection. Because we are
-/// generating PIC code, a static dispatch to `init()` would generate a relative jump from the
-/// callee to `init()`. However, when calling `init()`, code just finished copying the binary to the
-/// actual link-time address, and hence is still running at whatever location the previous loader
-/// has put it. So we do not want a relative jump, because it would not jump to the relocated code.
-///
-/// By indirecting through a trait object, we can make use of the property that vtables store
-/// absolute addresses. So calling `init()` this way will kick execution to the relocated binary.
-pub trait RunTimeInit {
-    /// Equivalent to `crt0` or `c0` code in C/C++ world. Clears the `bss` section, then jumps to
-    /// kernel init code.
-    ///
-    /// # Safety
-    ///
-    /// - Only a single core must be active and running this function.
-    unsafe fn runtime_init(&self) -> ! {
-        zero_bss();
-
-        crate::kernel_init()
-    }
-}
-
-//--------------------------------------------------------------------------------------------------
 // Private Code
 //--------------------------------------------------------------------------------------------------

-impl RunTimeInit for Traitor {}
-
 /// Return the range spanning the .bss section.
 ///
 /// # Safety
@@ -78,7 +44,14 @@
 // Public Code
 //--------------------------------------------------------------------------------------------------

-/// Give the callee a `RunTimeInit` trait object.
-pub fn get() -> &'static dyn RunTimeInit {
-    &Traitor {}
+/// Equivalent to `crt0` or `c0` code in C/C++ world. Clears the `bss` section, then jumps to kernel
+/// init code.
+///
+/// # Safety
+///
+/// - Only a single core must be active and running this function.
+pub unsafe fn runtime_init() -> ! {
+    zero_bss();
+
+    crate::kernel_init()
 }

diff -uNr 07_uart_chainloader/src/time.rs 08_timestamps/src/time.rs
--- 07_uart_chainloader/src/time.rs
+++ 08_timestamps/src/time.rs
@@ -0,0 +1,35 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2020 Andre Richter <andre.o.richter@gmail.com>
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

```
