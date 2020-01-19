# Tutorial 08 - Timestamps

## tl;dr

We add abstractions for the architectural timer, implement it for `aarch64` and
use it to annotate prints with timestamps; A `warn!()` macro is added.

## Test it

Check it out via chainboot (added in previous tutorial):
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
[MP] ⏩ Pushing 12 KiB =========================================🦀 100% 0 KiB/s Time: 00:00:00
[ML] Loaded! Executing the payload now

[    0.585762] Booting on: Raspberry Pi 3
[    0.586849] Architectural timer resolution: 52 ns
[    0.589152] Drivers loaded:
[    0.590498]       1. GPIO
[    0.591758]       2. PL011Uart
[W   0.593235] Spin duration smaller than architecturally supported, skipping
[    0.596623] Spinning for 1 second
[    1.598232] Spinning for 1 second
[    2.599104] Spinning for 1 second
```

## Diff to previous
```diff
Binary files 07_uart_chainloader/demo_payload_rpi3.img and 08_timestamps/demo_payload_rpi3.img differ
Binary files 07_uart_chainloader/demo_payload_rpi4.img and 08_timestamps/demo_payload_rpi4.img differ

diff -uNr 07_uart_chainloader/Makefile 08_timestamps/Makefile
--- 07_uart_chainloader/Makefile
+++ 08_timestamps/Makefile
@@ -20,8 +20,7 @@
 	QEMU_MACHINE_TYPE = raspi3
 	QEMU_RELEASE_ARGS = -serial stdio -display none
 	LINKER_FILE       = src/bsp/rpi/link.ld
-	RUSTC_MISC_ARGS   = -C target-cpu=cortex-a53 -C relocation-model=pic
-	CHAINBOOT_DEMO_PAYLOAD = demo_payload_rpi3.img
+	RUSTC_MISC_ARGS   = -C target-cpu=cortex-a53
 else ifeq ($(BSP),rpi4)
 	TARGET            = aarch64-unknown-none-softfloat
 	OUTPUT            = kernel8.img
@@ -29,8 +28,7 @@
 	# QEMU_MACHINE_TYPE =
 	# QEMU_RELEASE_ARGS = -serial stdio -display none
 	LINKER_FILE       = src/bsp/rpi/link.ld
-	RUSTC_MISC_ARGS   = -C target-cpu=cortex-a72 -C relocation-model=pic
-	CHAINBOOT_DEMO_PAYLOAD = demo_payload_rpi4.img
+	RUSTC_MISC_ARGS   = -C target-cpu=cortex-a72
 endif

 RUSTFLAGS          = -C link-arg=-T$(LINKER_FILE) $(RUSTC_MISC_ARGS)
@@ -58,7 +56,7 @@
 DOCKER_EXEC_QEMU     = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)
 DOCKER_EXEC_MINIPUSH = ruby /utils/minipush.rb

-.PHONY: all doc qemu qemuasm chainboot clippy clean readelf objdump nm
+.PHONY: all doc qemu chainboot clippy clean readelf objdump nm

 all: clean $(OUTPUT)

@@ -76,25 +74,17 @@
 ifeq ($(QEMU_MACHINE_TYPE),)
 qemu:
 	@echo "This board is not yet supported for QEMU."
-
-qemuasm:
-	@echo "This board is not yet supported for QEMU."
 else
 qemu: all
 	@$(DOCKER_CMD) $(DOCKER_ARG_DIR_TUT) $(DOCKER_IMAGE) \
 		$(DOCKER_EXEC_QEMU) $(QEMU_RELEASE_ARGS)     \
 		-kernel $(OUTPUT)
-
-qemuasm: all
-	@$(DOCKER_CMD) $(DOCKER_ARG_DIR_TUT) $(DOCKER_IMAGE) \
-		$(DOCKER_EXEC_QEMU) $(QEMU_RELEASE_ARGS)     \
-		-kernel $(OUTPUT) -d in_asm
 endif

