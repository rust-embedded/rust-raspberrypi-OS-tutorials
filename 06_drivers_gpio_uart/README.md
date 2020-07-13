# Tutorial 06 - Drivers: GPIO and UART

## tl;dr

Now that we enabled safe globals in the previous tutorial, the infrastructure is laid for adding the
first real device drivers. We throw out the magic QEMU console and use a real `UART` now. Like
serious embedded hackers do!

## Notable additions

- For the first time, we will be able to run the code on the real hardware.
  - Therefore, building is now differentiated between the **RPi 3** and the **RPi4**.
  - By default, all `Makefile` targets will build for the **RPi 3**.
  - In order to build for the the **RPi4**, prepend `BSP=rpi4` to each target. For example:
    - `BSP=rpi4 make`
    - `BSP=rpi4 make doc`
  - Unfortunately, QEMU does not yet support the **RPi4**, so `BSP=rpi4 make qemu` won't work.
- A `driver::interface::DeviceDriver` trait is added for abstracting `BSP` driver implementations
  from kernel code.
- Drivers are stored in `src/bsp/device_driver`, and can be reused between `BSP`s.
    - We introduce the `GPIO` driver, which pinmuxes the RPi's PL011 UART.
    - Most importantly, the `PL011Uart` driver: It implements the `console::interface::*` traits and
      is from now on used as the main system console output.
- `BSP`s now contain a memory map in `src/bsp/raspberrypi/memory.rs`. In the specific case, they contain the
  Raspberry's `MMIO` addresses which are used to instantiate the respectivedevice drivers.
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

