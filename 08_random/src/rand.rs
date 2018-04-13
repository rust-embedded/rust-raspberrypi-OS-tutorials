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
use cortex_a::asm;
use volatile_register::*;

const RNG_BASE: u32 = MMIO_BASE + 0x104_000;

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    CTRL: RW<u32>,     // 0x00
    STATUS: RW<u32>,   // 0x04
    DATA: RO<u32>,     // 0x08
    __reserved_0: u32, // 0x0c
    INT_MASK: RW<u32>, // 0x10
}

/// Public interface to the RNG
pub struct Rng;

impl ops::Deref for Rng {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

impl Rng {
    pub fn new() -> Rng {
        Rng
    }

    /// Returns a pointer to the register block
    fn ptr() -> *const RegisterBlock {
        RNG_BASE as *const _
    }

    /// Initialize the RNG
    pub fn init(&self) {
        unsafe {
            self.STATUS.write(0x40_000);

            // mask interrupt
            self.INT_MASK.modify(|x| x | 1);

            // enable
            self.CTRL.modify(|x| x | 1);
        }

        // wait for gaining some entropy
        loop {
            if (self.STATUS.read() >> 24) != 0 {
                break;
            }

            asm::nop();
        }
    }

    /// Return a random number between [min..max]
    pub fn rand(&self, min: u32, max: u32) -> u32 {
        let r = self.DATA.read();

        r % (max - min) + min
    }
}