-chainboot:
+chainboot: all
 	@$(DOCKER_CMD) $(DOCKER_ARG_DIR_TUT) $(DOCKER_ARG_DIR_UTILS) $(DOCKER_ARG_TTY) \
 		$(DOCKER_IMAGE) $(DOCKER_EXEC_MINIPUSH) $(DEV_SERIAL)                  \
-		$(CHAINBOOT_DEMO_PAYLOAD)
+		$(OUTPUT)

 clippy:
 	RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" cargo xclippy --target=$(TARGET) --features bsp_$(BSP)

diff -uNr 07_uart_chainloader/src/arch/aarch64/time.rs 08_timestamps/src/arch/aarch64/time.rs
--- 07_uart_chainloader/src/arch/aarch64/time.rs
+++ 08_timestamps/src/arch/aarch64/time.rs
@@ -0,0 +1,81 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
+
+//! Timer primitives.
+
+use crate::{interface, warn};
+use core::time::Duration;
+use cortex_a::regs::*;
+
+const NS_PER_S: u64 = 1_000_000_000;
+
+//--------------------------------------------------------------------------------------------------
+// Arch-public
+//--------------------------------------------------------------------------------------------------
+
+pub struct Timer;
+
+//--------------------------------------------------------------------------------------------------
+// OS interface implementations
+//--------------------------------------------------------------------------------------------------
+
+impl interface::time::Timer for Timer {
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

diff -uNr 07_uart_chainloader/src/arch/aarch64.rs 08_timestamps/src/arch/aarch64.rs
--- 07_uart_chainloader/src/arch/aarch64.rs
+++ 08_timestamps/src/arch/aarch64.rs
@@ -5,8 +5,9 @@
 //! AArch64.

 pub mod sync;
+mod time;

-use crate::bsp;
+use crate::{bsp, interface};
 use cortex_a::{asm, regs::*};

 /// The entry of the `kernel` binary.
@@ -22,7 +23,7 @@

     if bsp::BOOT_CORE_ID == MPIDR_EL1.get() & CORE_MASK {
         SP.set(bsp::BOOT_CORE_STACK_START);
-        crate::relocate::relocate_self::<u64>()
+        crate::runtime_init::runtime_init()
     } else {
         // If not core0, infinitely wait for events.
         wait_forever()
@@ -30,6 +31,12 @@
 }

 //--------------------------------------------------------------------------------------------------
+// Global instances
+//--------------------------------------------------------------------------------------------------
+
+static TIMER: time::Timer = time::Timer;
+
+//--------------------------------------------------------------------------------------------------
 // Implementation of the kernel's architecture abstraction code
 //--------------------------------------------------------------------------------------------------

@@ -42,6 +49,11 @@
     }
 }

+/// Return a reference to a `interface::time::TimeKeeper` implementation.
+pub fn timer() -> &'static impl interface::time::Timer {
+    &TIMER
+}
+
 /// Pause execution on the calling CPU core.
 #[inline(always)]
 pub fn wait_forever() -> ! {

diff -uNr 07_uart_chainloader/src/bsp/driver/bcm/bcm2xxx_pl011_uart.rs 08_timestamps/src/bsp/driver/bcm/bcm2xxx_pl011_uart.rs
--- 07_uart_chainloader/src/bsp/driver/bcm/bcm2xxx_pl011_uart.rs
+++ 08_timestamps/src/bsp/driver/bcm/bcm2xxx_pl011_uart.rs
@@ -293,11 +293,18 @@
                 arch::nop();
             }

+            // Read one character.
+            let mut ret = inner.DR.get() as u8 as char;
+
+            // Convert carrige return to newline.
+            if ret == '\r' {
+                ret = '\n'
+            }
+
             // Update statistics.
             inner.chars_read += 1;

-            // Read one character.
-            inner.DR.get() as u8 as char
+            ret
         })
     }


