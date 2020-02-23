# Tutorial 06 - Drivers: GPIO and UART

## tl;dr

Now that we enabled safe globals in the previous tutorial, the infrastructure is
laid for adding the first real device drivers. We throw out the magic QEMU
console and use a real UART now. Like serious embedded hackers do!

- For the first time, we will be able to run the code on the real hardware.
  - Therefore, building is now differentiated between the **RPi 3** and the **RPi4**.
  - By default, all `Makefile` targets will build for the **RPi 3**.
  - In order to build for the the **RPi4**, prepend `BSP=rpi4` to each target. For example:
    - `BSP=rpi4 make`
    - `BSP=rpi4 make doc`
  - Unfortunately, QEMU does not yet support the **RPi4**, so `BSP=rpi4 make qemu` won't work.
- A `DeviceDriver` trait is added for abstracting `BSP` driver implementations
  from kernel code.
- Drivers are stored in `bsp/driver`, and can be reused between `BSP`s.
    - Introducing the `GPIO` driver, which pinmuxes the RPi's PL011 UART.
    - Most importantly, the `PL011Uart` driver: It implements the `Console`
      traits and is from now on used as the system console output.
- `BSP`s now contain a`memory_map.rs`. In the specific case, they contain the
  RPi's MMIO addresses which are used to instantiate compatible device drivers
  from `bsp/driver`.
