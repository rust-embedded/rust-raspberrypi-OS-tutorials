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
use register::{mmio::*, register_bitfields};

register_bitfields! {
    u32,

    CTRL [
        ENABLE OFFSET(0) NUMBITS(1) [
            True = 1,
            False = 0
        ]
    ],

    INT_MASK [
        INT_OFF OFFSET(0) NUMBITS(1) [
            True = 1,
            False = 0
        ]
    ]
}

const RNG_BASE: u32 = MMIO_BASE + 0x104_000;
const RNG_WARMUP_COUNT: u32 = 0x40_000;

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    CTRL: ReadWrite<u32, CTRL::Register>,         // 0x00
    STATUS: ReadWrite<u32>,                       // 0x04
    DATA: ReadOnly<u32>,                          // 0x08
    __reserved_0: u32,                            // 0x0c
    INT_MASK: ReadWrite<u32, INT_MASK::Register>, // 0x10
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
        // Disable interrupts
        self.INT_MASK.modify(INT_MASK::INT_OFF::True);

        // Set warm-up count and enable
        self.STATUS.set(RNG_WARMUP_COUNT);
        self.CTRL.modify(CTRL::ENABLE::True);
    }

    /// Return a random number between [min..max]
    pub fn rand(&self, min: u32, max: u32) -> u32 {
        // wait for gaining some entropy
        loop {
            if (self.STATUS.get() >> 24) != 0 {
                break;
            }

            asm::nop();
        }

        let r = self.DATA.get();

        r % (max - min) + min
    }
}
