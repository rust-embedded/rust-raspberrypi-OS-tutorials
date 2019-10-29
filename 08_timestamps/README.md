# Tutorial 08 - Timestamps

## tl;dr

We add abstractions for the architectural timer, implement it for `aarch64` and
use it to annotate prints with timestamps; A `warn!()` macro is added.

Check it out via chainboot (added in previous tutorial):
```console
make chainboot
[...]
### Listening on /dev/ttyUSB0
 __  __ _      _ _                 _
|  \/  (_)_ _ (_) |   ___  __ _ __| |
| |\/| | | ' \| | |__/ _ \/ _` / _` |
|_|  |_|_|_||_|_|____\___/\__,_\__,_|

           Raspberry Pi 3

[ML] Requesting binary
### sending kernel kernel8.img [13872 byte]
### finished sending
[ML] Loaded! Executing the payload now

[    1.233286] Booting on: Raspberry Pi 3
[    1.236282] Architectural timer resolution: 52 ns
[    1.241023] Drivers loaded:
[    1.243833]       1. GPIO
[    1.246467]       2. PL011Uart
[W   1.249453] Spin duration smaller than architecturally supported, skipping
[    1.256390] Spinning for 1 second
[    2.259728] Spinning for 1 second
[    3.262286] Spinning for 1 second
```