- We also modify the `panic!` handler, so that it does not anymore rely on `println!`, which uses
  the globally-shared instance of the `UART` that might be locked when an error is encountered (for
  now this can't happen due to the `NullLock`, but with a real lock it becomes an issue).
    - Instead, it creates a new UART driver instance, re-initializes the device and uses that one to
      print. This increases the chances that the system is able to print a final important message
      before it suspends itself.

## Boot it from SD card

Some steps for preparing the SD card differ between RPi3 and RPi4, so be careful.

### Common for both

1. Make a single `FAT32` partition named `boot`.
2. On the card, generate a file named `config.txt` with the following contents:

```txt
init_uart_clock=48000000
```
### Pi 3

3. Copy the following files from the [Raspberry Pi firmware repo](https://github.com/raspberrypi/firmware/tree/master/boot)  onto the SD card:
    - [bootcode.bin](https://github.com/raspberrypi/firmware/raw/master/boot/bootcode.bin)
    - [fixup.dat](https://github.com/raspberrypi/firmware/raw/master/boot/fixup.dat)
    - [start.elf](https://github.com/raspberrypi/firmware/raw/master/boot/start.elf)
4. Run `make` and copy the [kernel8.img](kernel8.img) onto the SD card.

### Pi 4

3. Copy the following files from the [Raspberry Pi firmware repo](https://github.com/raspberrypi/firmware/tree/master/boot)  onto the SD card:
    - [fixup4.dat](https://github.com/raspberrypi/firmware/raw/master/boot/fixup4.dat)
    - [start4.elf](https://github.com/raspberrypi/firmware/raw/master/boot/start4.elf)
    - [bcm2711-rpi-4-b.dtb](https://github.com/raspberrypi/firmware/raw/master/boot/bcm2711-rpi-4-b.dtb)
4. Run `BSP=rpi4 make` and copy the [kernel8.img](kernel8.img) onto the SD card.


_**Note**: Should it not work on your RPi4, try renaming `start4.elf` to `start.elf` (without the 4) on the SD card._


### Common again

5. Insert the SD card into the RPi and connect the USB serial to your host PC.
    - Wiring diagram at [top-level README](../README.md#usb-serial).
6. Run `screen` (you might need to install it first):

```console
sudo screen /dev/ttyUSB0 230400
```

7. Hit <kbd>Enter</kbd> to kick off the kernel boot process. Observe the output:

```console
[0] Booting on: Raspberry Pi 3
[1] Drivers loaded:
      1. GPIO
      2. PL011Uart
[2] Chars written: 84
[3] Echoing input now
```

8. Exit screen by pressing <kbd>ctrl-a</kbd> <kbd>ctrl-d</kbd> or disconnecting the USB serial.

## Diff to previous
```diff

diff -uNr 05_safe_globals/Cargo.toml 06_drivers_gpio_uart/Cargo.toml
--- 05_safe_globals/Cargo.toml
+++ 06_drivers_gpio_uart/Cargo.toml
@@ -10,10 +10,11 @@
 # The features section is used to select the target board.
 [features]
 default = []
-bsp_rpi3 = ["cortex-a"]
-bsp_rpi4 = ["cortex-a"]
+bsp_rpi3 = ["cortex-a", "register"]
+bsp_rpi4 = ["cortex-a", "register"]

 [dependencies]

 # Optional dependencies
 cortex-a = { version = "2.9.x", optional = true }
+register = { version = "0.5.x", optional = true }

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

diff -uNr 05_safe_globals/src/bsp/driver/bcm/bcm2xxx_gpio.rs 06_drivers_gpio_uart/src/bsp/driver/bcm/bcm2xxx_gpio.rs
--- 05_safe_globals/src/bsp/driver/bcm/bcm2xxx_gpio.rs
+++ 06_drivers_gpio_uart/src/bsp/driver/bcm/bcm2xxx_gpio.rs
@@ -0,0 +1,145 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
+
+//! GPIO driver.
+
+use crate::{arch, arch::sync::NullLock, interface};
+use core::ops;
+use register::{mmio::ReadWrite, register_bitfields, register_structs};
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
+register_structs! {
+    #[allow(non_snake_case)]
+    RegisterBlock {
+        (0x00 => GPFSEL0: ReadWrite<u32>),
+        (0x04 => GPFSEL1: ReadWrite<u32, GPFSEL1::Register>),
+        (0x08 => GPFSEL2: ReadWrite<u32>),
+        (0x0C => GPFSEL3: ReadWrite<u32>),
+        (0x10 => GPFSEL4: ReadWrite<u32>),
+        (0x14 => GPFSEL5: ReadWrite<u32>),
+        (0x18 => _reserved1),
+        (0x94 => GPPUD: ReadWrite<u32>),
+        (0x98 => GPPUDCLK0: ReadWrite<u32, GPPUDCLK0::Register>),
+        (0x9C => GPPUDCLK1: ReadWrite<u32>),
+        (0xA0 => @END),
+    }
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
+    /// Map PL011 UART as standard output.
+    ///
+    /// TX to pin 14
+    /// RX to pin 15
+    pub fn map_pl011_uart(&self) {
+        let mut r = &self.inner;
+        r.lock(|inner| {
+            // Map to pins.
+            inner
+                .GPFSEL1
+                .modify(GPFSEL1::FSEL14::AltFunc0 + GPFSEL1::FSEL15::AltFunc0);
+
+            // Enable pins 14 and 15.
+            inner.GPPUD.set(0);
+            arch::spin_for_cycles(150);
+
+            inner
+                .GPPUDCLK0
+                .write(GPPUDCLK0::PUDCLK14::AssertClock + GPPUDCLK0::PUDCLK15::AssertClock);
+            arch::spin_for_cycles(150);
+
+            inner.GPPUDCLK0.set(0);
+        })
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
@@ -0,0 +1,312 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
+
+//! PL011 UART driver.
+
+use crate::{arch, arch::sync::NullLock, interface};
+use core::{fmt, ops};
+use register::{mmio::*, register_bitfields, register_structs};
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
+    /// Interrupt Clear Register
+    ICR [
+        /// Meta field for all pending interrupts
+        ALL OFFSET(0) NUMBITS(11) []
+    ]
+}
+
+register_structs! {
+    #[allow(non_snake_case)]
+    pub RegisterBlock {
+        (0x00 => DR: ReadWrite<u32>),
+        (0x04 => _reserved1),
+        (0x18 => FR: ReadOnly<u32, FR::Register>),
+        (0x1c => _reserved2),
+        (0x24 => IBRD: WriteOnly<u32, IBRD::Register>),
+        (0x28 => FBRD: WriteOnly<u32, FBRD::Register>),
+        (0x2c => LCRH: WriteOnly<u32, LCRH::Register>),
+        (0x30 => CR: WriteOnly<u32, CR::Register>),
+        (0x34 => _reserved3),
+        (0x44 => ICR: WriteOnly<u32, ICR::Register>),
+        (0x48 => @END),
+    }
+}
+
+/// The driver's mutex protected part.
+pub struct PL011UartInner {
+    base_addr: usize,
+    chars_written: usize,
+    chars_read: usize,
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
+    pub const unsafe fn new(base_addr: usize) -> PL011UartInner {
+        PL011UartInner {
+            base_addr,
+            chars_written: 0,
+            chars_read: 0,
+        }
+    }
+
+    /// Set up baud rate and characteristics.
+    ///
+    /// Results in 8N1 and 230400 baud (if the clk has been previously set to 48 MHz by the
+    /// firmware).
+    pub fn init(&self) {
+        // Turn it off temporarily.
+        self.CR.set(0);
+
+        self.ICR.write(ICR::ALL::CLEAR);
+        self.IBRD.write(IBRD::IBRD.val(13));
+        self.FBRD.write(FBRD::FBRD.val(2));
+        self.LCRH
+            .write(LCRH::WLEN::EightBit + LCRH::FEN::FifosEnabled); // 8N1 + Fifo on
+        self.CR
+            .write(CR::UARTEN::Enabled + CR::TXE::Enabled + CR::RXE::Enabled);
+    }
+
+    /// Return a pointer to the register block.
+    fn ptr(&self) -> *const RegisterBlock {
+        self.base_addr as *const _
+    }
+
+    /// Send a character.
+    fn write_char(&mut self, c: char) {
+        // Spin while TX FIFO full is set, waiting for an empty slot.
+        while self.FR.matches_all(FR::TXFF::SET) {
+            arch::nop();
+        }
+
+        // Write the character to the buffer.
+        self.DR.set(c as u32);
+
+        self.chars_written += 1;
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
+            self.write_char(c);
+        }
+
+        Ok(())
+    }
+}
+
+//--------------------------------------------------------------------------------------------------
+// Export the inner struct so that BSPs can use it for the panic handler
+//--------------------------------------------------------------------------------------------------
+pub use PL011UartInner as PanicUart;
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
+    fn init(&self) -> interface::driver::Result {
+        let mut r = &self.inner;
+        r.lock(|inner| inner.init());
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
+            // Spin while RX FIFO empty is set.
+            while inner.FR.matches_all(FR::RXFE::SET) {
+                arch::nop();
+            }
+
+            // Read one character.
+            let mut ret = inner.DR.get() as u8 as char;
+
+            // Convert carrige return to newline.
+            if ret == '\r' {
+                ret = '\n'
+            }
+
+            // Update statistics.
+            inner.chars_read += 1;
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
+
+    fn chars_read(&self) -> usize {
+        let mut r = &self.inner;
+        r.lock(|inner| inner.chars_read)
+    }
+}

diff -uNr 05_safe_globals/src/bsp/driver/bcm.rs 06_drivers_gpio_uart/src/bsp/driver/bcm.rs
--- 05_safe_globals/src/bsp/driver/bcm.rs
+++ 06_drivers_gpio_uart/src/bsp/driver/bcm.rs
@@ -0,0 +1,11 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
+
+//! BCM driver top level.
+
+mod bcm2xxx_gpio;
+mod bcm2xxx_pl011_uart;
+
+pub use bcm2xxx_gpio::GPIO;
+pub use bcm2xxx_pl011_uart::{PL011Uart, PanicUart};

diff -uNr 05_safe_globals/src/bsp/driver.rs 06_drivers_gpio_uart/src/bsp/driver.rs
--- 05_safe_globals/src/bsp/driver.rs
+++ 06_drivers_gpio_uart/src/bsp/driver.rs
@@ -0,0 +1,11 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
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
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
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
@@ -4,7 +4,10 @@

 //! Board Support Package for the Raspberry Pi.

-use crate::{arch::sync::NullLock, interface};
+mod memory_map;
+
+use super::driver;
+use crate::interface;
 use core::fmt;

 /// Used by `arch` code to find the early boot core.
@@ -13,108 +16,59 @@
 /// The early boot core's stack address.
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
-            if c == '\n' {
-                self.write_char('\r')
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

-impl interface::console::Read for QEMUOutput {}
-
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
+/// Return a reference to a `console::All` implementation.
+pub fn console() -> &'static impl interface::console::All {
+    &PL011_UART
+}

-static QEMU_OUTPUT: QEMUOutput = QEMUOutput::new();
+/// In case of a panic, the panic handler uses this function to take a last shot at printing
+/// something before the system is halted.
+///
+/// # Safety
+///
+/// - Use only for printing during a panic.
+pub unsafe fn panic_console_out() -> impl fmt::Write {
+    let uart = driver::PanicUart::new(memory_map::mmio::PL011_UART_BASE);
+    uart.init();
+    uart
+}

-//--------------------------------------------------------------------------------------------------
-// Implementation of the kernel's BSP calls
-//--------------------------------------------------------------------------------------------------
+/// Return an array of references to all `DeviceDriver` compatible `BSP` drivers.
+///
+/// # Safety
+///
+/// The order of devices is the order in which `DeviceDriver::init()` is called.
+pub fn device_drivers() -> [&'static dyn interface::driver::DeviceDriver; 2] {
+    [&GPIO, &PL011_UART]
+}

-/// Return a reference to a `console::All` implementation.
-pub fn console() -> &'static impl interface::console::All {
-    &QEMU_OUTPUT
+/// BSP initialization code that runs after driver init.
+pub fn post_driver_init() {
+    // Configure PL011Uart's output pins.
+    GPIO.map_pl011_uart();
 }

diff -uNr 05_safe_globals/src/bsp.rs 06_drivers_gpio_uart/src/bsp.rs
--- 05_safe_globals/src/bsp.rs
+++ 06_drivers_gpio_uart/src/bsp.rs
@@ -4,6 +4,8 @@

 //! Conditional exporting of Board Support Packages.

+mod driver;
+
 #[cfg(any(feature = "bsp_rpi3", feature = "bsp_rpi4"))]
 mod rpi;


diff -uNr 05_safe_globals/src/interface.rs 06_drivers_gpio_uart/src/interface.rs
--- 05_safe_globals/src/interface.rs
+++ 06_drivers_gpio_uart/src/interface.rs
@@ -24,6 +24,9 @@

     /// Console write functions.
     pub trait Write {
+        /// Write a single character.
+        fn write_char(&self, c: char);
+
         /// Write a Rust format string.
         fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result;
     }
@@ -85,3 +88,20 @@
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
@@ -42,16 +42,48 @@

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
+            panic!("Error loading driver: {}", i.compatible())
+        }
+    }
+    bsp::post_driver_init();
+    // println! is usable from here on.
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
+        if bsp::console().read_char() == '\n' {
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

diff -uNr 05_safe_globals/src/panic_wait.rs 06_drivers_gpio_uart/src/panic_wait.rs
--- 05_safe_globals/src/panic_wait.rs
+++ 06_drivers_gpio_uart/src/panic_wait.rs
@@ -4,15 +4,31 @@

 //! A panic handler that infinitely waits.

-use crate::{arch, println};
-use core::panic::PanicInfo;
+use crate::{arch, bsp};
+use core::{fmt, panic::PanicInfo};
+
+fn _panic_print(args: fmt::Arguments) {
+    use fmt::Write;
+
+    unsafe { bsp::panic_console_out().write_fmt(args).unwrap() };
+}
+
+/// Prints with a newline - only use from the panic handler.
+///
+/// Carbon copy from https://doc.rust-lang.org/src/std/macros.rs.html
+#[macro_export]
+macro_rules! panic_println {
+    ($($arg:tt)*) => ({
+        _panic_print(format_args_nl!($($arg)*));
+    })
+}

 #[panic_handler]
 fn panic(info: &PanicInfo) -> ! {
     if let Some(args) = info.message() {
-        println!("\nKernel panic: {}", args);
+        panic_println!("\nKernel panic: {}", args);
     } else {
-        println!("\nKernel panic!");
+        panic_println!("\nKernel panic!");
     }

     arch::wait_forever()

```
