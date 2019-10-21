# Tutorial 06 - Drivers: GPIO and UART

## tl;dr

Now that we enabled safe globals in the previous tutorial, the infrastructure is
laid for adding the first real device drivers. We throw out the magic QEMU
console and use a real UART now. Like serious embedded hackers do!

- A `DeviceDriver` trait is added for abstracting `BSP` driver implementations
  from kernel code.
- Drivers are stored in `bsp/driver`, and can be reused between `BSP`s.
    - Introducing the `GPIO` driver, which pinmuxes the RPi's PL011 UART.
    - Most importantly, the `PL011Uart` driver: It implements the `Console`
      traits and is from now on used as the system console output.
- `BSP`s now contain a`memory_map.rs`. In the specific case, they contain the
  RPi's MMIO addresses which are used to instantiate compatible device drivers
  from `bsp/driver`.

## Boot it from SD card

1. Make a single `FAT32` partition named `boot`.
2. Copy [bootcode.bin](https://github.com/raspberrypi/firmware/raw/master/boot/bootcode.bin), [fixup.dat](https://github.com/raspberrypi/firmware/raw/master/boot/fixup.dat) and [start.elf](https://github.com/raspberrypi/firmware/raw/master/boot/start.elf) from the [Raspberry Pi firmware repo](https://github.com/raspberrypi/firmware/tree/master/boot) onto the SD card.
3. Copy our [kernel8.img](kernel8.img) onto the SD card.
4. Insert the SD card into the RPi and connect the USB serial to your host PC.
    - Wiring diagram at [top-level README](../README.md#usb-serial).
5. Run `screen` (you might need to install it first):

```console
sudo screen /dev/ttyUSB0 115200
```

6. Hit <kbd>Enter</kbd> to kick off the kernel boot process.
7. Exit screen by pressing <kbd>ctrl-a</kbd> <kbd>ctrl-d</kbd> or disconnecting the USB serial.

## Diff to previous
```diff

diff -uNr 05_safe_globals/Cargo.toml 06_drivers_gpio_uart/Cargo.toml
--- 05_safe_globals/Cargo.toml
+++ 06_drivers_gpio_uart/Cargo.toml
@@ -10,10 +10,12 @@
 # The features section is used to select the target board.
 [features]
 default = []
-bsp_rpi3 = ["cortex-a"]
+bsp_rpi3 = ["cortex-a", "register"]
+bsp_rpi4 = ["cortex-a", "register"]

 [dependencies]
 r0 = "0.2.*"

 # Optional dependencies
 cortex-a = { version = "2.*", optional = true }
+register = { version = "0.3.*", optional = true }

diff -uNr 05_safe_globals/Makefile 06_drivers_gpio_uart/Makefile
--- 05_safe_globals/Makefile
+++ 06_drivers_gpio_uart/Makefile
@@ -16,6 +16,14 @@
 	QEMU_MISC_ARGS = -serial stdio
 	LINKER_FILE = src/bsp/rpi/link.ld
 	RUSTC_MISC_ARGS = -C target-cpu=cortex-a53
+else ifeq ($(BSP),rpi4)
+	TARGET = aarch64-unknown-none-softfloat
+	OUTPUT = kernel8.img
+#	QEMU_BINARY = qemu-system-aarch64
+#	QEMU_MACHINE_TYPE =
+#	QEMU_MISC_ARGS = -serial stdio
+	LINKER_FILE = src/bsp/rpi/link.ld
+	RUSTC_MISC_ARGS = -C target-cpu=cortex-a72
 endif

 SOURCES = $(wildcard **/*.rs) $(wildcard **/*.S) $(wildcard **/*.ld)
@@ -56,9 +64,14 @@
 	cargo xdoc --target=$(TARGET) --features bsp_$(BSP) --document-private-items
 	xdg-open target/$(TARGET)/doc/kernel/index.html

+ifeq ($(QEMU_MACHINE_TYPE),)
+$(info This board is not yet supported for QEMU.)
+qemu:
+else
 qemu: all
 	$(DOCKER_CMD) $(DOCKER_ARG_CURDIR) $(CONTAINER_UTILS) \
 	$(DOCKER_EXEC_QEMU) $(QEMU_MISC_ARGS)
+endif

 clippy:
 	cargo xclippy --target=$(TARGET) --features bsp_$(BSP)

diff -uNr 05_safe_globals/src/arch/aarch64.rs 06_drivers_gpio_uart/src/arch/aarch64.rs
--- 05_safe_globals/src/arch/aarch64.rs
+++ 06_drivers_gpio_uart/src/arch/aarch64.rs
@@ -33,6 +33,15 @@
 // Implementation of the kernel's architecture abstraction code
 //--------------------------------------------------------------------------------------------------

+pub use asm::nop;
+
+/// Spin for `n` cycles.
+pub fn spin_for_cycles(n: usize) {
+    for _ in 0..n {
+        asm::nop();
+    }
+}
+
 /// Pause execution on the calling CPU core.
 #[inline(always)]
 pub fn wait_forever() -> ! {

diff -uNr 05_safe_globals/src/arch.rs 06_drivers_gpio_uart/src/arch.rs
--- 05_safe_globals/src/arch.rs
+++ 06_drivers_gpio_uart/src/arch.rs
@@ -4,8 +4,8 @@

 //! Conditional exporting of processor architecture code.

-#[cfg(feature = "bsp_rpi3")]
+#[cfg(any(feature = "bsp_rpi3", feature = "bsp_rpi4"))]
 mod aarch64;

-#[cfg(feature = "bsp_rpi3")]
+#[cfg(any(feature = "bsp_rpi3", feature = "bsp_rpi4"))]
 pub use aarch64::*;

diff -uNr 05_safe_globals/src/bsp/driver/bcm/bcm2xxx_gpio.rs 06_drivers_gpio_uart/src/bsp/driver/bcm/bcm2xxx_gpio.rs
--- 05_safe_globals/src/bsp/driver/bcm/bcm2xxx_gpio.rs
+++ 06_drivers_gpio_uart/src/bsp/driver/bcm/bcm2xxx_gpio.rs
@@ -0,0 +1,158 @@
+// SPDX-License-Identifier: MIT
+//
+// Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>
+
+//! GPIO driver.
+
+use crate::{arch, arch::sync::NullLock, interface};
+use core::ops;
+use register::{mmio::ReadWrite, register_bitfields};
+
+// GPIO registers.
+//
+// Descriptions taken from
+// https://github.com/raspberrypi/documentation/files/1888662/BCM2837-ARM-Peripherals.-.Revised.-.V2-1.pdf
+register_bitfields! {
+    u32,
+
+    /// GPIO Function Select 1
+    GPFSEL1 [
+        /// Pin 15
+        FSEL15 OFFSET(15) NUMBITS(3) [
+            Input = 0b000,
+            Output = 0b001,
+            AltFunc0 = 0b100  // PL011 UART RX
+
+        ],
+
+        /// Pin 14
+        FSEL14 OFFSET(12) NUMBITS(3) [
+            Input = 0b000,
+            Output = 0b001,
+            AltFunc0 = 0b100  // PL011 UART TX
+        ]
+    ],
+
+    /// GPIO Pull-up/down Clock Register 0
+    GPPUDCLK0 [
+        /// Pin 15
+        PUDCLK15 OFFSET(15) NUMBITS(1) [
+            NoEffect = 0,
+            AssertClock = 1
+        ],
+
+        /// Pin 14
+        PUDCLK14 OFFSET(14) NUMBITS(1) [
+            NoEffect = 0,
+            AssertClock = 1
+        ]
+    ]
+}
+
+#[allow(non_snake_case)]
+#[repr(C)]
+pub struct RegisterBlock {
+    pub GPFSEL0: ReadWrite<u32>,                        // 0x00
+    pub GPFSEL1: ReadWrite<u32, GPFSEL1::Register>,     // 0x04
+    pub GPFSEL2: ReadWrite<u32>,                        // 0x08
+    pub GPFSEL3: ReadWrite<u32>,                        // 0x0C
+    pub GPFSEL4: ReadWrite<u32>,                        // 0x10
+    pub GPFSEL5: ReadWrite<u32>,                        // 0x14
+    __reserved_0: u32,                                  // 0x18
+    GPSET0: ReadWrite<u32>,                             // 0x1C
+    GPSET1: ReadWrite<u32>,                             // 0x20
+    __reserved_1: u32,                                  //
+    GPCLR0: ReadWrite<u32>,                             // 0x28
+    __reserved_2: [u32; 2],                             //
+    GPLEV0: ReadWrite<u32>,                             // 0x34
+    GPLEV1: ReadWrite<u32>,                             // 0x38
+    __reserved_3: u32,                                  //
+    GPEDS0: ReadWrite<u32>,                             // 0x40
+    GPEDS1: ReadWrite<u32>,                             // 0x44
+    __reserved_4: [u32; 7],                             //
+    GPHEN0: ReadWrite<u32>,                             // 0x64
+    GPHEN1: ReadWrite<u32>,                             // 0x68
+    __reserved_5: [u32; 10],                            //
+    pub GPPUD: ReadWrite<u32>,                          // 0x94
+    pub GPPUDCLK0: ReadWrite<u32, GPPUDCLK0::Register>, // 0x98
+    pub GPPUDCLK1: ReadWrite<u32>,                      // 0x9C
+}
+
+/// The driver's private data.
+struct GPIOInner {
+    base_addr: usize,
+}
+
+/// Deref to RegisterBlock.
+impl ops::Deref for GPIOInner {
+    type Target = RegisterBlock;
+
+    fn deref(&self) -> &Self::Target {
+        unsafe { &*self.ptr() }
+    }
+}
+
+impl GPIOInner {
+    const fn new(base_addr: usize) -> GPIOInner {
+        GPIOInner { base_addr }
+    }
+
+    /// Return a pointer to the register block.
+    fn ptr(&self) -> *const RegisterBlock {
+        self.base_addr as *const _
+    }
+
+    /// Map PL011 UART as standard output.
+    ///
+    /// TX to pin 14
+    /// RX to pin 15
+    pub fn map_pl011_uart(&mut self) {
+        // Map to pins.
+        self.GPFSEL1
+            .modify(GPFSEL1::FSEL14::AltFunc0 + GPFSEL1::FSEL15::AltFunc0);
+
+        // Enable pins 14 and 15.
+        self.GPPUD.set(0);
+        arch::spin_for_cycles(150);
+
+        self.GPPUDCLK0
+            .write(GPPUDCLK0::PUDCLK14::AssertClock + GPPUDCLK0::PUDCLK15::AssertClock);
+        arch::spin_for_cycles(150);
+
+        self.GPPUDCLK0.set(0);
+    }
+}
+
+//--------------------------------------------------------------------------------------------------
+// BSP-public
+//--------------------------------------------------------------------------------------------------
+use interface::sync::Mutex;
+
+/// The driver's main struct.
+pub struct GPIO {
+    inner: NullLock<GPIOInner>,
+}
+
+impl GPIO {
+    pub const unsafe fn new(base_addr: usize) -> GPIO {
+        GPIO {
+            inner: NullLock::new(GPIOInner::new(base_addr)),
+        }
+    }
+
+    // Only visible to other BSP code.
+    pub fn map_pl011_uart(&self) {
+        let mut r = &self.inner;
+        r.lock(|inner| inner.map_pl011_uart());
+    }
+}
+
+//--------------------------------------------------------------------------------------------------
+// OS interface implementations
+//--------------------------------------------------------------------------------------------------
+
+impl interface::driver::DeviceDriver for GPIO {
+    fn compatible(&self) -> &str {
+        "GPIO"
+    }
+}

diff -uNr 05_safe_globals/src/bsp/driver/bcm/bcm2xxx_pl011_uart.rs 06_drivers_gpio_uart/src/bsp/driver/bcm/bcm2xxx_pl011_uart.rs
--- 05_safe_globals/src/bsp/driver/bcm/bcm2xxx_pl011_uart.rs
+++ 06_drivers_gpio_uart/src/bsp/driver/bcm/bcm2xxx_pl011_uart.rs
@@ -0,0 +1,308 @@
+// SPDX-License-Identifier: MIT
+//
+// Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>
+
+//! PL011 UART driver.
+
+use crate::{arch, arch::sync::NullLock, interface};
+use core::{fmt, ops};
+use register::{mmio::*, register_bitfields};
+
+// PL011 UART registers.
+//
+// Descriptions taken from
+// https://github.com/raspberrypi/documentation/files/1888662/BCM2837-ARM-Peripherals.-.Revised.-.V2-1.pdf
+register_bitfields! {
+    u32,
+
+    /// Flag Register
+    FR [
+        /// Transmit FIFO empty. The meaning of this bit depends on the state of the FEN bit in the
+        /// Line Control Register, UARTLCR_ LCRH.
+        ///
+        /// If the FIFO is disabled, this bit is set when the transmit holding register is empty. If
+        /// the FIFO is enabled, the TXFE bit is set when the transmit FIFO is empty. This bit does
+        /// not indicate if there is data in the transmit shift register.
+        TXFE OFFSET(7) NUMBITS(1) [],
+
+        /// Transmit FIFO full. The meaning of this bit depends on the state of the FEN bit in the
+        /// UARTLCR_ LCRH Register.
+        ///
+        /// If the FIFO is disabled, this bit is set when the transmit holding register is full. If
+        /// the FIFO is enabled, the TXFF bit is set when the transmit FIFO is full.
+        TXFF OFFSET(5) NUMBITS(1) [],
+
+        /// Receive FIFO empty. The meaning of this bit depends on the state of the FEN bit in the
+        /// UARTLCR_H Register.
+        ///
+        /// If the FIFO is disabled, this bit is set when the receive holding register is empty. If
+        /// the FIFO is enabled, the RXFE bit is set when the receive FIFO is empty.
+        RXFE OFFSET(4) NUMBITS(1) []
+    ],
+
+    /// Integer Baud rate divisor
+    IBRD [
+        /// Integer Baud rate divisor
+        IBRD OFFSET(0) NUMBITS(16) []
+    ],
+
+    /// Fractional Baud rate divisor
+    FBRD [
+        /// Fractional Baud rate divisor
+        FBRD OFFSET(0) NUMBITS(6) []
+    ],
+
+    /// Line Control register
+    LCRH [
+        /// Word length. These bits indicate the number of data bits transmitted or received in a
+        /// frame.
+        WLEN OFFSET(5) NUMBITS(2) [
+            FiveBit = 0b00,
+            SixBit = 0b01,
+            SevenBit = 0b10,
+            EightBit = 0b11
+        ],
+
+        /// Enable FIFOs:
+        ///
+        /// 0 = FIFOs are disabled (character mode) that is, the FIFOs become 1-byte-deep holding
+        /// registers
+        ///
+        /// 1 = transmit and receive FIFO buffers are enabled (FIFO mode).
+        FEN  OFFSET(4) NUMBITS(1) [
+            FifosDisabled = 0,
+            FifosEnabled = 1
+        ]
+    ],
+
+    /// Control Register
+    CR [
+        /// Receive enable. If this bit is set to 1, the receive section of the UART is enabled.
+        /// Data reception occurs for UART signals. When the UART is disabled in the middle of
+        /// reception, it completes the current character before stopping.
+        RXE    OFFSET(9) NUMBITS(1) [
+            Disabled = 0,
+            Enabled = 1
+        ],
+
+        /// Transmit enable. If this bit is set to 1, the transmit section of the UART is enabled.
+        /// Data transmission occurs for UART signals. When the UART is disabled in the middle of
+        /// transmission, it completes the current character before stopping.
+        TXE    OFFSET(8) NUMBITS(1) [
+            Disabled = 0,
+            Enabled = 1
+        ],
+
+        /// UART enable
+        UARTEN OFFSET(0) NUMBITS(1) [
+            /// If the UART is disabled in the middle of transmission or reception, it completes the
+            /// current character before stopping.
+            Disabled = 0,
+            Enabled = 1
+        ]
+    ],
+
+    /// Interupt Clear Register
+    ICR [
+        /// Meta field for all pending interrupts
+        ALL OFFSET(0) NUMBITS(11) []
+    ]
+}
+
+#[allow(non_snake_case)]
+#[repr(C)]
+pub struct RegisterBlock {
+    DR: ReadWrite<u32>,                   // 0x00
+    __reserved_0: [u32; 5],               // 0x04
+    FR: ReadOnly<u32, FR::Register>,      // 0x18
+    __reserved_1: [u32; 2],               // 0x1c
+    IBRD: WriteOnly<u32, IBRD::Register>, // 0x24
+    FBRD: WriteOnly<u32, FBRD::Register>, // 0x28
+    LCRH: WriteOnly<u32, LCRH::Register>, // 0x2C
+    CR: WriteOnly<u32, CR::Register>,     // 0x30
+    __reserved_2: [u32; 4],               // 0x34
+    ICR: WriteOnly<u32, ICR::Register>,   // 0x44
+}
+
+/// The driver's mutex protected part.
+struct PL011UartInner {
+    base_addr: usize,
+    chars_written: usize,
+}
+
+/// Deref to RegisterBlock.
+///
+/// Allows writing
+/// ```
+/// self.DR.read()
+/// ```
+/// instead of something along the lines of
+/// ```
+/// unsafe { (*PL011UartInner::ptr()).DR.read() }
+/// ```
+impl ops::Deref for PL011UartInner {
+    type Target = RegisterBlock;
+
+    fn deref(&self) -> &Self::Target {
+        unsafe { &*self.ptr() }
+    }
+}
+
+impl PL011UartInner {
+    const fn new(base_addr: usize) -> PL011UartInner {
+        PL011UartInner {
+            base_addr,
+            chars_written: 0,
+        }
+    }
+
+    /// Return a pointer to the register block.
+    fn ptr(&self) -> *const RegisterBlock {
+        self.base_addr as *const _
+    }
+
+    /// Send a character.
+    fn write_char(&mut self, c: char) {
+        // Wait until we can send.
+        loop {
+            if !self.FR.is_set(FR::TXFF) {
+                break;
+            }
+
+            arch::nop();
+        }
+
+        // Write the character to the buffer.
+        self.DR.set(c as u32);
+    }
+}
+
+/// Implementing `core::fmt::Write` enables usage of the `format_args!` macros, which in turn are
+/// used to implement the `kernel`'s `print!` and `println!` macros. By implementing `write_str()`,
+/// we get `write_fmt()` automatically.
+///
+/// The function takes an `&mut self`, so it must be implemented for the inner struct.
+///
+/// See [`src/print.rs`].
+///
+/// [`src/print.rs`]: ../../print/index.html
+impl fmt::Write for PL011UartInner {
+    fn write_str(&mut self, s: &str) -> fmt::Result {
+        for c in s.chars() {
+            // Convert newline to carrige return + newline.
+            if c == '
' {
+                self.write_char('')
+            }
+
+            self.write_char(c);
+        }
+
+        self.chars_written += s.len();
+
+        Ok(())
+    }
+}
+
+//--------------------------------------------------------------------------------------------------
+// BSP-public
+//--------------------------------------------------------------------------------------------------
+
+/// The driver's main struct.
+pub struct PL011Uart {
+    inner: NullLock<PL011UartInner>,
+}
+
+impl PL011Uart {
+    /// # Safety
+    ///
+    /// The user must ensure to provide the correct `base_addr`.
+    pub const unsafe fn new(base_addr: usize) -> PL011Uart {
+        PL011Uart {
+            inner: NullLock::new(PL011UartInner::new(base_addr)),
+        }
+    }
+}
+
+//--------------------------------------------------------------------------------------------------
+// OS interface implementations
+//--------------------------------------------------------------------------------------------------
+use interface::sync::Mutex;
+
+impl interface::driver::DeviceDriver for PL011Uart {
+    fn compatible(&self) -> &str {
+        "PL011Uart"
+    }
+
+    /// Set up baud rate and characteristics
+    ///
+    /// Results in 8N1 and 115200 baud (if the clk has been previously set to 4 MHz by the
+    /// firmware).
+    fn init(&self) -> interface::driver::Result {
+        let mut r = &self.inner;
+        r.lock(|inner| {
+            // Turn it off temporarily.
+            inner.CR.set(0);
+
+            inner.ICR.write(ICR::ALL::CLEAR);
+            inner.IBRD.write(IBRD::IBRD.val(26)); // Results in 115200 baud for UART Clk of 48 MHz.
+            inner.FBRD.write(FBRD::FBRD.val(3));
+            inner
+                .LCRH
+                .write(LCRH::WLEN::EightBit + LCRH::FEN::FifosEnabled); // 8N1 + Fifo on
+            inner
+                .CR
+                .write(CR::UARTEN::Enabled + CR::TXE::Enabled + CR::RXE::Enabled);
+        });
+
+        Ok(())
+    }
+}
+
+impl interface::console::Write for PL011Uart {
+    /// Passthrough of `args` to the `core::fmt::Write` implementation, but guarded by a Mutex to
+    /// serialize access.
+    fn write_char(&self, c: char) {
+        let mut r = &self.inner;
+        r.lock(|inner| inner.write_char(c));
+    }
+
+    fn write_fmt(&self, args: core::fmt::Arguments) -> fmt::Result {
+        // Fully qualified syntax for the call to `core::fmt::Write::write:fmt()` to increase
+        // readability.
+        let mut r = &self.inner;
+        r.lock(|inner| fmt::Write::write_fmt(inner, args))
+    }
+}
+
+impl interface::console::Read for PL011Uart {
+    fn read_char(&self) -> char {
+        let mut r = &self.inner;
+        r.lock(|inner| {
+            // Wait until buffer is filled.
+            loop {
+                if !inner.FR.is_set(FR::RXFE) {
+                    break;
+                }
+
+                arch::nop();
+            }
+
+            // Read one character.
+            let mut ret = inner.DR.get() as u8 as char;
+
+            // Convert carrige return to newline.
+            if ret == '' {
+                ret = '
'
+            }
+
+            ret
+        })
+    }
+}
+
+impl interface::console::Statistics for PL011Uart {
+    fn chars_written(&self) -> usize {
+        let mut r = &self.inner;
+        r.lock(|inner| inner.chars_written)
+    }
+}

diff -uNr 05_safe_globals/src/bsp/driver/bcm.rs 06_drivers_gpio_uart/src/bsp/driver/bcm.rs
--- 05_safe_globals/src/bsp/driver/bcm.rs
+++ 06_drivers_gpio_uart/src/bsp/driver/bcm.rs
@@ -0,0 +1,11 @@
+// SPDX-License-Identifier: MIT
+//
+// Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>
+
+//! BCM driver top level.
+
+mod bcm2xxx_gpio;
+mod bcm2xxx_pl011_uart;
+
+pub use bcm2xxx_gpio::GPIO;
+pub use bcm2xxx_pl011_uart::PL011Uart;

diff -uNr 05_safe_globals/src/bsp/driver.rs 06_drivers_gpio_uart/src/bsp/driver.rs
--- 05_safe_globals/src/bsp/driver.rs
+++ 06_drivers_gpio_uart/src/bsp/driver.rs
@@ -0,0 +1,11 @@
+// SPDX-License-Identifier: MIT
+//
+// Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>
+
+//! Drivers.
+
+#[cfg(any(feature = "bsp_rpi3", feature = "bsp_rpi4"))]
+mod bcm;
+
+#[cfg(any(feature = "bsp_rpi3", feature = "bsp_rpi4"))]
+pub use bcm::*;

diff -uNr 05_safe_globals/src/bsp/rpi/memory_map.rs 06_drivers_gpio_uart/src/bsp/rpi/memory_map.rs
--- 05_safe_globals/src/bsp/rpi/memory_map.rs
+++ 06_drivers_gpio_uart/src/bsp/rpi/memory_map.rs
@@ -0,0 +1,18 @@
+// SPDX-License-Identifier: MIT
+//
+// Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>
+
+//! The board's memory map.
+
+/// Physical devices.
+#[rustfmt::skip]
+pub mod mmio {
+    #[cfg(feature = "bsp_rpi3")]
+    pub const BASE:            usize =        0x3F00_0000;
+
+    #[cfg(feature = "bsp_rpi4")]
+    pub const BASE:            usize =        0xFE00_0000;
+
+    pub const GPIO_BASE:       usize = BASE + 0x0020_0000;
+    pub const PL011_UART_BASE: usize = BASE + 0x0020_1000;
+}

diff -uNr 05_safe_globals/src/bsp/rpi.rs 06_drivers_gpio_uart/src/bsp/rpi.rs
--- 05_safe_globals/src/bsp/rpi.rs
+++ 06_drivers_gpio_uart/src/bsp/rpi.rs
@@ -4,114 +4,55 @@

 //! Board Support Package for the Raspberry Pi.

-use crate::{arch::sync::NullLock, interface};
-use core::fmt;
+mod memory_map;
+
+use super::driver;
+use crate::interface;

 pub const BOOT_CORE_ID: u64 = 0;
 pub const BOOT_CORE_STACK_START: u64 = 0x80_000;

-/// A mystical, magical device for generating QEMU output out of the void.
-///
-/// The mutex protected part.
-struct QEMUOutputInner {
-    chars_written: usize,
-}
-
-impl QEMUOutputInner {
-    const fn new() -> QEMUOutputInner {
-        QEMUOutputInner { chars_written: 0 }
-    }
-
-    /// Send a character.
-    fn write_char(&mut self, c: char) {
-        unsafe {
-            core::ptr::write_volatile(0x3F20_1000 as *mut u8, c as u8);
-        }
-    }
-}
-
-/// Implementing `core::fmt::Write` enables usage of the `format_args!` macros, which in turn are
-/// used to implement the `kernel`'s `print!` and `println!` macros. By implementing `write_str()`,
-/// we get `write_fmt()` automatically.
-///
-/// The function takes an `&mut self`, so it must be implemented for the inner struct.
-///
-/// See [`src/print.rs`].
-///
-/// [`src/print.rs`]: ../../print/index.html
-impl fmt::Write for QEMUOutputInner {
-    fn write_str(&mut self, s: &str) -> fmt::Result {
-        for c in s.chars() {
-            // Convert newline to carrige return + newline.
-            if c == '
' {
-                self.write_char('')
-            }
-
-            self.write_char(c);
-        }
-
-        self.chars_written += s.len();
-
-        Ok(())
-    }
-}
-
 //--------------------------------------------------------------------------------------------------