diff -uNr 07_uart_chainloader/src/bsp/rpi/link.ld 08_timestamps/src/bsp/rpi/link.ld
--- 07_uart_chainloader/src/bsp/rpi/link.ld
+++ 08_timestamps/src/bsp/rpi/link.ld
@@ -5,10 +5,9 @@

 SECTIONS
 {
-    /* Set the link address to the top-most 40 KiB of DRAM (assuming 1GiB) */
-    . = 0x3F000000 - 0x10000;
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

diff -uNr 07_uart_chainloader/src/bsp/rpi.rs 08_timestamps/src/bsp/rpi.rs
--- 07_uart_chainloader/src/bsp/rpi.rs
+++ 08_timestamps/src/bsp/rpi.rs
@@ -16,9 +16,6 @@
 /// The early boot core's stack address.
 pub const BOOT_CORE_STACK_START: u64 = 0x80_000;

-/// The address on which the RPi3 firmware loads every binary by default.
-pub const BOARD_DEFAULT_LOAD_ADDRESS: usize = 0x80_000;
-
 //--------------------------------------------------------------------------------------------------
 // Global BSP driver instances
 //--------------------------------------------------------------------------------------------------

diff -uNr 07_uart_chainloader/src/interface.rs 08_timestamps/src/interface.rs
--- 07_uart_chainloader/src/interface.rs
+++ 08_timestamps/src/interface.rs
@@ -112,3 +112,22 @@
         }
     }
 }
+
+/// Timekeeping interfaces.
+pub mod time {
+    use core::time::Duration;
+
+    /// Timer functions.
+    pub trait Timer {
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

diff -uNr 07_uart_chainloader/src/main.rs 08_timestamps/src/main.rs
--- 07_uart_chainloader/src/main.rs
+++ 08_timestamps/src/main.rs
@@ -29,11 +29,7 @@
 // the first function to run.
 mod arch;

-// `_start()` then calls `relocate::relocate_self()`.
-mod relocate;
-
-// `relocate::relocate_self()` calls `runtime_init()`, which on completion, jumps to
-// `kernel_init()`.
+// `_start()` then calls `runtime_init()`, which on completion, jumps to `kernel_init()`.
 mod runtime_init;

 // Conditionally includes the selected `BSP` code.
@@ -67,51 +63,25 @@

 /// The main function running after the early init.
 fn kernel_main() -> ! {
-    use interface::console::All;
-
-    println!(" __  __ _      _ _                 _ ");
-    println!("|  \\/  (_)_ _ (_) |   ___  __ _ __| |");
-    println!("| |\\/| | | ' \\| | |__/ _ \\/ _` / _` |");
-    println!("|_|  |_|_|_||_|_|____\\___/\\__,_\\__,_|");
-    println!();
-    println!("{:^37}", bsp::board_name());
-    println!();
-    println!("[ML] Requesting binary");
-    bsp::console().flush();
-
-    // Clear the RX FIFOs, if any, of spurious received characters before starting with the loader
-    // protocol.
-    bsp::console().clear();
-
-    // Notify `Minipush` to send the binary.
-    for _ in 0..3 {
-        bsp::console().write_char(3 as char);
-    }
+    use core::time::Duration;
+    use interface::time::Timer;

-    // Read the binary's size.
-    let mut size: u32 = u32::from(bsp::console().read_char() as u8);
-    size |= u32::from(bsp::console().read_char() as u8) << 8;
-    size |= u32::from(bsp::console().read_char() as u8) << 16;
-    size |= u32::from(bsp::console().read_char() as u8) << 24;
-
-    // Trust it's not too big.
-    bsp::console().write_char('O');
-    bsp::console().write_char('K');
-
-    let kernel_addr: *mut u8 = bsp::BOARD_DEFAULT_LOAD_ADDRESS as *mut u8;
-    unsafe {
-        // Read the kernel byte by byte.
-        for i in 0..size {
-            *kernel_addr.offset(i as isize) = bsp::console().read_char() as u8;
-        }
+    info!("Booting on: {}", bsp::board_name());
+    info!(
+        "Architectural timer resolution: {} ns",
+        arch::timer().resolution().as_nanos()
+    );
+
+    info!("Drivers loaded:");
+    for (i, driver) in bsp::device_drivers().iter().enumerate() {
+        info!("      {}. {}", i + 1, driver.compatible());
     }

-    println!("[ML] Loaded! Executing the payload now\n");
-    bsp::console().flush();
-
-    // Use black magic to get a function pointer.
-    let kernel: extern "C" fn() -> ! = unsafe { core::mem::transmute(kernel_addr as *const ()) };
+    // Test a failing timer case.
+    arch::timer().spin_for(Duration::from_nanos(1));

-    // Jump to loaded kernel!
-    kernel()
+    loop {
+        info!("Spinning for 1 second");
+        arch::timer().spin_for(Duration::from_secs(1));
+    }
 }

diff -uNr 07_uart_chainloader/src/print.rs 08_timestamps/src/print.rs
--- 07_uart_chainloader/src/print.rs
+++ 08_timestamps/src/print.rs
@@ -32,3 +32,71 @@
         $crate::print::_print(format_args_nl!($($arg)*));
     })
 }