## Diff to previous
```diff
Binary files 07_uart_chainloader/demo_payload_rpi3.img and 08_timestamps/demo_payload_rpi3.img differ
Binary files 07_uart_chainloader/demo_payload_rpi4.img and 08_timestamps/demo_payload_rpi4.img differ

diff -uNr 07_uart_chainloader/Makefile 08_timestamps/Makefile
--- 07_uart_chainloader/Makefile
+++ 08_timestamps/Makefile
@@ -15,8 +15,7 @@
 	QEMU_MACHINE_TYPE = raspi3
 	QEMU_MISC_ARGS = -serial stdio
 	LINKER_FILE = src/bsp/rpi/link.ld
-	RUSTC_MISC_ARGS = -C target-cpu=cortex-a53 -C relocation-model=pic
-	CHAINBOOT_DEMO_PAYLOAD = demo_payload_rpi3.img
+	RUSTC_MISC_ARGS = -C target-cpu=cortex-a53
 else ifeq ($(BSP),rpi4)
 	TARGET = aarch64-unknown-none-softfloat
 	OUTPUT = kernel8.img
@@ -24,8 +23,7 @@
 #	QEMU_MACHINE_TYPE =
 #	QEMU_MISC_ARGS = -serial stdio
 	LINKER_FILE = src/bsp/rpi/link.ld
-	RUSTC_MISC_ARGS = -C target-cpu=cortex-a72 -C relocation-model=pic
-	CHAINBOOT_DEMO_PAYLOAD = demo_payload_rpi4.img
+	RUSTC_MISC_ARGS = -C target-cpu=cortex-a72
 endif

 SOURCES = $(wildcard **/*.rs) $(wildcard **/*.S) $(wildcard **/*.ld)
@@ -56,7 +54,7 @@
 DOCKER_EXEC_RASPBOOT_DEV = /dev/ttyUSB0
 # DOCKER_EXEC_RASPBOOT_DEV = /dev/ttyACM0

-.PHONY: all doc qemu qemuasm chainboot clippy clean readelf objdump nm
+.PHONY: all doc qemu chainboot clippy clean readelf objdump nm

 all: clean $(OUTPUT)

@@ -74,23 +72,16 @@
 ifeq ($(QEMU_MACHINE_TYPE),)
 qemu:
 	@echo "This board is not yet supported for QEMU."
-
-qemuasm:
-	@echo "This board is not yet supported for QEMU."
 else
 qemu: all
 	$(DOCKER_CMD) $(DOCKER_ARG_CURDIR) $(CONTAINER_UTILS) \
 	$(DOCKER_EXEC_QEMU) $(QEMU_MISC_ARGS)
-
-qemuasm: all
-	$(DOCKER_CMD) $(DOCKER_ARG_CURDIR) $(CONTAINER_UTILS) \
-	$(DOCKER_EXEC_QEMU) -d in_asm
 endif

-chainboot:
+chainboot: all
 	$(DOCKER_CMD) $(DOCKER_ARG_CURDIR) $(DOCKER_ARG_TTY) \
 	$(CONTAINER_UTILS) $(DOCKER_EXEC_RASPBOOT) $(DOCKER_EXEC_RASPBOOT_DEV) \
-	$(CHAINBOOT_DEMO_PAYLOAD)
+	$(OUTPUT)

 clippy:
 	cargo xclippy --target=$(TARGET) --features bsp_$(BSP)

diff -uNr 07_uart_chainloader/src/arch/aarch64/time.rs 08_timestamps/src/arch/aarch64/time.rs
--- 07_uart_chainloader/src/arch/aarch64/time.rs
+++ 08_timestamps/src/arch/aarch64/time.rs
@@ -0,0 +1,77 @@
+// SPDX-License-Identifier: MIT
+//
+// Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>
+
+//! Timer primitives.
+
+use crate::{interface, warn};
+use core::time::Duration;
+use cortex_a::regs::*;
+
+const NS_PER_S: u64 = 1_000_000_000;
+
+pub struct Timer;
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
+        loop {
+            // ISTATUS will be '1' when cval ticks have passed. Busy-check it.
+            if CNTP_CTL_EL0.is_set(CNTP_CTL_EL0::ISTATUS) {
+                break;
+            }
+        }
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
+        crate::runtime_init::init()
     } else {
         // if not core0, infinitely wait for events
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
@@ -300,7 +300,14 @@
             }

             // Read one character.
-            inner.DR.get() as u8 as char
+            let mut ret = inner.DR.get() as u8 as char;
+
+            // Convert carrige return to newline.
+            if ret == '\r' {
+                ret = '\n'
+            }
+
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
@@ -32,14 +31,5 @@
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
@@ -12,9 +12,6 @@
 pub const BOOT_CORE_ID: u64 = 0;
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
@@ -108,3 +108,22 @@
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
-// `relocate::relocate_self()` calls `runtime_init::init()`, which on completion, jumps to
-// `kernel_init()`.
+// `_start()` then calls `runtime_init::init()`, which on completion, jumps to `kernel_init()`.
 mod runtime_init;

 // Conditionally includes the selected `BSP` code.
@@ -68,50 +64,25 @@

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
-    // Notify raspbootcom to send the binary.
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
-    print!("OK");
-
-    let kernel_addr: *mut u8 = bsp::BOARD_DEFAULT_LOAD_ADDRESS as *mut u8;
-    unsafe {
-        // Read the kernel byte by byte.
-        for i in 0..size {
-            *kernel_addr.offset(i as isize) = bsp::console().read_char() as u8;
-        }
+    println!("Booting on: {}", bsp::board_name());
+    println!(
+        "Architectural timer resolution: {} ns",
+        arch::timer().resolution().as_nanos()
+    );
+
+    println!("Drivers loaded:");
+    for (i, driver) in bsp::device_drivers().iter().enumerate() {
+        println!("      {}. {}", i + 1, driver.compatible());
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
+        println!("Spinning for 1 second");
+        arch::timer().spin_for(Duration::from_secs(1));
+    }
 }

diff -uNr 07_uart_chainloader/src/print.rs 08_timestamps/src/print.rs
--- 07_uart_chainloader/src/print.rs
+++ 08_timestamps/src/print.rs
@@ -16,13 +16,71 @@
 }

 /// Prints with a newline.
-///
-/// Carbon copy from https://doc.rust-lang.org/src/std/macros.rs.html
 #[macro_export]
 macro_rules! println {
     () => ($crate::print!("\n"));
-    ($($arg:tt)*) => ({
-        $crate::print::_print(format_args_nl!($($arg)*));
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
     })
 }


diff -uNr 07_uart_chainloader/src/relocate.rs 08_timestamps/src/relocate.rs
--- 07_uart_chainloader/src/relocate.rs
+++ 08_timestamps/src/relocate.rs
@@ -1,46 +0,0 @@
-// SPDX-License-Identifier: MIT
-//
-// Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>
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
-    crate::runtime_init::get().init()
-}

diff -uNr 07_uart_chainloader/src/runtime_init.rs 08_timestamps/src/runtime_init.rs
--- 07_uart_chainloader/src/runtime_init.rs
+++ 08_timestamps/src/runtime_init.rs
@@ -4,39 +4,21 @@

 //! Rust runtime initialization code.

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
-    unsafe fn init(&self) -> ! {
-        extern "C" {
-            // Boundaries of the .bss section, provided by the linker script.
-            static mut __bss_start: u64;
-            static mut __bss_end: u64;
-        }
-
-        // Zero out the .bss section.
-        r0::zero_bss(&mut __bss_start, &mut __bss_end);
-
-        crate::kernel_init()
+/// # Safety
+///
+/// - Only a single core must be active and running this function.
+pub unsafe fn init() -> ! {
+    extern "C" {
+        // Boundaries of the .bss section, provided by the linker script.
+        static mut __bss_start: u64;
+        static mut __bss_end: u64;
     }
-}

-struct Traitor;
-impl RunTimeInit for Traitor {}
+    // Zero out the .bss section.
+    r0::zero_bss(&mut __bss_start, &mut __bss_end);

-/// Give the callee a `RunTimeInit` trait object.
-pub fn get() -> &'static dyn RunTimeInit {
-    &Traitor {}
+    crate::kernel_init()
 }

```