3. Copy the following files from the [Raspberry Pi firmware repo](https://github.com/raspberrypi/firmware/tree/master/boot) onto the SD card:
    - [bootcode.bin](https://github.com/raspberrypi/firmware/raw/master/boot/bootcode.bin)
    - [fixup.dat](https://github.com/raspberrypi/firmware/raw/master/boot/fixup.dat)
    - [start.elf](https://github.com/raspberrypi/firmware/raw/master/boot/start.elf)
4. Run `make` and copy the [kernel8.img](kernel8.img) onto the SD card.

### Pi 4

3. Copy the following files from the [Raspberry Pi firmware repo](https://github.com/raspberrypi/firmware/tree/master/boot) onto the SD card:
    - [fixup4.dat](https://github.com/raspberrypi/firmware/raw/master/boot/fixup4.dat)
    - [start4.elf](https://github.com/raspberrypi/firmware/raw/master/boot/start4.elf)
    - [bcm2711-rpi-4-b.dtb](https://github.com/raspberrypi/firmware/raw/master/boot/bcm2711-rpi-4-b.dtb)
4. Run `BSP=rpi4 make` and copy the [kernel8.img](kernel8.img) onto the SD card.

_**Note**: Should it not work on your RPi4, try renaming `start4.elf` to `start.elf` (without the 4)
on the SD card._

### Common again

5. Insert the SD card into the RPi and connect the USB serial to your host PC.
    - Wiring diagram at [top-level README](../README.md#usb-serial).
6. Run `screen` (you might need to install it first):

```console
$ sudo screen /dev/ttyUSB0 230400
```

> ❗ **NOTE**: Depending on your host operating system, the serial device name might differ.
> For example, on `macOS`, it might be something like `/dev/tty.usbserial-0001`.

7. Hit <kbd>Enter</kbd> to kick off the kernel boot process. Observe the output:

```console
[0] Booting on: Raspberry Pi 3
[1] Drivers loaded:
      1. BCM GPIO
      2. BCM PL011 UART
[2] Chars written: 93
[3] Echoing input now
```

8. Exit screen by pressing <kbd>ctrl-a</kbd> <kbd>ctrl-d</kbd> or disconnecting the USB serial.

## Diff to previous
```diff

diff -uNr 05_safe_globals/Cargo.toml 06_drivers_gpio_uart/Cargo.toml
--- 05_safe_globals/Cargo.toml
+++ 06_drivers_gpio_uart/Cargo.toml
@@ -7,10 +7,11 @@
 # The features section is used to select the target board.
 [features]
 default = []
-bsp_rpi3 = ["cortex-a"]
-bsp_rpi4 = ["cortex-a"]
+bsp_rpi3 = ["cortex-a", "register"]
+bsp_rpi4 = ["cortex-a", "register"]

 [dependencies]

 # Optional dependencies
 cortex-a = { version = "3.0.x", optional = true }
+register = { version = "0.5.x", optional = true }

diff -uNr 05_safe_globals/src/_arch/aarch64/cpu.rs 06_drivers_gpio_uart/src/_arch/aarch64/cpu.rs
--- 05_safe_globals/src/_arch/aarch64/cpu.rs
+++ 06_drivers_gpio_uart/src/_arch/aarch64/cpu.rs
@@ -37,6 +37,16 @@
 // Public Code
 //--------------------------------------------------------------------------------------------------

+pub use asm::nop;
+
+/// Spin for `n` cycles.
+#[inline(always)]
+pub fn spin_for_cycles(n: usize) {
+    for _ in 0..n {
+        asm::nop();
+    }
+}
+
 /// Pause execution on the core.
 #[inline(always)]
 pub fn wait_forever() -> ! {

diff -uNr 05_safe_globals/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs 06_drivers_gpio_uart/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs
--- 05_safe_globals/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs
+++ 06_drivers_gpio_uart/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs
@@ -0,0 +1,138 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
+
+//! GPIO Driver.
+
+use crate::{
+    bsp::device_driver::common::MMIODerefWrapper, cpu, driver, synchronization,
+    synchronization::NullLock,
+};
+use register::{mmio::*, register_bitfields, register_structs};
+
+//--------------------------------------------------------------------------------------------------
+// Private Definitions
+//--------------------------------------------------------------------------------------------------
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
+/// Abstraction for the associated MMIO registers.
+type Registers = MMIODerefWrapper<RegisterBlock>;
+
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
+/// Representation of the GPIO HW.
+pub struct GPIO {
+    registers: NullLock<Registers>,
+}
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+impl GPIO {
+    /// Create an instance.
+    ///
+    /// # Safety
+    ///
+    /// - The user must ensure to provide the correct `base_addr`.
+    pub const unsafe fn new(base_addr: usize) -> Self {
+        Self {
+            registers: NullLock::new(Registers::new(base_addr)),
+        }
+    }
+
+    /// Map PL011 UART as standard output.
+    ///
+    /// TX to pin 14
+    /// RX to pin 15
+    pub fn map_pl011_uart(&self) {
+        let mut r = &self.registers;
+        r.lock(|registers| {
+            // Map to pins.
+            registers
+                .GPFSEL1
+                .modify(GPFSEL1::FSEL14::AltFunc0 + GPFSEL1::FSEL15::AltFunc0);
+
+            // Enable pins 14 and 15.
+            registers.GPPUD.set(0);
+            cpu::spin_for_cycles(150);
+
+            registers
+                .GPPUDCLK0
+                .write(GPPUDCLK0::PUDCLK14::AssertClock + GPPUDCLK0::PUDCLK15::AssertClock);
+            cpu::spin_for_cycles(150);
+
+            registers.GPPUDCLK0.set(0);
+        })
+    }
+}
+
+//------------------------------------------------------------------------------
+// OS Interface Code
+//------------------------------------------------------------------------------
+use synchronization::interface::Mutex;
+
+impl driver::interface::DeviceDriver for GPIO {
+    fn compatible(&self) -> &str {
+        "BCM GPIO"
+    }
+}

diff -uNr 05_safe_globals/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs 06_drivers_gpio_uart/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
--- 05_safe_globals/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
+++ 06_drivers_gpio_uart/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
@@ -0,0 +1,307 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
+
+//! PL011 UART driver.
+
+use crate::{
+    bsp::device_driver::common::MMIODerefWrapper, console, cpu, driver, synchronization,
+    synchronization::NullLock,
+};
+use core::fmt;
+use register::{mmio::*, register_bitfields, register_structs};
+
+//--------------------------------------------------------------------------------------------------
+// Private Definitions
+//--------------------------------------------------------------------------------------------------
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
+/// Abstraction for the associated MMIO registers.
+type Registers = MMIODerefWrapper<RegisterBlock>;
+
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
+pub struct PL011UartInner {
+    registers: Registers,
+    chars_written: usize,
+    chars_read: usize,
+}
+
+// Export the inner struct so that BSPs can use it for the panic handler.
+pub use PL011UartInner as PanicUart;
+
+/// Representation of the UART.
+pub struct PL011Uart {
+    inner: NullLock<PL011UartInner>,
+}
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+impl PL011UartInner {
+    /// Create an instance.
+    ///
+    /// # Safety
+    ///
+    /// - The user must ensure to provide the correct `base_addr`.
+    pub const unsafe fn new(base_addr: usize) -> Self {
+        Self {
+            registers: Registers::new(base_addr),
+            chars_written: 0,
+            chars_read: 0,
+        }
+    }
+
+    /// Set up baud rate and characteristics.
+    ///
+    /// Results in 8N1 and 230400 baud (if the clk has been previously set to 48 MHz by the
+    /// firmware).
+    pub fn init(&mut self) {
+        // Turn it off temporarily.
+        self.registers.CR.set(0);
+
+        self.registers.ICR.write(ICR::ALL::CLEAR);
+        self.registers.IBRD.write(IBRD::IBRD.val(13));
+        self.registers.FBRD.write(FBRD::FBRD.val(1));
+        self.registers
+            .LCRH
+            .write(LCRH::WLEN::EightBit + LCRH::FEN::FifosEnabled); // 8N1 + Fifo on
+        self.registers
+            .CR
+            .write(CR::UARTEN::Enabled + CR::TXE::Enabled + CR::RXE::Enabled);
+    }
+
+    /// Send a character.
+    fn write_char(&mut self, c: char) {
+        // Spin while TX FIFO full is set, waiting for an empty slot.
+        while self.registers.FR.matches_all(FR::TXFF::SET) {
+            cpu::nop();
+        }
+
+        // Write the character to the buffer.
+        self.registers.DR.set(c as u32);
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
+impl PL011Uart {
+    /// # Safety
+    ///
+    /// - The user must ensure to provide the correct `base_addr`.
+    pub const unsafe fn new(base_addr: usize) -> Self {
+        Self {
+            inner: NullLock::new(PL011UartInner::new(base_addr)),
+        }
+    }
+}
+
+//------------------------------------------------------------------------------
+// OS Interface Code
+//------------------------------------------------------------------------------
+use synchronization::interface::Mutex;
+
+impl driver::interface::DeviceDriver for PL011Uart {
+    fn compatible(&self) -> &str {
+        "BCM PL011 UART"
+    }
+
+    fn init(&self) -> Result<(), ()> {
+        let mut r = &self.inner;
+        r.lock(|inner| inner.init());
+
+        Ok(())
+    }
+}
+
+impl console::interface::Write for PL011Uart {
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
+impl console::interface::Read for PL011Uart {
+    fn read_char(&self) -> char {
+        let mut r = &self.inner;
+        r.lock(|inner| {
+            // Spin while RX FIFO empty is set.
+            while inner.registers.FR.matches_all(FR::RXFE::SET) {
+                cpu::nop();
+            }
+
+            // Read one character.
+            let mut ret = inner.registers.DR.get() as u8 as char;
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
+impl console::interface::Statistics for PL011Uart {
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

diff -uNr 05_safe_globals/src/bsp/device_driver/bcm.rs 06_drivers_gpio_uart/src/bsp/device_driver/bcm.rs
--- 05_safe_globals/src/bsp/device_driver/bcm.rs
+++ 06_drivers_gpio_uart/src/bsp/device_driver/bcm.rs
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
+pub use bcm2xxx_gpio::*;
+pub use bcm2xxx_pl011_uart::*;

diff -uNr 05_safe_globals/src/bsp/device_driver/common.rs 06_drivers_gpio_uart/src/bsp/device_driver/common.rs
--- 05_safe_globals/src/bsp/device_driver/common.rs
+++ 06_drivers_gpio_uart/src/bsp/device_driver/common.rs
@@ -0,0 +1,35 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2020 Andre Richter <andre.o.richter@gmail.com>
+
+//! Common device driver code.
+
+use core::{marker::PhantomData, ops};
+
+pub struct MMIODerefWrapper<T> {
+    base_addr: usize,
+    phantom: PhantomData<T>,
+}
+
+impl<T> MMIODerefWrapper<T> {
+    /// Create an instance.
+    pub const unsafe fn new(base_addr: usize) -> Self {
+        Self {
+            base_addr,
+            phantom: PhantomData,
+        }
+    }
+
+    /// Return a pointer to the associated MMIO register block.
+    fn ptr(&self) -> *const T {
+        self.base_addr as *const _
+    }
+}
+
+impl<T> ops::Deref for MMIODerefWrapper<T> {
+    type Target = T;
+
+    fn deref(&self) -> &Self::Target {
+        unsafe { &*self.ptr() }
+    }
+}

diff -uNr 05_safe_globals/src/bsp/device_driver.rs 06_drivers_gpio_uart/src/bsp/device_driver.rs
--- 05_safe_globals/src/bsp/device_driver.rs
+++ 06_drivers_gpio_uart/src/bsp/device_driver.rs
@@ -0,0 +1,12 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
+
+//! Device driver.
+
+#[cfg(any(feature = "bsp_rpi3", feature = "bsp_rpi4"))]
+mod bcm;
+mod common;
+
+#[cfg(any(feature = "bsp_rpi3", feature = "bsp_rpi4"))]
+pub use bcm::*;

diff -uNr 05_safe_globals/src/bsp/raspberrypi/console.rs 06_drivers_gpio_uart/src/bsp/raspberrypi/console.rs
--- 05_safe_globals/src/bsp/raspberrypi/console.rs
+++ 06_drivers_gpio_uart/src/bsp/raspberrypi/console.rs
@@ -4,115 +4,27 @@

 //! BSP console facilities.

-use crate::{console, synchronization, synchronization::NullLock};
+use super::memory;
+use crate::{bsp::device_driver, console};
 use core::fmt;

 //--------------------------------------------------------------------------------------------------
-// Private Definitions
-//--------------------------------------------------------------------------------------------------
-
-/// A mystical, magical device for generating QEMU output out of the void.
-///
-/// The mutex protected part.
-struct QEMUOutputInner {
-    chars_written: usize,
-}
-
-//--------------------------------------------------------------------------------------------------
-// Public Definitions
-//--------------------------------------------------------------------------------------------------
-
-/// The main struct.
-pub struct QEMUOutput {
-    inner: NullLock<QEMUOutputInner>,
-}
-
-//--------------------------------------------------------------------------------------------------
-// Global instances
-//--------------------------------------------------------------------------------------------------
-
-static QEMU_OUTPUT: QEMUOutput = QEMUOutput::new();
-
-//--------------------------------------------------------------------------------------------------
-// Private Code
+// Public Code
 //--------------------------------------------------------------------------------------------------

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
-
-        self.chars_written += 1;
-    }
-}
-
-/// Implementing `core::fmt::Write` enables usage of the `format_args!` macros, which in turn are
-/// used to implement the `kernel`'s `print!` and `println!` macros. By implementing `write_str()`,
-/// we get `write_fmt()` automatically.
+/// In case of a panic, the panic handler uses this function to take a last shot at printing
+/// something before the system is halted.
 ///
-/// The function takes an `&mut self`, so it must be implemented for the inner struct.
+/// # Safety
 ///
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
-        Ok(())
-    }
-}
-
-//--------------------------------------------------------------------------------------------------
-// Public Code
-//--------------------------------------------------------------------------------------------------
-
-impl QEMUOutput {
-    /// Create a new instance.
-    pub const fn new() -> QEMUOutput {
-        QEMUOutput {
-            inner: NullLock::new(QEMUOutputInner::new()),
-        }
-    }
+/// - Use only for printing during a panic.
+pub unsafe fn panic_console_out() -> impl fmt::Write {
+    let mut uart = device_driver::PanicUart::new(memory::map::mmio::PL011_UART_BASE);
+    uart.init();
+    uart
 }

 /// Return a reference to the console.
 pub fn console() -> &'static impl console::interface::All {
-    &QEMU_OUTPUT
-}
-
-//------------------------------------------------------------------------------
-// OS Interface Code
-//------------------------------------------------------------------------------
-use synchronization::interface::Mutex;
-
-/// Passthrough of `args` to the `core::fmt::Write` implementation, but guarded by a Mutex to
-/// serialize access.
-impl console::interface::Write for QEMUOutput {
-    fn write_fmt(&self, args: core::fmt::Arguments) -> fmt::Result {
-        // Fully qualified syntax for the call to `core::fmt::Write::write:fmt()` to increase
-        // readability.
-        let mut r = &self.inner;
-        r.lock(|inner| fmt::Write::write_fmt(inner, args))
-    }
-}
-
-impl console::interface::Statistics for QEMUOutput {
-    fn chars_written(&self) -> usize {
-        let mut r = &self.inner;
-        r.lock(|inner| inner.chars_written)
-    }
+    &super::PL011_UART
 }

diff -uNr 05_safe_globals/src/bsp/raspberrypi/driver.rs 06_drivers_gpio_uart/src/bsp/raspberrypi/driver.rs
--- 05_safe_globals/src/bsp/raspberrypi/driver.rs
+++ 06_drivers_gpio_uart/src/bsp/raspberrypi/driver.rs
@@ -0,0 +1,49 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
+
+//! BSP driver support.
+
+use crate::driver;
+
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
+/// Device Driver Manager type.
+pub struct BSPDriverManager {
+    device_drivers: [&'static (dyn DeviceDriver + Sync); 2],
+}
+
+//--------------------------------------------------------------------------------------------------
+// Global instances
+//--------------------------------------------------------------------------------------------------
+
+static BSP_DRIVER_MANAGER: BSPDriverManager = BSPDriverManager {
+    device_drivers: [&super::GPIO, &super::PL011_UART],
+};
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+/// Return a reference to the driver manager.
+pub fn driver_manager() -> &'static impl driver::interface::DriverManager {
+    &BSP_DRIVER_MANAGER
+}
+
+//------------------------------------------------------------------------------
+// OS Interface Code
+//------------------------------------------------------------------------------
+use driver::interface::DeviceDriver;
+
+impl driver::interface::DriverManager for BSPDriverManager {
+    fn all_device_drivers(&self) -> &[&'static (dyn DeviceDriver + Sync)] {
+        &self.device_drivers[..]
+    }
+
+    fn post_device_driver_init(&self) {
+        // Configure PL011Uart's output pins.
+        super::GPIO.map_pl011_uart();
+    }
+}

diff -uNr 05_safe_globals/src/bsp/raspberrypi/memory.rs 06_drivers_gpio_uart/src/bsp/raspberrypi/memory.rs
--- 05_safe_globals/src/bsp/raspberrypi/memory.rs
+++ 06_drivers_gpio_uart/src/bsp/raspberrypi/memory.rs
@@ -0,0 +1,36 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
+
+//! BSP Memory Management.
+
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
+/// The board's memory map.
+#[rustfmt::skip]
+pub(super) mod map {
+    pub const GPIO_OFFSET:                              usize =        0x0020_0000;
+    pub const UART_OFFSET:                              usize =        0x0020_1000;
+
+    /// Physical devices.
+    #[cfg(feature = "bsp_rpi3")]
+    pub mod mmio {
+        use super::*;
+
+        pub const BASE:                                 usize =        0x3F00_0000;
+        pub const GPIO_BASE:                            usize = BASE + GPIO_OFFSET;
+        pub const PL011_UART_BASE:                      usize = BASE + UART_OFFSET;
+    }
+
+    /// Physical devices.
+    #[cfg(feature = "bsp_rpi4")]
+    pub mod mmio {
+        use super::*;
+
+        pub const BASE:                                 usize =        0xFE00_0000;
+        pub const GPIO_BASE:                            usize = BASE + GPIO_OFFSET;
+        pub const PL011_UART_BASE:                      usize = BASE + UART_OFFSET;
+    }
+}

diff -uNr 05_safe_globals/src/bsp/raspberrypi.rs 06_drivers_gpio_uart/src/bsp/raspberrypi.rs
--- 05_safe_globals/src/bsp/raspberrypi.rs
+++ 06_drivers_gpio_uart/src/bsp/raspberrypi.rs
@@ -6,3 +6,33 @@

 pub mod console;
 pub mod cpu;
+pub mod driver;
+pub mod memory;
+
+//--------------------------------------------------------------------------------------------------
+// Global instances
+//--------------------------------------------------------------------------------------------------
+use super::device_driver;
+
+static GPIO: device_driver::GPIO =
+    unsafe { device_driver::GPIO::new(memory::map::mmio::GPIO_BASE) };
+
+static PL011_UART: device_driver::PL011Uart =
+    unsafe { device_driver::PL011Uart::new(memory::map::mmio::PL011_UART_BASE) };
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+/// Board identification.
+pub fn board_name() -> &'static str {
+    #[cfg(feature = "bsp_rpi3")]
+    {
+        "Raspberry Pi 3"
+    }
+
+    #[cfg(feature = "bsp_rpi4")]
+    {
+        "Raspberry Pi 4"
+    }
+}

diff -uNr 05_safe_globals/src/bsp.rs 06_drivers_gpio_uart/src/bsp.rs
--- 05_safe_globals/src/bsp.rs
+++ 06_drivers_gpio_uart/src/bsp.rs
@@ -4,6 +4,8 @@

 //! Conditional re-exporting of Board Support Packages.

+mod device_driver;
+
 #[cfg(any(feature = "bsp_rpi3", feature = "bsp_rpi4"))]
 mod raspberrypi;


diff -uNr 05_safe_globals/src/console.rs 06_drivers_gpio_uart/src/console.rs
--- 05_safe_globals/src/console.rs
+++ 06_drivers_gpio_uart/src/console.rs
@@ -14,18 +14,34 @@

     /// Console write functions.
     pub trait Write {
+        /// Write a single character.
+        fn write_char(&self, c: char);
+
         /// Write a Rust format string.
         fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result;
     }

+    /// Console read functions.
+    pub trait Read {
+        /// Read a single character.
+        fn read_char(&self) -> char {
+            ' '
+        }
+    }
+
     /// Console statistics.
     pub trait Statistics {
         /// Return the number of characters written.
         fn chars_written(&self) -> usize {
             0
         }
+
+        /// Return the number of characters read.
+        fn chars_read(&self) -> usize {
+            0
+        }
     }

     /// Trait alias for a full-fledged console.
-    pub trait All = Write + Statistics;
+    pub trait All = Write + Read + Statistics;
 }

diff -uNr 05_safe_globals/src/driver.rs 06_drivers_gpio_uart/src/driver.rs
--- 05_safe_globals/src/driver.rs
+++ 06_drivers_gpio_uart/src/driver.rs
@@ -0,0 +1,41 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
+
+//! Driver support.
+
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
+/// Driver interfaces.
+pub mod interface {
+
+    /// Device Driver functions.
+    pub trait DeviceDriver {
+        /// Return a compatibility string for identifying the driver.
+        fn compatible(&self) -> &str;
+
+        /// Called by the kernel to bring up the device.
+        fn init(&self) -> Result<(), ()> {
+            Ok(())
+        }
+    }
+
+    /// Device driver management functions.
+    ///
+    /// The `BSP` is supposed to supply one global instance.
+    pub trait DriverManager {
+        /// Return a slice of references to all `BSP`-instantiated drivers.
+        ///
+        /// # Safety
+        ///
+        /// - The order of devices is the order in which `DeviceDriver::init()` is called.
+        fn all_device_drivers(&self) -> &[&'static (dyn DeviceDriver + Sync)];
+
+        /// Initialization code that runs after driver init.
+        ///
+        /// For example, device driver code that depends on other drivers already being online.
+        fn post_device_driver_init(&self);
+    }
+}

diff -uNr 05_safe_globals/src/main.rs 06_drivers_gpio_uart/src/main.rs
--- 05_safe_globals/src/main.rs
+++ 06_drivers_gpio_uart/src/main.rs
@@ -7,6 +7,14 @@

 //! The `kernel` binary.
 //!
+//! # TL;DR - Overview of important Kernel entities
+//!
+//! - [`bsp::console::console()`] - Returns a reference to the kernel's [console interface].
+//! - [`bsp::driver::driver_manager()`] - Returns a reference to the kernel's [driver interface].
+//!
+//! [console interface]: ../libkernel/console/interface/index.html
+//! [driver interface]: ../libkernel/driver/interface/trait.DriverManager.html
+//!
 //! # Code organization and architecture
 //!
 //! The code is divided into different *modules*, each representing a typical **subsystem** of the
@@ -105,6 +113,7 @@
 mod bsp;
 mod console;
 mod cpu;
+mod driver;
 mod memory;
 mod panic_wait;
 mod print;
@@ -116,16 +125,53 @@
 /// # Safety
 ///
 /// - Only a single core must be active and running this function.
+/// - The init calls in this function must appear in the correct order.
 unsafe fn kernel_init() -> ! {
-    use console::interface::Statistics;
+    use driver::interface::DriverManager;
+
+    for i in bsp::driver::driver_manager().all_device_drivers().iter() {
+        if i.init().is_err() {
+            panic!("Error loading driver: {}", i.compatible())
+        }
+    }
+    bsp::driver::driver_manager().post_device_driver_init();
+    // println! is usable from here on.
+
+    // Transition from unsafe to safe.
+    kernel_main()
+}

-    println!("[0] Hello from pure Rust!");
+/// The main function running after the early init.
+fn kernel_main() -> ! {
+    use console::interface::All;
+    use driver::interface::DriverManager;
+
+    // Wait for user to hit Enter.
+    loop {
+        if bsp::console::console().read_char() == '\n' {
+            break;
+        }
+    }
+
+    println!("[0] Booting on: {}", bsp::board_name());
+
+    println!("[1] Drivers loaded:");
+    for (i, driver) in bsp::driver::driver_manager()
+        .all_device_drivers()
+        .iter()
+        .enumerate()
+    {
+        println!("      {}. {}", i + 1, driver.compatible());
+    }

     println!(
-        "[1] Chars written: {}",
+        "[2] Chars written: {}",
         bsp::console::console().chars_written()
     );
+    println!("[3] Echoing input now");

-    println!("[2] Stopping here.");
-    cpu::wait_forever()
+    loop {
+        let c = bsp::console::console().read_char();
+        bsp::console::console().write_char(c);
+    }
 }

diff -uNr 05_safe_globals/src/panic_wait.rs 06_drivers_gpio_uart/src/panic_wait.rs
--- 05_safe_globals/src/panic_wait.rs
+++ 06_drivers_gpio_uart/src/panic_wait.rs
@@ -4,15 +4,35 @@

 //! A panic handler that infinitely waits.

-use crate::{cpu, println};
-use core::panic::PanicInfo;
+use crate::{bsp, cpu};
+use core::{fmt, panic::PanicInfo};
+
+//--------------------------------------------------------------------------------------------------
+// Private Code
+//--------------------------------------------------------------------------------------------------
+
+fn _panic_print(args: fmt::Arguments) {
+    use fmt::Write;
+
+    unsafe { bsp::console::panic_console_out().write_fmt(args).unwrap() };
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

     cpu::wait_forever()

```