-// BSP-public
+// Global BSP driver instances
 //--------------------------------------------------------------------------------------------------

-/// The main struct.
-pub struct QEMUOutput {
-    inner: NullLock<QEMUOutputInner>,
-}
-
-impl QEMUOutput {
-    pub const fn new() -> QEMUOutput {
-        QEMUOutput {
-            inner: NullLock::new(QEMUOutputInner::new()),
-        }
-    }
-}
+static GPIO: driver::GPIO = unsafe { driver::GPIO::new(memory_map::mmio::GPIO_BASE) };
+static PL011_UART: driver::PL011Uart =
+    unsafe { driver::PL011Uart::new(memory_map::mmio::PL011_UART_BASE) };

 //--------------------------------------------------------------------------------------------------
-// OS interface implementations
+// Implementation of the kernel's BSP calls
 //--------------------------------------------------------------------------------------------------

-/// Passthrough of `args` to the `core::fmt::Write` implementation, but guarded by a Mutex to
-/// serialize access.
-impl interface::console::Write for QEMUOutput {
-    fn write_fmt(&self, args: core::fmt::Arguments) -> fmt::Result {
-        use interface::sync::Mutex;
-
-        // Fully qualified syntax for the call to `core::fmt::Write::write:fmt()` to increase
-        // readability.
-        let mut r = &self.inner;
-        r.lock(|inner| fmt::Write::write_fmt(inner, args))
+/// Board identification.
+pub fn board_name() -> &'static str {
+    #[cfg(feature = "bsp_rpi3")]
+    {
+        "Raspberry Pi 3"
     }
-}
-
-impl interface::console::Read for QEMUOutput {}

-impl interface::console::Statistics for QEMUOutput {
-    fn chars_written(&self) -> usize {
-        use interface::sync::Mutex;
-
-        let mut r = &self.inner;
-        r.lock(|inner| inner.chars_written)
+    #[cfg(feature = "bsp_rpi4")]
+    {
+        "Raspberry Pi 4"
     }
 }

-//--------------------------------------------------------------------------------------------------
-// Global instances
-//--------------------------------------------------------------------------------------------------
-
-static QEMU_OUTPUT: QEMUOutput = QEMUOutput::new();
-
-//--------------------------------------------------------------------------------------------------
-// Implementation of the kernel's BSP calls
-//--------------------------------------------------------------------------------------------------
-
 /// Return a reference to a `console::All` implementation.
 pub fn console() -> &'static impl interface::console::All {
-    &QEMU_OUTPUT
+    &PL011_UART
+}
+
+/// Return an array of references to all `DeviceDriver` compatible `BSP` drivers.
+///
+/// # Safety
+///
+/// The order of devices is the order in which `DeviceDriver::init()` is called.
+pub fn device_drivers() -> [&'static dyn interface::driver::DeviceDriver; 2] {
+    [&GPIO, &PL011_UART]
+}
+
+/// BSP initialization code that runs after driver init.
+pub fn post_driver_init() {
+    // Configure PL011Uart's output pins.
+    GPIO.map_pl011_uart();
 }

diff -uNr 05_safe_globals/src/bsp.rs 06_drivers_gpio_uart/src/bsp.rs
--- 05_safe_globals/src/bsp.rs
+++ 06_drivers_gpio_uart/src/bsp.rs
@@ -4,8 +4,10 @@

 //! Conditional exporting of Board Support Packages.

-#[cfg(feature = "bsp_rpi3")]
+mod driver;
+
+#[cfg(any(feature = "bsp_rpi3", feature = "bsp_rpi4"))]
 mod rpi;

-#[cfg(feature = "bsp_rpi3")]
+#[cfg(any(feature = "bsp_rpi3", feature = "bsp_rpi4"))]
 pub use rpi::*;

diff -uNr 05_safe_globals/src/interface.rs 06_drivers_gpio_uart/src/interface.rs
--- 05_safe_globals/src/interface.rs
+++ 06_drivers_gpio_uart/src/interface.rs
@@ -24,6 +24,7 @@

     /// Console write functions.
     pub trait Write {
+        fn write_char(&self, c: char);
         fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result;
     }

@@ -83,3 +84,20 @@
         fn lock<R>(&mut self, f: impl FnOnce(&mut Self::Data) -> R) -> R;
     }
 }
