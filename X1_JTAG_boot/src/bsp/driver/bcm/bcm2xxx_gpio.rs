// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! GPIO driver.

use crate::{arch, arch::sync::NullLock, interface};
use core::ops;
use register::{mmio::ReadWrite, register_bitfields, register_structs};

// GPIO registers.
//
// Descriptions taken from
// https://github.com/raspberrypi/documentation/files/1888662/BCM2837-ARM-Peripherals.-.Revised.-.V2-1.pdf
register_bitfields! {
    u32,

    /// GPIO Function Select 1
    GPFSEL1 [
        /// Pin 15
        FSEL15 OFFSET(15) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AltFunc0 = 0b100  // PL011 UART RX

        ],

        /// Pin 14
        FSEL14 OFFSET(12) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AltFunc0 = 0b100  // PL011 UART TX
        ]
    ],

    /// GPIO Function Select 2
    GPFSEL2 [
        /// Pin 27
        FSEL27 OFFSET(21) NUMBITS(3)[
            Input = 0b000,
            Output = 0b001,
            AltFunc4 = 0b011 // JTAG TMS
        ],

        /// Pin 26
        FSEL26 OFFSET(18) NUMBITS(3)[
            Input = 0b000,
            Output = 0b001,
            AltFunc4 = 0b011 // JTAG TDI
        ],

        /// Pin 25
        FSEL25 OFFSET(15) NUMBITS(3)[
            Input = 0b000,
            Output = 0b001,
            AltFunc4 = 0b011 // JTAG TCK
        ],

        /// Pin 24
        FSEL24 OFFSET(12) NUMBITS(3)[
            Input = 0b000,
            Output = 0b001,
            AltFunc4 = 0b011 // JTAG TDO
        ],

        /// Pin 23
        FSEL23 OFFSET(9) NUMBITS(3)[
            Input = 0b000,
            Output = 0b001,
            AltFunc4 = 0b011 // JTAG RTCK
        ],

        /// Pin 22
        FSEL22 OFFSET(6) NUMBITS(3)[
            Input = 0b000,
            Output = 0b001,
            AltFunc4 = 0b011 // JTAG TRST
        ]
   ],

    /// GPIO Pull-up/down Clock Register 0
    GPPUDCLK0 [
        /// Pin 27
        PUDCLK27 OFFSET(27) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],
        /// Pin 26
        PUDCLK26 OFFSET(26) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 25
        PUDCLK25 OFFSET(25) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 24
        PUDCLK24 OFFSET(24) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 23
        PUDCLK23 OFFSET(23) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 22
        PUDCLK22 OFFSET(22) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 15
        PUDCLK15 OFFSET(15) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 14
        PUDCLK14 OFFSET(14) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ]
    ]
}

register_structs! {
    #[allow(non_snake_case)]
    RegisterBlock {
        (0x00 => GPFSEL0: ReadWrite<u32>),
        (0x04 => GPFSEL1: ReadWrite<u32, GPFSEL1::Register>),
        (0x08 => GPFSEL2: ReadWrite<u32, GPFSEL2::Register>),
        (0x0C => GPFSEL3: ReadWrite<u32>),
        (0x10 => GPFSEL4: ReadWrite<u32>),
        (0x14 => GPFSEL5: ReadWrite<u32>),
        (0x18 => _reserved1),
        (0x94 => GPPUD: ReadWrite<u32>),
        (0x98 => GPPUDCLK0: ReadWrite<u32, GPPUDCLK0::Register>),
        (0x9C => GPPUDCLK1: ReadWrite<u32>),
        (0xA0 => @END),
    }
}

/// The driver's private data.
struct GPIOInner {
    base_addr: usize,
}

/// Deref to RegisterBlock.
impl ops::Deref for GPIOInner {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}

impl GPIOInner {
    const fn new(base_addr: usize) -> GPIOInner {
        GPIOInner { base_addr }
    }

    /// Return a pointer to the register block.
    fn ptr(&self) -> *const RegisterBlock {
        self.base_addr as *const _
    }
}

//--------------------------------------------------------------------------------------------------
// BSP-public
//--------------------------------------------------------------------------------------------------
use interface::sync::Mutex;

/// The driver's main struct.
pub struct GPIO {
    inner: NullLock<GPIOInner>,
}

impl GPIO {
    pub const unsafe fn new(base_addr: usize) -> GPIO {
        GPIO {
            inner: NullLock::new(GPIOInner::new(base_addr)),
        }
    }

    /// Map PL011 UART as standard output.
    ///
    /// TX to pin 14
    /// RX to pin 15
    pub fn map_pl011_uart(&self) {
        let mut r = &self.inner;
        r.lock(|inner| {
            // Map to pins.
            inner
                .GPFSEL1
                .modify(GPFSEL1::FSEL14::AltFunc0 + GPFSEL1::FSEL15::AltFunc0);

            // Enable pins 14 and 15.
            inner.GPPUD.set(0);
            arch::spin_for_cycles(150);

            inner
                .GPPUDCLK0
                .write(GPPUDCLK0::PUDCLK14::AssertClock + GPPUDCLK0::PUDCLK15::AssertClock);
            arch::spin_for_cycles(150);

            inner.GPPUDCLK0.set(0);
        })
    }
}

//--------------------------------------------------------------------------------------------------
// OS interface implementations
//--------------------------------------------------------------------------------------------------

impl interface::driver::DeviceDriver for GPIO {
    fn compatible(&self) -> &str {
        "GPIO"
    }
}

impl interface::driver::JTAGOps for GPIO {
    fn enable(&self) -> interface::driver::Result {
        let mut r = &self.inner;
        r.lock(|inner| {
            inner.GPFSEL2.modify(
                GPFSEL2::FSEL22::AltFunc4
                    + GPFSEL2::FSEL23::AltFunc4
                    + GPFSEL2::FSEL24::AltFunc4
                    + GPFSEL2::FSEL25::AltFunc4
                    + GPFSEL2::FSEL26::AltFunc4
                    + GPFSEL2::FSEL27::AltFunc4,
            );

            inner.GPPUD.set(0);
            arch::spin_for_cycles(150);

            inner.GPPUDCLK0.write(
                GPPUDCLK0::PUDCLK22::AssertClock
                    + GPPUDCLK0::PUDCLK23::AssertClock
                    + GPPUDCLK0::PUDCLK24::AssertClock
                    + GPPUDCLK0::PUDCLK25::AssertClock
                    + GPPUDCLK0::PUDCLK26::AssertClock
                    + GPPUDCLK0::PUDCLK27::AssertClock,
            );
            arch::spin_for_cycles(150);

            inner.GPPUDCLK0.set(0);

            Ok(())
        })
    }
}
