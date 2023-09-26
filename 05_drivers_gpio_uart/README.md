# Tutorial 05 - Drivers: GPIO and UART

## tl;dr

- Drivers for the real `UART` and the `GPIO` controller are added.
- **For the first time, we will be able to run the code on the real hardware** (scroll down for
  instructions).

## Introduction

Now that we enabled safe globals in the previous tutorial, the infrastructure is laid for adding the
first real device drivers. We throw out the magic QEMU console and introduce a `driver manager`,
which allows the `BSP` to register device drivers with the `kernel`.

## Driver Manager

The first step consists of adding a `driver subsystem` to the kernel. The corresponding code will
live in `src/driver.rs`. The subsystem introduces `interface::DeviceDriver`, a common trait that
every device driver will need to implement and that is known to the kernel. A global
`DRIVER_MANAGER` instance (of type `DriverManager`) that is instantiated in the same file serves as
the central entity that can be called to manage all things device drivers in the kernel. For
example, by using the globally accessible `crate::driver::driver_manager().register_driver(...)`,
any code can can register an object with static lifetime that implements the
`interface::DeviceDriver` trait.

During kernel init, a call to `crate::driver::driver_manager().init_drivers(...)` will let the
driver manager loop over all registered drivers and kick off their initialization, and also execute
an optional `post-init callback` that can be registered alongside the driver. For example, this
mechanism is used to switch over to the `UART` driver as the main system console after the `UART`
driver has been initialized.

## BSP Driver Implementation

In `src/bsp/raspberrypi/driver.rs`, the function `init()` takes care of registering the `UART` and
`GPIO` drivers. It is therefore important that during kernel init, the correct order of (i) first
initializing the BSP driver subsystem, and only then (ii) calling the `driver_manager()` is
followed, like the following excerpt from `main.rs` shows:

```rust
unsafe fn kernel_init() -> ! {
    // Initialize the BSP driver subsystem.
    if let Err(x) = bsp::driver::init() {
        panic!("Error initializing BSP driver subsystem: {}", x);
    }

    // Initialize all device drivers.
    driver::driver_manager().init_drivers();
    // println! is usable from here on.
```



The drivers themselves are stored in `src/bsp/device_driver`, and can be reused between `BSP`s. The
first driver added in these tutorials is the `PL011Uart` driver: It implements the
`console::interface::*` traits and is from now on used as the main system console. The second driver
is the `GPIO` driver, which pinmuxes (that is, routing signals from inside the `SoC` to actual HW
pins) the RPi's PL011 UART accordingly. Note how the `GPIO` driver differentiates between **RPi 3**
and **RPi 4**. Their HW is different, so we have to account for it in SW.

The `BSP`s now also contain a memory map in `src/bsp/raspberrypi/memory.rs`. It provides the
Raspberry's `MMIO` addresses which are used by the `BSP` to instantiate the respective device
drivers, so that the driver code knows where to find the device's registers in memory.

## Boot it from SD card

Since we have real `UART` output now, we can run the code on the real hardware. Building is
differentiated between the **RPi 3** and the **RPi 4** due to before mentioned differences in the
`GPIO` driver. By default, all `Makefile` targets will build for the **RPi 3**. In order to build
for the the **RPi 4**, prepend `BSP=rpi4` to each target. For example:

```console
$ BSP=rpi4 make
$ BSP=rpi4 make doc
```

Unfortunately, QEMU does not yet support the **RPi 4**, so `BSP=rpi4 make qemu` won't work.

**Some steps for preparing the SD card differ between RPi 3 and RPi 4, so be careful in the
following.**

### Common for both

1. Make a single `FAT32` partition named `boot`.
2. On the card, generate a file named `config.txt` with the following contents:

```txt
arm_64bit=1
init_uart_clock=48000000
```
### RPi 3

