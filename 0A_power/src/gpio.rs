/*
 * MIT License
 *
 * Copyright (c) 2018 Andre Richter <andre.o.richter@gmail.com>
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

use super::MMIO_BASE;
use core::ops;
use volatile_register::RW;

const GPIO_BASE: u32 = MMIO_BASE + 0x200_000;

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    pub GPFSEL0: RW<u32>,    // 0x00
    pub GPFSEL1: RW<u32>,    // 0x04
    pub GPFSEL2: RW<u32>,    // 0x08
    pub GPFSEL3: RW<u32>,    // 0x0C
    pub GPFSEL4: RW<u32>,    // 0x10
    pub GPFSEL5: RW<u32>,    // 0x14
    __reserved_0: u32,       // 0x18
    GPSET0: RW<u32>,         // 0x1C
    GPSET1: RW<u32>,         // 0x20
    __reserved_1: u32,       //
    GPCLR0: RW<u32>,         // 0x28
    __reserved_2: [u32; 2],  //
    GPLEV0: RW<u32>,         // 0x34
    GPLEV1: RW<u32>,         // 0x38
    __reserved_3: u32,       //
    GPEDS0: RW<u32>,         // 0x40
    GPEDS1: RW<u32>,         // 0x44
    __reserved_4: [u32; 7],  //
    GPHEN0: RW<u32>,         // 0x64
    GPHEN1: RW<u32>,         // 0x68
    __reserved_5: [u32; 10], //
    pub GPPUD: RW<u32>,      // 0x94
    pub GPPUDCLK0: RW<u32>,  // 0x98
    pub GPPUDCLK1: RW<u32>,  // 0x9C
}

/// Public interface to the GPIO MMIO area
pub struct GPIO;

impl ops::Deref for GPIO {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

impl GPIO {
    pub fn new() -> GPIO {
        GPIO
    }

    /// Returns a pointer to the register block
    fn ptr() -> *const RegisterBlock {
        GPIO_BASE as *const _
    }
}