+
+/// Prints an info, with newline.
+#[macro_export]
+macro_rules! info {
+    ($string:expr) => ({
+        #[allow(unused_imports)]
+        use crate::interface::time::Timer;
+
+        let timestamp = $crate::arch::timer().uptime();
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
+        use crate::interface::time::Timer;
+
+        let timestamp = $crate::arch::timer().uptime();
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
+/// Prints a warning, with newline.
+#[macro_export]
+macro_rules! warn {
+    ($string:expr) => ({
+        #[allow(unused_imports)]
+        use crate::interface::time::Timer;
+
+        let timestamp = $crate::arch::timer().uptime();
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
+        use crate::interface::time::Timer;
+
+        let timestamp = $crate::arch::timer().uptime();
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
@@ -1,46 +0,0 @@
-// SPDX-License-Identifier: MIT OR Apache-2.0
-//
-// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
-
-//! Relocation code.
-
-/// Relocates the own binary from `bsp::BOARD_DEFAULT_LOAD_ADDRESS` to the `__binary_start` address
-/// from the linker script.
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
-    let mut src_addr: *const T = crate::bsp::BOARD_DEFAULT_LOAD_ADDRESS as *const _;
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
-    // Call `init()` through a trait object, causing the jump to use an absolute address to reach
-    // the relocated binary. An elaborate explanation can be found in the runtime_init.rs source
-    // comments.
-    crate::runtime_init::get().runtime_init()
-}

diff -uNr 07_uart_chainloader/src/runtime_init.rs 08_timestamps/src/runtime_init.rs
--- 07_uart_chainloader/src/runtime_init.rs
+++ 08_timestamps/src/runtime_init.rs
@@ -36,32 +36,14 @@
     memory::zero_volatile(bss_range());
 }

-/// We are outsmarting the compiler here by using a trait as a layer of indirection. Because we are
-/// generating PIC code, a static dispatch to `init()` would generate a relative jump from the
-/// callee to `init()`. However, when calling `init()`, code just finished copying the binary to the
-/// actual link-time address, and hence is still running at whatever location the previous loader
-/// has put it. So we do not want a relative jump, because it would not jump to the relocated code.
+/// Equivalent to `crt0` or `c0` code in C/C++ world. Clears the `bss` section, then jumps to kernel
+/// init code.
 ///
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
-struct Traitor;
-impl RunTimeInit for Traitor {}
+/// # Safety
+///
+/// - Only a single core must be active and running this function.
+pub unsafe fn runtime_init() -> ! {
+    zero_bss();

-/// Give the callee a `RunTimeInit` trait object.
-pub fn get() -> &'static dyn RunTimeInit {
-    &Traitor {}
+    crate::kernel_init()
 }

```