3. Copy the following files from the [Raspberry Pi firmware repo](https://github.com/raspberrypi/firmware/tree/master/boot) onto the SD card:
    - [bootcode.bin](https://github.com/raspberrypi/firmware/raw/master/boot/bootcode.bin)
    - [fixup.dat](https://github.com/raspberrypi/firmware/raw/master/boot/fixup.dat)
    - [start.elf](https://github.com/raspberrypi/firmware/raw/master/boot/start.elf)
4. Run `make`.

### RPi 4

3. Copy the following files from the [Raspberry Pi firmware repo](https://github.com/raspberrypi/firmware/tree/master/boot) onto the SD card:
    - [fixup4.dat](https://github.com/raspberrypi/firmware/raw/master/boot/fixup4.dat)
    - [start4.elf](https://github.com/raspberrypi/firmware/raw/master/boot/start4.elf)
    - [bcm2711-rpi-4-b.dtb](https://github.com/raspberrypi/firmware/raw/master/boot/bcm2711-rpi-4-b.dtb)
4. Run `BSP=rpi4 make`.


_**Note**: Should it not work on your RPi 4, try renaming `start4.elf` to `start.elf` (without the 4)
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
    - **NOTE**: TX (transmit) wire connects to the RX (receive) pin.
    - Make sure that you **DID NOT** connect the power pin of the USB serial. Only RX/TX and GND.
8. Connect the RPi to the (USB) power cable and observe the output:

```console
Miniterm 1.0

[MT] ⏳ Waiting for /dev/ttyUSB0
[MT] ✅ Serial connected
[0] mingo version 0.5.0
[1] Booting on: Raspberry Pi 3
[2] Drivers loaded:
      1. BCM PL011 UART
      2. BCM GPIO
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
 edition = "2021"

@@ -9,8 +9,8 @@

 [features]
 default = []
-bsp_rpi3 = []
-bsp_rpi4 = []
+bsp_rpi3 = ["tock-registers"]
+bsp_rpi4 = ["tock-registers"]

 [[bin]]
 name = "kernel"
@@ -22,6 +22,9 @@

 [dependencies]

+# Optional dependencies
+tock-registers = { version = "0.8.x", default-features = false, features = ["register_types"], optional = true }
+
 # Platform specific dependencies
 [target.'cfg(target_arch = "aarch64")'.dependencies]
 aarch64-cpu = { version = "9.x.x" }

diff -uNr 04_safe_globals/Makefile 05_drivers_gpio_uart/Makefile
--- 04_safe_globals/Makefile
+++ 05_drivers_gpio_uart/Makefile
@@ -13,6 +13,9 @@
 # Default to the RPi3.
 BSP ?= rpi3

+# Default to a serial device name that is common in Linux.
+DEV_SERIAL ?= /dev/ttyUSB0
+


 ##--------------------------------------------------------------------------------------------------
@@ -88,6 +91,7 @@

 EXEC_QEMU          = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)
 EXEC_TEST_DISPATCH = ruby ../common/tests/dispatch.rb
+EXEC_MINITERM      = ruby ../common/serial/miniterm.rb

 ##------------------------------------------------------------------------------
 ## Dockerization
@@ -95,18 +99,26 @@
 DOCKER_CMD            = docker run -t --rm -v $(shell pwd):/work/tutorial -w /work/tutorial
 DOCKER_CMD_INTERACT   = $(DOCKER_CMD) -i
 DOCKER_ARG_DIR_COMMON = -v $(shell pwd)/../common:/work/common
+DOCKER_ARG_DEV        = --privileged -v /dev:/dev

 # DOCKER_IMAGE defined in include file (see top of this file).
 DOCKER_QEMU  = $(DOCKER_CMD_INTERACT) $(DOCKER_IMAGE)
 DOCKER_TOOLS = $(DOCKER_CMD) $(DOCKER_IMAGE)
 DOCKER_TEST  = $(DOCKER_CMD) $(DOCKER_ARG_DIR_COMMON) $(DOCKER_IMAGE)

+# Dockerize commands, which require USB device passthrough, only on Linux.
+ifeq ($(shell uname -s),Linux)
+    DOCKER_CMD_DEV = $(DOCKER_CMD_INTERACT) $(DOCKER_ARG_DEV)
+
+    DOCKER_MINITERM = $(DOCKER_CMD_DEV) $(DOCKER_ARG_DIR_COMMON) $(DOCKER_IMAGE)
+endif
+


 ##--------------------------------------------------------------------------------------------------
 ## Targets
 ##--------------------------------------------------------------------------------------------------
-.PHONY: all doc qemu clippy clean readelf objdump nm check
+.PHONY: all doc qemu miniterm clippy clean readelf objdump nm check

 all: $(KERNEL_BIN)

@@ -156,9 +168,16 @@
 qemu: $(KERNEL_BIN)
 	$(call color_header, "Launching QEMU")
 	@$(DOCKER_QEMU) $(EXEC_QEMU) $(QEMU_RELEASE_ARGS) -kernel $(KERNEL_BIN)
+
 endif

 ##------------------------------------------------------------------------------
+## Connect to the target's serial
+##------------------------------------------------------------------------------
+miniterm:
+	@$(DOCKER_MINITERM) $(EXEC_MINITERM) $(DEV_SERIAL)
+
+##------------------------------------------------------------------------------
 ## Run clippy
 ##------------------------------------------------------------------------------
 clippy:

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
@@ -0,0 +1,228 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
+
+//! GPIO Driver.
+
+use crate::{
+    bsp::device_driver::common::MMIODerefWrapper, driver, synchronization,
+    synchronization::NullLock,
+};
+use tock_registers::{
+    interfaces::{ReadWriteable, Writeable},
+    register_bitfields, register_structs,
+    registers::ReadWrite,
+};
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
+struct GPIOInner {
+    registers: Registers,
+}
+
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
+/// Representation of the GPIO HW.
+pub struct GPIO {
+    inner: NullLock<GPIOInner>,
+}
+
+//--------------------------------------------------------------------------------------------------
+// Private Code
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
+        // - According to Wikipedia, the fastest RPi4 clocks around 1.5 GHz.
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
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+impl GPIO {
+    pub const COMPATIBLE: &'static str = "BCM GPIO";
+
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
+        Self::COMPATIBLE
+    }
+}

diff -uNr 04_safe_globals/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs 05_drivers_gpio_uart/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
--- 04_safe_globals/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
+++ 05_drivers_gpio_uart/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
@@ -0,0 +1,407 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
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
+use tock_registers::{
+    interfaces::{Readable, Writeable},
+    register_bitfields, register_structs,
+    registers::{ReadOnly, ReadWrite, WriteOnly},
+};
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
+        #[allow(clippy::enum_variant_names)]
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
+struct PL011UartInner {
+    registers: Registers,
+    chars_written: usize,
+    chars_read: usize,
+}
+
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
+/// Representation of the UART.
+pub struct PL011Uart {
+    inner: NullLock<PL011UartInner>,
+}
+
+//--------------------------------------------------------------------------------------------------
+// Private Code
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
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+impl PL011Uart {
+    pub const COMPATIBLE: &'static str = "BCM PL011 UART";
+
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
+        Self::COMPATIBLE
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
+        // Fully qualified syntax for the call to `core::fmt::Write::write_fmt()` to increase
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
+
+impl console::interface::All for PL011Uart {}

diff -uNr 04_safe_globals/src/bsp/device_driver/bcm.rs 05_drivers_gpio_uart/src/bsp/device_driver/bcm.rs
--- 04_safe_globals/src/bsp/device_driver/bcm.rs
+++ 05_drivers_gpio_uart/src/bsp/device_driver/bcm.rs
@@ -0,0 +1,11 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
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
+// Copyright (c) 2020-2023 Andre Richter <andre.o.richter@gmail.com>
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
+// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
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
@@ -4,115 +4,13 @@

 //! BSP console facilities.

-use crate::{console, synchronization, synchronization::NullLock};
-use core::fmt;
-
-//--------------------------------------------------------------------------------------------------
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
-        Ok(())
-    }
-}
+use crate::console;

 //--------------------------------------------------------------------------------------------------
 // Public Code
 //--------------------------------------------------------------------------------------------------

-impl QEMUOutput {
-    /// Create a new instance.
-    pub const fn new() -> QEMUOutput {
-        QEMUOutput {
-            inner: NullLock::new(QEMUOutputInner::new()),
-        }
-    }
-}
-
 /// Return a reference to the console.
 pub fn console() -> &'static dyn console::interface::All {
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
-        // Fully qualified syntax for the call to `core::fmt::Write::write_fmt()` to increase
-        // readability.
-        self.inner.lock(|inner| fmt::Write::write_fmt(inner, args))
-    }
-}
-
-impl console::interface::Statistics for QEMUOutput {
-    fn chars_written(&self) -> usize {
-        self.inner.lock(|inner| inner.chars_written)
-    }
+    &super::driver::PL011_UART
 }
-
-impl console::interface::All for QEMUOutput {}

diff -uNr 04_safe_globals/src/bsp/raspberrypi/driver.rs 05_drivers_gpio_uart/src/bsp/raspberrypi/driver.rs
--- 04_safe_globals/src/bsp/raspberrypi/driver.rs
+++ 05_drivers_gpio_uart/src/bsp/raspberrypi/driver.rs
@@ -0,0 +1,71 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
+
+//! BSP driver support.
+
+use super::memory::map::mmio;
+use crate::{bsp::device_driver, console, driver as generic_driver};
+use core::sync::atomic::{AtomicBool, Ordering};
+
+//--------------------------------------------------------------------------------------------------
+// Global instances
+//--------------------------------------------------------------------------------------------------
+
+static PL011_UART: device_driver::PL011Uart =
+    unsafe { device_driver::PL011Uart::new(mmio::PL011_UART_START) };
+static GPIO: device_driver::GPIO = unsafe { device_driver::GPIO::new(mmio::GPIO_START) };
+
+//--------------------------------------------------------------------------------------------------
+// Private Code
+//--------------------------------------------------------------------------------------------------
+
+/// This must be called only after successful init of the UART driver.
+fn post_init_uart() -> Result<(), &'static str> {
+    console::register_console(&PL011_UART);
+
+    Ok(())
+}
+
+/// This must be called only after successful init of the GPIO driver.
+fn post_init_gpio() -> Result<(), &'static str> {
+    GPIO.map_pl011_uart();
+    Ok(())
+}
+
+fn driver_uart() -> Result<(), &'static str> {
+    let uart_descriptor =
+        generic_driver::DeviceDriverDescriptor::new(&PL011_UART, Some(post_init_uart));
+    generic_driver::driver_manager().register_driver(uart_descriptor);
+
+    Ok(())
+}
+
+fn driver_gpio() -> Result<(), &'static str> {
+    let gpio_descriptor = generic_driver::DeviceDriverDescriptor::new(&GPIO, Some(post_init_gpio));
+    generic_driver::driver_manager().register_driver(gpio_descriptor);
+
+    Ok(())
+}
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+/// Initialize the driver subsystem.
+///
+/// # Safety
+///
+/// See child function calls.
+pub unsafe fn init() -> Result<(), &'static str> {
+    static INIT_DONE: AtomicBool = AtomicBool::new(false);
+    if INIT_DONE.load(Ordering::Relaxed) {
+        return Err("Init already done");
+    }
+
+    driver_uart()?;
+    driver_gpio()?;
+
+    INIT_DONE.store(true, Ordering::Relaxed);
+    Ok(())
+}

diff -uNr 04_safe_globals/src/bsp/raspberrypi/memory.rs 05_drivers_gpio_uart/src/bsp/raspberrypi/memory.rs
--- 04_safe_globals/src/bsp/raspberrypi/memory.rs
+++ 05_drivers_gpio_uart/src/bsp/raspberrypi/memory.rs
@@ -0,0 +1,37 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
+
+//! BSP Memory Management.
+
+//--------------------------------------------------------------------------------------------------
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

diff -uNr 04_safe_globals/src/bsp/raspberrypi.rs 05_drivers_gpio_uart/src/bsp/raspberrypi.rs
--- 04_safe_globals/src/bsp/raspberrypi.rs
+++ 05_drivers_gpio_uart/src/bsp/raspberrypi.rs
@@ -4,5 +4,23 @@

 //! Top-level BSP file for the Raspberry Pi 3 and 4.

-pub mod console;
 pub mod cpu;
+pub mod driver;
+pub mod memory;
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


diff -uNr 04_safe_globals/src/console/null_console.rs 05_drivers_gpio_uart/src/console/null_console.rs
--- 04_safe_globals/src/console/null_console.rs
+++ 05_drivers_gpio_uart/src/console/null_console.rs
@@ -0,0 +1,41 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2022-2023 Andre Richter <andre.o.richter@gmail.com>
+
+//! Null console.
+
+use super::interface;
+use core::fmt;
+
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
+pub struct NullConsole;
+
+//--------------------------------------------------------------------------------------------------
+// Global instances
+//--------------------------------------------------------------------------------------------------
+
+pub static NULL_CONSOLE: NullConsole = NullConsole {};
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+impl interface::Write for NullConsole {
+    fn write_char(&self, _c: char) {}
+
+    fn write_fmt(&self, _args: fmt::Arguments) -> fmt::Result {
+        fmt::Result::Ok(())
+    }
+
+    fn flush(&self) {}
+}
+
+impl interface::Read for NullConsole {
+    fn clear_rx(&self) {}
+}
+
+impl interface::Statistics for NullConsole {}
+impl interface::All for NullConsole {}

diff -uNr 04_safe_globals/src/console.rs 05_drivers_gpio_uart/src/console.rs
--- 04_safe_globals/src/console.rs
+++ 05_drivers_gpio_uart/src/console.rs
@@ -4,7 +4,9 @@

 //! System console.

-use crate::bsp;
+mod null_console;
+
+use crate::synchronization::{self, NullLock};

 //--------------------------------------------------------------------------------------------------
 // Public Definitions
@@ -16,8 +18,25 @@

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
@@ -26,19 +45,37 @@
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
-    pub trait All: Write + Statistics {}
+    pub trait All: Write + Read + Statistics {}
 }

 //--------------------------------------------------------------------------------------------------
+// Global instances
+//--------------------------------------------------------------------------------------------------
+
+static CUR_CONSOLE: NullLock<&'static (dyn interface::All + Sync)> =
+    NullLock::new(&null_console::NULL_CONSOLE);
+
+//--------------------------------------------------------------------------------------------------
 // Public Code
 //--------------------------------------------------------------------------------------------------
+use synchronization::interface::Mutex;
+
+/// Register a new console.
+pub fn register_console(new_console: &'static (dyn interface::All + Sync)) {
+    CUR_CONSOLE.lock(|con| *con = new_console);
+}

-/// Return a reference to the console.
+/// Return a reference to the currently registered console.
 ///
 /// This is the global console used by all printing macros.
 pub fn console() -> &'static dyn interface::All {
-    bsp::console::console()
+    CUR_CONSOLE.lock(|con| *con)
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
@@ -0,0 +1,167 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
+
+//! Driver support.
+
+use crate::{
+    println,
+    synchronization::{interface::Mutex, NullLock},
+};
+
+//--------------------------------------------------------------------------------------------------
+// Private Definitions
+//--------------------------------------------------------------------------------------------------
+
+const NUM_DRIVERS: usize = 5;
+
+struct DriverManagerInner {
+    next_index: usize,
+    descriptors: [Option<DeviceDriverDescriptor>; NUM_DRIVERS],
+}
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
+}
+
+/// Tpye to be used as an optional callback after a driver's init() has run.
+pub type DeviceDriverPostInitCallback = unsafe fn() -> Result<(), &'static str>;
+
+/// A descriptor for device drivers.
+#[derive(Copy, Clone)]
+pub struct DeviceDriverDescriptor {
+    device_driver: &'static (dyn interface::DeviceDriver + Sync),
+    post_init_callback: Option<DeviceDriverPostInitCallback>,
+}
+
+/// Provides device driver management functions.
+pub struct DriverManager {
+    inner: NullLock<DriverManagerInner>,
+}
+
+//--------------------------------------------------------------------------------------------------
+// Global instances
+//--------------------------------------------------------------------------------------------------
+
+static DRIVER_MANAGER: DriverManager = DriverManager::new();
+
+//--------------------------------------------------------------------------------------------------
+// Private Code
+//--------------------------------------------------------------------------------------------------
+
+impl DriverManagerInner {
+    /// Create an instance.
+    pub const fn new() -> Self {
+        Self {
+            next_index: 0,
+            descriptors: [None; NUM_DRIVERS],
+        }
+    }
+}
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+impl DeviceDriverDescriptor {
+    /// Create an instance.
+    pub fn new(
+        device_driver: &'static (dyn interface::DeviceDriver + Sync),
+        post_init_callback: Option<DeviceDriverPostInitCallback>,
+    ) -> Self {
+        Self {
+            device_driver,
+            post_init_callback,
+        }
+    }
+}
+
+/// Return a reference to the global DriverManager.
+pub fn driver_manager() -> &'static DriverManager {
+    &DRIVER_MANAGER
+}
+
+impl DriverManager {
+    /// Create an instance.
+    pub const fn new() -> Self {
+        Self {
+            inner: NullLock::new(DriverManagerInner::new()),
+        }
+    }
+
+    /// Register a device driver with the kernel.
+    pub fn register_driver(&self, descriptor: DeviceDriverDescriptor) {
+        self.inner.lock(|inner| {
+            inner.descriptors[inner.next_index] = Some(descriptor);
+            inner.next_index += 1;
+        })
+    }
+
+    /// Helper for iterating over registered drivers.
+    fn for_each_descriptor<'a>(&'a self, f: impl FnMut(&'a DeviceDriverDescriptor)) {
+        self.inner.lock(|inner| {
+            inner
+                .descriptors
+                .iter()
+                .filter_map(|x| x.as_ref())
+                .for_each(f)
+        })
+    }
+
+    /// Fully initialize all drivers.
+    ///
+    /// # Safety
+    ///
+    /// - During init, drivers might do stuff with system-wide impact.
+    pub unsafe fn init_drivers(&self) {
+        self.for_each_descriptor(|descriptor| {
+            // 1. Initialize driver.
+            if let Err(x) = descriptor.device_driver.init() {
+                panic!(
+                    "Error initializing driver: {}: {}",
+                    descriptor.device_driver.compatible(),
+                    x
+                );
+            }
+
+            // 2. Call corresponding post init callback.
+            if let Some(callback) = &descriptor.post_init_callback {
+                if let Err(x) = callback() {
+                    panic!(
+                        "Error during driver post-init callback: {}: {}",
+                        descriptor.device_driver.compatible(),
+                        x
+                    );
+                }
+            }
+        });
+    }
+
+    /// Enumerate all registered device drivers.
+    pub fn enumerate(&self) {
+        let mut i: usize = 1;
+        self.for_each_descriptor(|descriptor| {
+            println!("      {}. {}", i, descriptor.device_driver.compatible());
+
+            i += 1;
+        });
+    }
+}

diff -uNr 04_safe_globals/src/main.rs 05_drivers_gpio_uart/src/main.rs
--- 04_safe_globals/src/main.rs
+++ 05_drivers_gpio_uart/src/main.rs
@@ -106,6 +106,7 @@
 //!     - It is implemented in `src/_arch/__arch_name__/cpu/boot.s`.
 //! 2. Once finished with architectural setup, the arch code calls `kernel_init()`.

+#![allow(clippy::upper_case_acronyms)]
 #![feature(asm_const)]
 #![feature(format_args_nl)]
 #![feature(panic_info_message)]
@@ -116,6 +117,7 @@
 mod bsp;
 mod console;
 mod cpu;
+mod driver;
 mod panic_wait;
 mod print;
 mod synchronization;
@@ -125,13 +127,42 @@
 /// # Safety
 ///
 /// - Only a single core must be active and running this function.
+/// - The init calls in this function must appear in the correct order.
 unsafe fn kernel_init() -> ! {
-    use console::console;
+    // Initialize the BSP driver subsystem.
+    if let Err(x) = bsp::driver::init() {
+        panic!("Error initializing BSP driver subsystem: {}", x);
+    }
+
+    // Initialize all device drivers.
+    driver::driver_manager().init_drivers();
+    // println! is usable from here on.

-    println!("[0] Hello from Rust!");
+    // Transition from unsafe to safe.
+    kernel_main()
+}

-    println!("[1] Chars written: {}", console().chars_written());
+/// The main function running after the early init.
+fn kernel_main() -> ! {
+    use console::console;

-    println!("[2] Stopping here.");
-    cpu::wait_forever()
+    println!(
+        "[0] {} version {}",
+        env!("CARGO_PKG_NAME"),
+        env!("CARGO_PKG_VERSION")
+    );
+    println!("[1] Booting on: {}", bsp::board_name());
+
+    println!("[2] Drivers loaded:");
+    driver::driver_manager().enumerate();
+
+    println!("[3] Chars written: {}", console().chars_written());
+    println!("[4] Echoing input now");
+
+    // Discard any spurious received characters before going into echo mode.
+    console().clear_rx();
+    loop {
+        let c = console().read_char();
+        console().write_char(c);
+    }
 }

diff -uNr 04_safe_globals/tests/boot_test_string.rb 05_drivers_gpio_uart/tests/boot_test_string.rb
--- 04_safe_globals/tests/boot_test_string.rb
+++ 05_drivers_gpio_uart/tests/boot_test_string.rb
@@ -1,3 +1,3 @@
 # frozen_string_literal: true

-EXPECTED_PRINT = 'Stopping here'
+EXPECTED_PRINT = 'Echoing input now'

```
