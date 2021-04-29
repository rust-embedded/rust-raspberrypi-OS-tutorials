# Tutorial 05 - Drivers: GPIO and UART

## tl;dr

- Now that we enabled safe globals in the previous tutorial, the infrastructure is laid for adding
  the first real device drivers.
- We throw out the magic QEMU console and use a real `UART` now. Like serious embedded hackers do!

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
    - We introduce the `GPIO` driver, which pinmuxes (that is, routing signals from inside the `SoC`
      to actual HW pins) the RPi's PL011 UART.
      - Note how this driver differentiates between **RPi 3** and **RPi4**. Their HW is different,
        so we have to account for it in SW.
    - Most importantly, the `PL011Uart` driver: It implements the `console::interface::*` traits and
      is from now on used as the main system console output.
- `BSP`s now contain a memory map in `src/bsp/raspberrypi/memory.rs`. In the specific case, they
  contain the Raspberry's `MMIO` addresses which are used to instantiate the respective device
  drivers.
- We also modify the `panic!` handler, so that it does not anymore rely on `println!`, which uses
  the globally-shared instance of the `UART` that might be locked when an error is encountered (for
  now, this can't happen due to the `NullLock`, but with a real lock it becomes an issue).
    - Instead, it creates a new UART driver instance, re-initializes the device and uses that one to
      print. This increases the chances that the system is able to print a final important message
      before it suspends itself.

## Boot it from SD card

Some steps for preparing the SD card differ between RPi3 and RPi4, so be careful.

### Common for both

1. Make a single `FAT32` partition named `boot`.
2. On the card, generate a file named `config.txt` with the following contents:

```txt
arm_64bit=1
init_uart_clock=48000000
```
### Pi 3

3. Copy the following files from the [Raspberry Pi firmware repo](https://github.com/raspberrypi/firmware/tree/master/boot) onto the SD card:
    - [bootcode.bin](https://github.com/raspberrypi/firmware/raw/master/boot/bootcode.bin)
    - [fixup.dat](https://github.com/raspberrypi/firmware/raw/master/boot/fixup.dat)
    - [start.elf](https://github.com/raspberrypi/firmware/raw/master/boot/start.elf)
4. Run `make`.

### Pi 4

3. Copy the following files from the [Raspberry Pi firmware repo](https://github.com/raspberrypi/firmware/tree/master/boot) onto the SD card:
    - [fixup4.dat](https://github.com/raspberrypi/firmware/raw/master/boot/fixup4.dat)
    - [start4.elf](https://github.com/raspberrypi/firmware/raw/master/boot/start4.elf)
    - [bcm2711-rpi-4-b.dtb](https://github.com/raspberrypi/firmware/raw/master/boot/bcm2711-rpi-4-b.dtb)
4. Run `BSP=rpi4 make`.


_**Note**: Should it not work on your RPi4, try renaming `start4.elf` to `start.elf` (without the 4)
on the SD card._

### Common again

5. Copy the `kernel8.img` onto the SD card and insert it back into the RPi.
6. Run the `miniterm` target, which opens the UART device on the host:

```console
$ make miniterm
```

> ❗ **NOTE**: `Miniterm` assumes a default serial device name of `/dev/ttyUSB0`. Depending on your
> host operating system, the device name might differ. For example, on `macOS`, it might be
> something like `/dev/tty.usbserial-0001`. In this case, please give the name explicitly:


```console
$ DEV_SERIAL=/dev/tty.usbserial-0001 make miniterm
```

7. Connect the USB serial to your host PC.
    - Wiring diagram at [top-level README](../README.md#-usb-serial-output).
    - Make sure that you **DID NOT** connect the power pin of the USB serial. Only RX/TX and GND.
8. Connect the RPi to the (USB) power cable and observe the output:

```console
Miniterm 1.0

[MT] ⏳ Waiting for /dev/ttyUSB0
[MT] ✅ Serial connected
[0] mingo version 0.5.0
[1] Booting on: Raspberry Pi 3
[2] Drivers loaded:
      1. BCM GPIO
      2. BCM PL011 UART
[3] Chars written: 117
[4] Echoing input now
```

8. Exit by pressing <kbd>ctrl-c</kbd>.

## Diff to previous
```diff

diff -uNr 04_safe_globals/Cargo.toml 05_drivers_gpio_uart/Cargo.toml
--- 04_safe_globals/Cargo.toml
+++ 05_drivers_gpio_uart/Cargo.toml
@@ -1,6 +1,6 @@
 [package]
 name = "mingo"
-version = "0.4.0"
+version = "0.5.0"
 authors = ["Andre Richter <andre.o.richter@gmail.com>"]
 edition = "2018"

@@ -9,8 +9,8 @@

 [features]
 default = []
-bsp_rpi3 = []
-bsp_rpi4 = []
+bsp_rpi3 = ["register"]
+bsp_rpi4 = ["register"]

 [[bin]]
 name = "kernel"
@@ -22,6 +22,9 @@

 [dependencies]

+# Optional dependencies
+register = { version = "1.x.x", optional = true }
+
 # Platform specific dependencies
 [target.'cfg(target_arch = "aarch64")'.dependencies]
 cortex-a = { version = "5.x.x" }

diff -uNr 04_safe_globals/Makefile 05_drivers_gpio_uart/Makefile
--- 04_safe_globals/Makefile
+++ 05_drivers_gpio_uart/Makefile
@@ -7,6 +7,12 @@
 # Default to the RPi3
 BSP ?= rpi3

+# Default to a serial device name that is common in Linux.
+DEV_SERIAL ?= /dev/ttyUSB0
+
+# Query the host system's kernel name
+UNAME_S = $(shell uname -s)
+
 # BSP-specific arguments
 ifeq ($(BSP),rpi3)
     TARGET            = aarch64-unknown-none-softfloat
@@ -58,13 +64,23 @@
 DOCKER_IMAGE         = rustembedded/osdev-utils
 DOCKER_CMD           = docker run --rm -v $(shell pwd):/work/tutorial -w /work/tutorial
 DOCKER_CMD_INTERACT  = $(DOCKER_CMD) -i -t
+DOCKER_ARG_DIR_UTILS = -v $(shell pwd)/../utils:/work/utils
+DOCKER_ARG_DEV       = --privileged -v /dev:/dev

 DOCKER_QEMU  = $(DOCKER_CMD_INTERACT) $(DOCKER_IMAGE)
 DOCKER_TOOLS = $(DOCKER_CMD) $(DOCKER_IMAGE)

-EXEC_QEMU = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)
+# Dockerize commands that require USB device passthrough only on Linux
+ifeq ($(UNAME_S),Linux)
+    DOCKER_CMD_DEV = $(DOCKER_CMD_INTERACT) $(DOCKER_ARG_DEV)
+
+    DOCKER_MINITERM = $(DOCKER_CMD_DEV) $(DOCKER_ARG_DIR_UTILS) $(DOCKER_IMAGE)
+endif
+
+EXEC_QEMU     = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)
+EXEC_MINITERM = ruby ../utils/miniterm.rb

-.PHONY: all $(KERNEL_ELF) $(KERNEL_BIN) doc qemu clippy clean readelf objdump nm check
+.PHONY: all $(KERNEL_ELF) $(KERNEL_BIN) doc qemu miniterm clippy clean readelf objdump nm check

 all: $(KERNEL_BIN)

@@ -88,6 +104,9 @@
 	@$(DOCKER_QEMU) $(EXEC_QEMU) $(QEMU_RELEASE_ARGS) -kernel $(KERNEL_BIN)
 endif

+miniterm:
+	@$(DOCKER_MINITERM) $(EXEC_MINITERM) $(DEV_SERIAL)
+
 clippy:
 	@RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(CLIPPY_CMD)


diff -uNr 04_safe_globals/src/_arch/aarch64/cpu.rs 05_drivers_gpio_uart/src/_arch/aarch64/cpu.rs
--- 04_safe_globals/src/_arch/aarch64/cpu.rs
+++ 05_drivers_gpio_uart/src/_arch/aarch64/cpu.rs
@@ -17,6 +17,17 @@
 // Public Code
 //--------------------------------------------------------------------------------------------------

+pub use asm::nop;
+
+/// Spin for `n` cycles.
+#[cfg(feature = "bsp_rpi3")]
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

diff -uNr 04_safe_globals/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs 05_drivers_gpio_uart/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs
--- 04_safe_globals/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs
+++ 05_drivers_gpio_uart/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs
@@ -0,0 +1,221 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
+
+//! GPIO Driver.
+
+use crate::{
+    bsp::device_driver::common::MMIODerefWrapper, driver, synchronization,
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
+// - https://github.com/raspberrypi/documentation/files/1888662/BCM2837-ARM-Peripherals.-.Revised.-.V2-1.pdf
+// - https://datasheets.raspberrypi.org/bcm2711/bcm2711-peripherals.pdf
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
+    /// GPIO Pull-up/down Register
+    ///
+    /// BCM2837 only.
+    GPPUD [
+        /// Controls the actuation of the internal pull-up/down control line to ALL the GPIO pins.
+        PUD OFFSET(0) NUMBITS(2) [
+            Off = 0b00,
+            PullDown = 0b01,
+            PullUp = 0b10
+        ]
+    ],
+
+    /// GPIO Pull-up/down Clock Register 0
+    ///
+    /// BCM2837 only.
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
+    ],
+
+    /// GPIO Pull-up / Pull-down Register 0
+    ///
+    /// BCM2711 only.
+    GPIO_PUP_PDN_CNTRL_REG0 [
+        /// Pin 15
+        GPIO_PUP_PDN_CNTRL15 OFFSET(30) NUMBITS(2) [
+            NoResistor = 0b00,
+            PullUp = 0b01
+        ],
+
+        /// Pin 14
+        GPIO_PUP_PDN_CNTRL14 OFFSET(28) NUMBITS(2) [
+            NoResistor = 0b00,
+            PullUp = 0b01
+        ]
+    ]
+}
+
+register_structs! {
+    #[allow(non_snake_case)]
+    RegisterBlock {
+        (0x00 => _reserved1),
+        (0x04 => GPFSEL1: ReadWrite<u32, GPFSEL1::Register>),
+        (0x08 => _reserved2),
+        (0x94 => GPPUD: ReadWrite<u32, GPPUD::Register>),
+        (0x98 => GPPUDCLK0: ReadWrite<u32, GPPUDCLK0::Register>),
+        (0x9C => _reserved3),
+        (0xE4 => GPIO_PUP_PDN_CNTRL_REG0: ReadWrite<u32, GPIO_PUP_PDN_CNTRL_REG0::Register>),
+        (0xE8 => @END),
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
+pub struct GPIOInner {
+    registers: Registers,
+}
+
+// Export the inner struct so that BSPs can use it for the panic handler.
+pub use GPIOInner as PanicGPIO;
+
+/// Representation of the GPIO HW.
+pub struct GPIO {
+    inner: NullLock<GPIOInner>,
+}
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+impl GPIOInner {
+    /// Create an instance.
+    ///
+    /// # Safety
+    ///
+    /// - The user must ensure to provide a correct MMIO start address.
+    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
+        Self {
+            registers: Registers::new(mmio_start_addr),
+        }
+    }
+
+    /// Disable pull-up/down on pins 14 and 15.
+    #[cfg(feature = "bsp_rpi3")]
+    fn disable_pud_14_15_bcm2837(&mut self) {
+        use crate::cpu;
+
+        // Make an educated guess for a good delay value (Sequence described in the BCM2837
+        // peripherals PDF).
+        //
+        // - According to Wikipedia, the fastest Pi3 clocks around 1.4 GHz.
+        // - The Linux 2837 GPIO driver waits 1 µs between the steps.
+        //
+        // So lets try to be on the safe side and default to 2000 cycles, which would equal 1 µs
+        // would the CPU be clocked at 2 GHz.
+        const DELAY: usize = 2000;
+
+        self.registers.GPPUD.write(GPPUD::PUD::Off);
+        cpu::spin_for_cycles(DELAY);
+
+        self.registers
+            .GPPUDCLK0
+            .write(GPPUDCLK0::PUDCLK15::AssertClock + GPPUDCLK0::PUDCLK14::AssertClock);
+        cpu::spin_for_cycles(DELAY);
+
+        self.registers.GPPUD.write(GPPUD::PUD::Off);
+        self.registers.GPPUDCLK0.set(0);
+    }
+
+    /// Disable pull-up/down on pins 14 and 15.
+    #[cfg(feature = "bsp_rpi4")]
+    fn disable_pud_14_15_bcm2711(&mut self) {
+        self.registers.GPIO_PUP_PDN_CNTRL_REG0.write(
+            GPIO_PUP_PDN_CNTRL_REG0::GPIO_PUP_PDN_CNTRL15::PullUp
+                + GPIO_PUP_PDN_CNTRL_REG0::GPIO_PUP_PDN_CNTRL14::PullUp,
+        );
+    }
+
+    /// Map PL011 UART as standard output.
+    ///
+    /// TX to pin 14
+    /// RX to pin 15
+    pub fn map_pl011_uart(&mut self) {
+        // Select the UART on pins 14 and 15.
+        self.registers
+            .GPFSEL1
+            .modify(GPFSEL1::FSEL15::AltFunc0 + GPFSEL1::FSEL14::AltFunc0);
+
+        // Disable pull-up/down on pins 14 and 15.
+        #[cfg(feature = "bsp_rpi3")]
+        self.disable_pud_14_15_bcm2837();
+
+        #[cfg(feature = "bsp_rpi4")]
+        self.disable_pud_14_15_bcm2711();
+    }
+}
+
+impl GPIO {
+    /// Create an instance.
+    ///
+    /// # Safety
+    ///
+    /// - The user must ensure to provide a correct MMIO start address.
+    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
+        Self {
+            inner: NullLock::new(GPIOInner::new(mmio_start_addr)),
+        }
+    }
+
+    /// Concurrency safe version of `GPIOInner.map_pl011_uart()`
+    pub fn map_pl011_uart(&self) {
+        self.inner.lock(|inner| inner.map_pl011_uart())
+    }
+}
+
+//------------------------------------------------------------------------------
+// OS Interface Code
+//------------------------------------------------------------------------------
+use synchronization::interface::Mutex;
+
+impl driver::interface::DeviceDriver for GPIO {
+    fn compatible(&self) -> &'static str {
+        "BCM GPIO"
+    }
+}

diff -uNr 04_safe_globals/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs 05_drivers_gpio_uart/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
--- 04_safe_globals/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
+++ 05_drivers_gpio_uart/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
@@ -0,0 +1,403 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
+
+//! PL011 UART driver.
+//!
+//! # Resources
+//!
+//! - <https://github.com/raspberrypi/documentation/files/1888662/BCM2837-ARM-Peripherals.-.Revised.-.V2-1.pdf>
+//! - <https://developer.arm.com/documentation/ddi0183/latest>
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
+// Descriptions taken from "PrimeCell UART (PL011) Technical Reference Manual" r1p5.
+register_bitfields! {
+    u32,
+
+    /// Flag Register.
+    FR [
+        /// Transmit FIFO empty. The meaning of this bit depends on the state of the FEN bit in the
+        /// Line Control Register, LCR_H.
+        ///
+        /// - If the FIFO is disabled, this bit is set when the transmit holding register is empty.
+        /// - If the FIFO is enabled, the TXFE bit is set when the transmit FIFO is empty.
+        /// - This bit does not indicate if there is data in the transmit shift register.
+        TXFE OFFSET(7) NUMBITS(1) [],
+
+        /// Transmit FIFO full. The meaning of this bit depends on the state of the FEN bit in the
+        /// LCR_H Register.
+        ///
+        /// - If the FIFO is disabled, this bit is set when the transmit holding register is full.
+        /// - If the FIFO is enabled, the TXFF bit is set when the transmit FIFO is full.
+        TXFF OFFSET(5) NUMBITS(1) [],
+
+        /// Receive FIFO empty. The meaning of this bit depends on the state of the FEN bit in the
+        /// LCR_H Register.
+        ///
+        /// If the FIFO is disabled, this bit is set when the receive holding register is empty. If
+        /// the FIFO is enabled, the RXFE bit is set when the receive FIFO is empty.
+
+        /// Receive FIFO empty. The meaning of this bit depends on the state of the FEN bit in the
+        /// LCR_H Register.
+        ///
+        /// - If the FIFO is disabled, this bit is set when the receive holding register is empty.
+        /// - If the FIFO is enabled, the RXFE bit is set when the receive FIFO is empty.
+        RXFE OFFSET(4) NUMBITS(1) [],
+
+        /// UART busy. If this bit is set to 1, the UART is busy transmitting data. This bit remains
+        /// set until the complete byte, including all the stop bits, has been sent from the shift
+        /// register.
+        ///
+        /// This bit is set as soon as the transmit FIFO becomes non-empty, regardless of whether
+        /// the UART is enabled or not.
+        BUSY OFFSET(3) NUMBITS(1) []
+    ],
+
+    /// Integer Baud Rate Divisor.
+    IBRD [
+        /// The integer baud rate divisor.
+        BAUD_DIVINT OFFSET(0) NUMBITS(16) []
+    ],
+
+    /// Fractional Baud Rate Divisor.
+    FBRD [
+        ///  The fractional baud rate divisor.
+        BAUD_DIVFRAC OFFSET(0) NUMBITS(6) []
+    ],
+
+    /// Line Control Register.
+    LCR_H [
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
+        /// registers.
+        ///
+        /// 1 = Transmit and receive FIFO buffers are enabled (FIFO mode).
+        FEN  OFFSET(4) NUMBITS(1) [
+            FifosDisabled = 0,
+            FifosEnabled = 1
+        ]
+    ],
+
+    /// Control Register.
+    CR [
+        /// Receive enable. If this bit is set to 1, the receive section of the UART is enabled.
+        /// Data reception occurs for either UART signals or SIR signals depending on the setting of
+        /// the SIREN bit. When the UART is disabled in the middle of reception, it completes the
+        /// current character before stopping.
+        RXE OFFSET(9) NUMBITS(1) [
+            Disabled = 0,
+            Enabled = 1
+        ],
+
+        /// Transmit enable. If this bit is set to 1, the transmit section of the UART is enabled.
+        /// Data transmission occurs for either UART signals, or SIR signals depending on the
+        /// setting of the SIREN bit. When the UART is disabled in the middle of transmission, it
+        /// completes the current character before stopping.
+        TXE OFFSET(8) NUMBITS(1) [
+            Disabled = 0,
+            Enabled = 1
+        ],
+
+        /// UART enable:
+        ///
+        /// 0 = UART is disabled. If the UART is disabled in the middle of transmission or
+        /// reception, it completes the current character before stopping.
+        ///
+        /// 1 = The UART is enabled. Data transmission and reception occurs for either UART signals
+        /// or SIR signals depending on the setting of the SIREN bit
+        UARTEN OFFSET(0) NUMBITS(1) [
+            /// If the UART is disabled in the middle of transmission or reception, it completes the
+            /// current character before stopping.
+            Disabled = 0,
+            Enabled = 1
+        ]
+    ],
+
+    /// Interrupt Clear Register.
+    ICR [
+        /// Meta field for all pending interrupts.
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
+        (0x2c => LCR_H: WriteOnly<u32, LCR_H::Register>),
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
+#[derive(PartialEq)]
+enum BlockingMode {
+    Blocking,
+    NonBlocking,
+}
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
+    /// - The user must ensure to provide a correct MMIO start address.
+    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
+        Self {
+            registers: Registers::new(mmio_start_addr),
+            chars_written: 0,
+            chars_read: 0,
+        }
+    }
+
+    /// Set up baud rate and characteristics.
+    ///
+    /// This results in 8N1 and 921_600 baud.
+    ///
+    /// The calculation for the BRD is (we set the clock to 48 MHz in config.txt):
+    /// `(48_000_000 / 16) / 921_600 = 3.2552083`.
+    ///
+    /// This means the integer part is `3` and goes into the `IBRD`.
+    /// The fractional part is `0.2552083`.
+    ///
+    /// `FBRD` calculation according to the PL011 Technical Reference Manual:
+    /// `INTEGER((0.2552083 * 64) + 0.5) = 16`.
+    ///
+    /// Therefore, the generated baud rate divider is: `3 + 16/64 = 3.25`. Which results in a
+    /// genrated baud rate of `48_000_000 / (16 * 3.25) = 923_077`.
+    ///
+    /// Error = `((923_077 - 921_600) / 921_600) * 100 = 0.16modulo`.
+    pub fn init(&mut self) {
+        // Execution can arrive here while there are still characters queued in the TX FIFO and
+        // actively being sent out by the UART hardware. If the UART is turned off in this case,
+        // those queued characters would be lost.
+        //
+        // For example, this can happen during runtime on a call to panic!(), because panic!()
+        // initializes its own UART instance and calls init().
+        //
+        // Hence, flush first to ensure all pending characters are transmitted.
+        self.flush();
+
+        // Turn the UART off temporarily.
+        self.registers.CR.set(0);
+
+        // Clear all pending interrupts.
+        self.registers.ICR.write(ICR::ALL::CLEAR);
+
+        // From the PL011 Technical Reference Manual:
+        //
+        // The LCR_H, IBRD, and FBRD registers form the single 30-bit wide LCR Register that is
+        // updated on a single write strobe generated by a LCR_H write. So, to internally update the
+        // contents of IBRD or FBRD, a LCR_H write must always be performed at the end.
+        //
+        // Set the baud rate, 8N1 and FIFO enabled.
+        self.registers.IBRD.write(IBRD::BAUD_DIVINT.val(3));
+        self.registers.FBRD.write(FBRD::BAUD_DIVFRAC.val(16));
+        self.registers
+            .LCR_H
+            .write(LCR_H::WLEN::EightBit + LCR_H::FEN::FifosEnabled);
+
+        // Turn the UART on.
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
+
+    /// Block execution until the last buffered character has been physically put on the TX wire.
+    fn flush(&self) {
+        // Spin until the busy bit is cleared.
+        while self.registers.FR.matches_all(FR::BUSY::SET) {
+            cpu::nop();
+        }
+    }
+
+    /// Retrieve a character.
+    fn read_char_converting(&mut self, blocking_mode: BlockingMode) -> Option<char> {
+        // If RX FIFO is empty,
+        if self.registers.FR.matches_all(FR::RXFE::SET) {
+            // immediately return in non-blocking mode.
+            if blocking_mode == BlockingMode::NonBlocking {
+                return None;
+            }
+
+            // Otherwise, wait until a char was received.
+            while self.registers.FR.matches_all(FR::RXFE::SET) {
+                cpu::nop();
+            }
+        }
+
+        // Read one character.
+        let mut ret = self.registers.DR.get() as u8 as char;
+
+        // Convert carrige return to newline.
+        if ret == '\r' {
+            ret = '\n'
+        }
+
+        // Update statistics.
+        self.chars_read += 1;
+
+        Some(ret)
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
+    /// Create an instance.
+    ///
+    /// # Safety
+    ///
+    /// - The user must ensure to provide a correct MMIO start address.
+    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
+        Self {
+            inner: NullLock::new(PL011UartInner::new(mmio_start_addr)),
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
+    fn compatible(&self) -> &'static str {
+        "BCM PL011 UART"
+    }
+
+    unsafe fn init(&self) -> Result<(), &'static str> {
+        self.inner.lock(|inner| inner.init());
+
+        Ok(())
+    }
+}
+
+impl console::interface::Write for PL011Uart {
+    /// Passthrough of `args` to the `core::fmt::Write` implementation, but guarded by a Mutex to
+    /// serialize access.
+    fn write_char(&self, c: char) {
+        self.inner.lock(|inner| inner.write_char(c));
+    }
+
+    fn write_fmt(&self, args: core::fmt::Arguments) -> fmt::Result {
+        // Fully qualified syntax for the call to `core::fmt::Write::write:fmt()` to increase
+        // readability.
+        self.inner.lock(|inner| fmt::Write::write_fmt(inner, args))
+    }
+
+    fn flush(&self) {
+        // Spin until TX FIFO empty is set.
+        self.inner.lock(|inner| inner.flush());
+    }
+}
+
+impl console::interface::Read for PL011Uart {
+    fn read_char(&self) -> char {
+        self.inner
+            .lock(|inner| inner.read_char_converting(BlockingMode::Blocking).unwrap())
+    }
+
+    fn clear_rx(&self) {
+        // Read from the RX FIFO until it is indicating empty.
+        while self
+            .inner
+            .lock(|inner| inner.read_char_converting(BlockingMode::NonBlocking))
+            .is_some()
+        {}
+    }
+}
+
+impl console::interface::Statistics for PL011Uart {
+    fn chars_written(&self) -> usize {
+        self.inner.lock(|inner| inner.chars_written)
+    }
+
+    fn chars_read(&self) -> usize {
+        self.inner.lock(|inner| inner.chars_read)
+    }
+}

diff -uNr 04_safe_globals/src/bsp/device_driver/bcm.rs 05_drivers_gpio_uart/src/bsp/device_driver/bcm.rs
--- 04_safe_globals/src/bsp/device_driver/bcm.rs
+++ 05_drivers_gpio_uart/src/bsp/device_driver/bcm.rs
@@ -0,0 +1,11 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
+
+//! BCM driver top level.
+
+mod bcm2xxx_gpio;
+mod bcm2xxx_pl011_uart;
+
+pub use bcm2xxx_gpio::*;
+pub use bcm2xxx_pl011_uart::*;

diff -uNr 04_safe_globals/src/bsp/device_driver/common.rs 05_drivers_gpio_uart/src/bsp/device_driver/common.rs
--- 04_safe_globals/src/bsp/device_driver/common.rs
+++ 05_drivers_gpio_uart/src/bsp/device_driver/common.rs
@@ -0,0 +1,38 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2020-2021 Andre Richter <andre.o.richter@gmail.com>
+
+//! Common device driver code.
+
+use core::{marker::PhantomData, ops};
+
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
+pub struct MMIODerefWrapper<T> {
+    start_addr: usize,
+    phantom: PhantomData<fn() -> T>,
+}
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+impl<T> MMIODerefWrapper<T> {
+    /// Create an instance.
+    pub const unsafe fn new(start_addr: usize) -> Self {
+        Self {
+            start_addr,
+            phantom: PhantomData,
+        }
+    }
+}
+
+impl<T> ops::Deref for MMIODerefWrapper<T> {
+    type Target = T;
+
+    fn deref(&self) -> &Self::Target {
+        unsafe { &*(self.start_addr as *const _) }
+    }
+}

diff -uNr 04_safe_globals/src/bsp/device_driver.rs 05_drivers_gpio_uart/src/bsp/device_driver.rs
--- 04_safe_globals/src/bsp/device_driver.rs
+++ 05_drivers_gpio_uart/src/bsp/device_driver.rs
@@ -0,0 +1,12 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
+
+//! Device driver.
+
+#[cfg(any(feature = "bsp_rpi3", feature = "bsp_rpi4"))]
+mod bcm;
+mod common;
+
+#[cfg(any(feature = "bsp_rpi3", feature = "bsp_rpi4"))]
+pub use bcm::*;

diff -uNr 04_safe_globals/src/bsp/raspberrypi/console.rs 05_drivers_gpio_uart/src/bsp/raspberrypi/console.rs
--- 04_safe_globals/src/bsp/raspberrypi/console.rs
+++ 05_drivers_gpio_uart/src/bsp/raspberrypi/console.rs
@@ -4,113 +4,34 @@

 //! BSP console facilities.

-use crate::{console, synchronization, synchronization::NullLock};
+use super::memory;
+use crate::{bsp::device_driver, console};
 use core::fmt;

 //--------------------------------------------------------------------------------------------------
-// Private Definitions
+// Public Code
 //--------------------------------------------------------------------------------------------------

-/// A mystical, magical device for generating QEMU output out of the void.
+/// In case of a panic, the panic handler uses this function to take a last shot at printing
+/// something before the system is halted.
 ///
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
-//--------------------------------------------------------------------------------------------------
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
-
-        self.chars_written += 1;
-    }
-}
-
-/// Implementing `core::fmt::Write` enables usage of the `format_args!` macros, which in turn are
-/// used to implement the `kernel`'s `print!` and `println!` macros. By implementing `write_str()`,
-/// we get `write_fmt()` automatically.
+/// We try to init panic-versions of the GPIO and the UART. The panic versions are not protected
+/// with synchronization primitives, which increases chances that we get to print something, even
+/// when the kernel's default GPIO or UART instances happen to be locked at the time of the panic.
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
+    let mut panic_gpio = device_driver::PanicGPIO::new(memory::map::mmio::GPIO_START);
+    let mut panic_uart = device_driver::PanicUart::new(memory::map::mmio::PL011_UART_START);
+
+    panic_gpio.map_pl011_uart();
+    panic_uart.init();
+    panic_uart
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
-        self.inner.lock(|inner| fmt::Write::write_fmt(inner, args))
-    }
-}
-
-impl console::interface::Statistics for QEMUOutput {
-    fn chars_written(&self) -> usize {
-        self.inner.lock(|inner| inner.chars_written)
-    }
+    &super::PL011_UART
 }

diff -uNr 04_safe_globals/src/bsp/raspberrypi/driver.rs 05_drivers_gpio_uart/src/bsp/raspberrypi/driver.rs
--- 04_safe_globals/src/bsp/raspberrypi/driver.rs
+++ 05_drivers_gpio_uart/src/bsp/raspberrypi/driver.rs
@@ -0,0 +1,49 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
+
+//! BSP driver support.
+
+use crate::driver;
+
+//--------------------------------------------------------------------------------------------------
+// Private Definitions
+//--------------------------------------------------------------------------------------------------
+
+/// Device Driver Manager type.
+struct BSPDriverManager {
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

diff -uNr 04_safe_globals/src/bsp/raspberrypi/memory.rs 05_drivers_gpio_uart/src/bsp/raspberrypi/memory.rs
--- 04_safe_globals/src/bsp/raspberrypi/memory.rs
+++ 05_drivers_gpio_uart/src/bsp/raspberrypi/memory.rs
@@ -17,6 +17,38 @@
 }

 //--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
+/// The board's physical memory map.
+#[rustfmt::skip]
+pub(super) mod map {
+
+    pub const GPIO_OFFSET:         usize = 0x0020_0000;
+    pub const UART_OFFSET:         usize = 0x0020_1000;
+
+    /// Physical devices.
+    #[cfg(feature = "bsp_rpi3")]
+    pub mod mmio {
+        use super::*;
+
+        pub const START:            usize =         0x3F00_0000;
+        pub const GPIO_START:       usize = START + GPIO_OFFSET;
+        pub const PL011_UART_START: usize = START + UART_OFFSET;
+    }
+
+    /// Physical devices.
+    #[cfg(feature = "bsp_rpi4")]
+    pub mod mmio {
+        use super::*;
+
+        pub const START:            usize =         0xFE00_0000;
+        pub const GPIO_START:       usize = START + GPIO_OFFSET;
+        pub const PL011_UART_START: usize = START + UART_OFFSET;
+    }
+}
+
+//--------------------------------------------------------------------------------------------------
 // Public Code
 //--------------------------------------------------------------------------------------------------


diff -uNr 04_safe_globals/src/bsp/raspberrypi.rs 05_drivers_gpio_uart/src/bsp/raspberrypi.rs
--- 04_safe_globals/src/bsp/raspberrypi.rs
+++ 05_drivers_gpio_uart/src/bsp/raspberrypi.rs
@@ -6,4 +6,33 @@

 pub mod console;
 pub mod cpu;
+pub mod driver;
 pub mod memory;
+
+//--------------------------------------------------------------------------------------------------
+// Global instances
+//--------------------------------------------------------------------------------------------------
+use super::device_driver;
+
+static GPIO: device_driver::GPIO =
+    unsafe { device_driver::GPIO::new(memory::map::mmio::GPIO_START) };
+
+static PL011_UART: device_driver::PL011Uart =
+    unsafe { device_driver::PL011Uart::new(memory::map::mmio::PL011_UART_START) };
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

diff -uNr 04_safe_globals/src/bsp.rs 05_drivers_gpio_uart/src/bsp.rs
--- 04_safe_globals/src/bsp.rs
+++ 05_drivers_gpio_uart/src/bsp.rs
@@ -4,6 +4,8 @@

 //! Conditional reexporting of Board Support Packages.

+mod device_driver;
+
 #[cfg(any(feature = "bsp_rpi3", feature = "bsp_rpi4"))]
 mod raspberrypi;


diff -uNr 04_safe_globals/src/console.rs 05_drivers_gpio_uart/src/console.rs
--- 04_safe_globals/src/console.rs
+++ 05_drivers_gpio_uart/src/console.rs
@@ -14,8 +14,25 @@

     /// Console write functions.
     pub trait Write {
+        /// Write a single character.
+        fn write_char(&self, c: char);
+
         /// Write a Rust format string.
         fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result;
+
+        /// Block until the last buffered character has been physically put on the TX wire.
+        fn flush(&self);
+    }
+
+    /// Console read functions.
+    pub trait Read {
+        /// Read a single character.
+        fn read_char(&self) -> char {
+            ' '
+        }
+
+        /// Clear RX buffers, if any.
+        fn clear_rx(&self);
     }

     /// Console statistics.
@@ -24,8 +41,13 @@
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

diff -uNr 04_safe_globals/src/cpu.rs 05_drivers_gpio_uart/src/cpu.rs
--- 04_safe_globals/src/cpu.rs
+++ 05_drivers_gpio_uart/src/cpu.rs
@@ -13,4 +13,7 @@
 //--------------------------------------------------------------------------------------------------
 // Architectural Public Reexports
 //--------------------------------------------------------------------------------------------------
-pub use arch_cpu::wait_forever;
+pub use arch_cpu::{nop, wait_forever};
+
+#[cfg(feature = "bsp_rpi3")]
+pub use arch_cpu::spin_for_cycles;

diff -uNr 04_safe_globals/src/driver.rs 05_drivers_gpio_uart/src/driver.rs
--- 04_safe_globals/src/driver.rs
+++ 05_drivers_gpio_uart/src/driver.rs
@@ -0,0 +1,44 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
+
+//! Driver support.
+
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
+/// Driver interfaces.
+pub mod interface {
+    /// Device Driver functions.
+    pub trait DeviceDriver {
+        /// Return a compatibility string for identifying the driver.
+        fn compatible(&self) -> &'static str;
+
+        /// Called by the kernel to bring up the device.
+        ///
+        /// # Safety
+        ///
+        /// - During init, drivers might do stuff with system-wide impact.
+        unsafe fn init(&self) -> Result<(), &'static str> {
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

diff -uNr 04_safe_globals/src/main.rs 05_drivers_gpio_uart/src/main.rs
--- 04_safe_globals/src/main.rs
+++ 05_drivers_gpio_uart/src/main.rs
@@ -106,6 +106,8 @@
 //!
 //! [`runtime_init::runtime_init()`]: runtime_init/fn.runtime_init.html

+#![allow(clippy::upper_case_acronyms)]
+#![feature(const_fn_fn_ptr_basics)]
 #![feature(format_args_nl)]
 #![feature(global_asm)]
 #![feature(panic_info_message)]
@@ -116,6 +118,7 @@
 mod bsp;
 mod console;
 mod cpu;
+mod driver;
 mod memory;
 mod panic_wait;
 mod print;
@@ -127,16 +130,54 @@
 /// # Safety
 ///
 /// - Only a single core must be active and running this function.
+/// - The init calls in this function must appear in the correct order.
 unsafe fn kernel_init() -> ! {
-    use console::interface::Statistics;
+    use driver::interface::DriverManager;

-    println!("[0] Hello from Rust!");
+    for i in bsp::driver::driver_manager().all_device_drivers().iter() {
+        if let Err(x) = i.init() {
+            panic!("Error loading driver: {}: {}", i.compatible(), x);
+        }
+    }
+    bsp::driver::driver_manager().post_device_driver_init();
+    // println! is usable from here on.
+
+    // Transition from unsafe to safe.
+    kernel_main()
+}
+
+/// The main function running after the early init.
+fn kernel_main() -> ! {
+    use bsp::console::console;
+    use console::interface::All;
+    use driver::interface::DriverManager;
+
+    println!(
+        "[0] {} version {}",
+        env!("CARGO_PKG_NAME"),
+        env!("CARGO_PKG_VERSION")
+    );
+    println!("[1] Booting on: {}", bsp::board_name());
+
+    println!("[2] Drivers loaded:");
+    for (i, driver) in bsp::driver::driver_manager()
+        .all_device_drivers()
+        .iter()
+        .enumerate()
+    {
+        println!("      {}. {}", i + 1, driver.compatible());
+    }

     println!(
-        "[1] Chars written: {}",
+        "[3] Chars written: {}",
         bsp::console::console().chars_written()
     );
+    println!("[4] Echoing input now");

-    println!("[2] Stopping here.");
-    cpu::wait_forever()
+    // Discard any spurious received characters before going into echo mode.
+    console().clear_rx();
+    loop {
+        let c = bsp::console::console().read_char();
+        bsp::console::console().write_char(c);
+    }
 }

diff -uNr 04_safe_globals/src/panic_wait.rs 05_drivers_gpio_uart/src/panic_wait.rs
--- 04_safe_globals/src/panic_wait.rs
+++ 05_drivers_gpio_uart/src/panic_wait.rs
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
+/// Carbon copy from <https://doc.rust-lang.org/src/std/macros.rs.html>
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
