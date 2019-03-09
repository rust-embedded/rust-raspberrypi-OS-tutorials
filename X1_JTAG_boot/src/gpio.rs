/*
 * MIT License
 *
 * Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>
 * Copyright (c) 2019 Nao Taco <naotaco@gmail.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

use core::ops;
use register::{mmio::ReadWrite, register_bitfields};

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
            RXD0 = 0b100, // UART0     - Alternate function 0
            RXD1 = 0b010  // Mini UART - Alternate function 5

        ],

        /// Pin 14
        FSEL14 OFFSET(12) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            TXD0 = 0b100, // UART0     - Alternate function 0
            TXD1 = 0b010  // Mini UART - Alternate function 5
        ]
    ],

    /// GPIO Function Select 2
    GPFSEL2 [
        /// Pin 27
        FSEL27 OFFSET(21) NUMBITS(3)[
            Input = 0b000,
            Output = 0b001,
            ARM_TMS = 0b011 // JTAG TMS - Alternate function 4
        ],

        /// Pin 26
        FSEL26 OFFSET(18) NUMBITS(3)[
            Input = 0b000,
            Output = 0b001,
            ARM_TDI = 0b011 // JTAG TDI - Alternate function 4
        ],

        /// Pin 25
        FSEL25 OFFSET(15) NUMBITS(3)[
            Input = 0b000,
            Output = 0b001,
            ARM_TCK = 0b011 // JTAG TCK - Alternate function 4
        ],

        /// Pin 24
        FSEL24 OFFSET(12) NUMBITS(3)[ // GPIO24
            Input = 0b000,
            Output = 0b001,
            ARM_TDO = 0b011 // JTAG TDO - Alternate function 4
        ],

        /// Pin 23
        FSEL23 OFFSET(9) NUMBITS(3)[
            Input = 0b000,
            Output = 0b001,
            ARM_RTCK = 0b011 // JTAG RTCK - Alternate function 4
        ],

        /// Pin 22
        FSEL22 OFFSET(6) NUMBITS(3)[
            Input = 0b000,
            Output = 0b001,
            ARM_TRST = 0b011 // JTAG TRST - Alternate function 4
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
    pub GPFSEL2: ReadWrite<u32, GPFSEL2::Register>,     // 0x08
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

/// Public interface to the GPIO MMIO area
pub struct GPIO {
    base_addr: usize,
}

impl ops::Deref for GPIO {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}

impl GPIO {
    pub fn new(base_addr: usize) -> GPIO {
        GPIO { base_addr }
    }

    /// Returns a pointer to the register block
    fn ptr(&self) -> *const RegisterBlock {
        self.base_addr as *const _
    }
}
