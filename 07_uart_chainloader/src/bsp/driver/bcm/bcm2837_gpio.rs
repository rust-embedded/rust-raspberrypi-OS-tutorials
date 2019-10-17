// SPDX-License-Identifier: MIT
//
// Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>

//! GPIO driver.

use crate::{arch, arch::sync::NullLock, interface};
use core::ops;
use register::{mmio::ReadWrite, register_bitfields};

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
            AltFunc5 = 0b010  // Mini UART RX

        ],

        /// Pin 14
        FSEL14 OFFSET(12) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AltFunc5 = 0b010  // Mini UART TX
        ]
    ],

    /// GPIO Pull-up/down Clock Register 0
    GPPUDCLK0 [
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

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    pub GPFSEL0: ReadWrite<u32>,                        // 0x00
    pub GPFSEL1: ReadWrite<u32, GPFSEL1::Register>,     // 0x04
    pub GPFSEL2: ReadWrite<u32>,                        // 0x08
    pub GPFSEL3: ReadWrite<u32>,                        // 0x0C
    pub GPFSEL4: ReadWrite<u32>,                        // 0x10
    pub GPFSEL5: ReadWrite<u32>,                        // 0x14
    __reserved_0: u32,                                  // 0x18
    GPSET0: ReadWrite<u32>,                             // 0x1C
    GPSET1: ReadWrite<u32>,                             // 0x20
    __reserved_1: u32,                                  //
    GPCLR0: ReadWrite<u32>,                             // 0x28
    __reserved_2: [u32; 2],                             //
    GPLEV0: ReadWrite<u32>,                             // 0x34
    GPLEV1: ReadWrite<u32>,                             // 0x38
    __reserved_3: u32,                                  //
    GPEDS0: ReadWrite<u32>,                             // 0x40
    GPEDS1: ReadWrite<u32>,                             // 0x44
    __reserved_4: [u32; 7],                             //
    GPHEN0: ReadWrite<u32>,                             // 0x64
    GPHEN1: ReadWrite<u32>,                             // 0x68
    __reserved_5: [u32; 10],                            //
    pub GPPUD: ReadWrite<u32>,                          // 0x94
    pub GPPUDCLK0: ReadWrite<u32, GPPUDCLK0::Register>, // 0x98
    pub GPPUDCLK1: ReadWrite<u32>,                      // 0x9C
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

    /// Map Mini UART as standard output.
    ///
    /// TX to pin 14
    /// RX to pin 15
    pub fn map_mini_uart(&mut self) {
        // Map to pins.
        self.GPFSEL1
            .modify(GPFSEL1::FSEL14::AltFunc5 + GPFSEL1::FSEL15::AltFunc5);

        // Enable pins 14 and 15.
        self.GPPUD.set(0);
        for _ in 0..150 {
            arch::nop();
        }

        self.GPPUDCLK0
            .write(GPPUDCLK0::PUDCLK14::AssertClock + GPPUDCLK0::PUDCLK15::AssertClock);
        for _ in 0..150 {
            arch::nop();
        }

        self.GPPUDCLK0.set(0);
    }
}

////////////////////////////////////////////////////////////////////////////////
// BSP-public
////////////////////////////////////////////////////////////////////////////////
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

    // Only visible to other BSP code.
    pub fn map_mini_uart(&self) {
        let mut r = &self.inner;
        r.lock(|inner| inner.map_mini_uart());
    }
}

////////////////////////////////////////////////////////////////////////////////
// OS interface implementations
////////////////////////////////////////////////////////////////////////////////

impl interface::driver::DeviceDriver for GPIO {
    fn compatible(&self) -> &str {
        "GPIO"
    }
}