+
+/// Driver interfaces.
+pub mod driver {
+    /// Driver result type, e.g. for indicating successful driver init.
+    pub type Result = core::result::Result<(), ()>;
+
+    /// Device Driver functions.
+    pub trait DeviceDriver {
+        /// Return a compatibility string for identifying the driver.
+        fn compatible(&self) -> &str;
+
+        /// Called by the kernel to bring up the device.
+        fn init(&self) -> Result {
+            Ok(())
+        }
+    }
+}

diff -uNr 05_safe_globals/src/main.rs 06_drivers_gpio_uart/src/main.rs
--- 05_safe_globals/src/main.rs
+++ 06_drivers_gpio_uart/src/main.rs
@@ -41,16 +41,50 @@

 /// Early init code.
 ///
+/// Concerned with with initializing `BSP` and `arch` parts.
+///
 /// # Safety
 ///
 /// - Only a single core must be active and running this function.
+/// - The init calls in this function must appear in the correct order.
 unsafe fn kernel_init() -> ! {
-    use interface::console::Statistics;
+    for i in bsp::device_drivers().iter() {
+        if let Err(()) = i.init() {
+            // This message will only be readable if, at the time of failure, the return value of
+            // `bsp::console()` is already in functioning state.
+            panic!("Error loading driver: {}", i.compatible())
+        }
+    }
+
+    bsp::post_driver_init();
+
+    // Transition from unsafe to safe.
+    kernel_main()
+}
+
+/// The main function running after the early init.
+fn kernel_main() -> ! {
+    use interface::console::All;
+
+    // UART should be functional now. Wait for user to hit Enter.
+    loop {
+        if bsp::console().read_char() == '
' {
+            break;
+        }
+    }
+
+    println!("[0] Booting on: {}", bsp::board_name());

-    println!("[0] Hello from pure Rust!");
+    println!("[1] Drivers loaded:");
+    for (i, driver) in bsp::device_drivers().iter().enumerate() {
+        println!("      {}. {}", i + 1, driver.compatible());
+    }

-    println!("[1] Chars written: {}", bsp::console().chars_written());
+    println!("[2] Chars written: {}", bsp::console().chars_written());
+    println!("[3] Echoing input now");

-    println!("[2] Stopping here.");
-    arch::wait_forever()
+    loop {
+        let c = bsp::console().read_char();
+        bsp::console().write_char(c);
+    }
 }
```
