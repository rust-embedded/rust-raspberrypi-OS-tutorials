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
use core::sync::atomic::{compiler_fence, Ordering};
use delays;
use gpio;
use mbox;
use volatile_register::*;

const POWER_BASE: u32 = MMIO_BASE + 0x100_01C;

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    PM_RSTC: RW<u32>, // 0x1C
    PM_RSTS: RW<u32>, // 0x20
    PM_WDOG: RW<u32>, // 0x24
}

const PM_WDOG_MAGIC: u32 = 0x5a_000_000;
const PM_RSTC_FULLRST: u32 = 0x20;

pub enum PowerError {
    MailboxError,
}
pub type Result<T> = ::core::result::Result<T, PowerError>;

/// Public interface to the Power subsystem
pub struct Power;

impl ops::Deref for Power {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

impl Power {
    pub fn new() -> Power {
        Power
    }

    /// Returns a pointer to the register block
    fn ptr() -> *const RegisterBlock {
        POWER_BASE as *const _
    }

    /// Shutdown the board
    pub fn off(&self, mbox: &mut mbox::Mbox, gpio: &gpio::GPIO) -> Result<()> {
        // power off devices one by one
        for dev_id in 0..16 {
            mbox.buffer[0] = 8 * 4;
            mbox.buffer[1] = mbox::REQUEST;
            mbox.buffer[2] = mbox::tag::SETPOWER;
            mbox.buffer[3] = 8;
            mbox.buffer[4] = 8;
            mbox.buffer[5] = dev_id; // device id
            mbox.buffer[6] = 0; // bit 0: off, bit 1: no wait
            mbox.buffer[7] = mbox::tag::LAST;

            // Insert a compiler fence that ensures that all stores to the
            // mbox buffer are finished before the GPU is signaled (which
            // is done by a store operation as well).
            compiler_fence(Ordering::Release);

            if mbox.call(mbox::channel::PROP).is_err() {
                return Err(PowerError::MailboxError); // Abort if UART clocks couldn't be set
            };
        }

        // power off gpio pins (but not VCC pins)
        unsafe {
            gpio.GPFSEL0.write(0);
            gpio.GPFSEL1.write(0);
            gpio.GPFSEL2.write(0);
            gpio.GPFSEL3.write(0);
            gpio.GPFSEL4.write(0);
            gpio.GPFSEL5.write(0);

            gpio.GPPUD.write(0);
            delays::wait_cycles(150);

            gpio.GPPUDCLK0.write(0xffff_ffff);
            gpio.GPPUDCLK1.write(0xffff_ffff);
            delays::wait_cycles(150);

            // flush GPIO setup
            gpio.GPPUDCLK0.write(0);
            gpio.GPPUDCLK1.write(0);

            // power off the SoC (GPU + CPU)
            self.PM_RSTS.modify(|reg| {
                let mut r = reg;

                r &= !0xffff_faaa;
                r |= 0x555; // partition 63 used to indicate halt
                PM_WDOG_MAGIC | r
            });
            self.PM_WDOG.write(PM_WDOG_MAGIC | 10);
            self.PM_RSTC.write(PM_WDOG_MAGIC | PM_RSTC_FULLRST);
        }

        Ok(())
    }

    /// Reboot
    pub fn reset(&self) {
        // trigger a restart by instructing the GPU to boot from partition 0
        unsafe {
            self.PM_RSTS.modify(|reg| {
                let mut r = reg;

                r &= !0xffff_faaa;
                PM_WDOG_MAGIC | r
            });
            self.PM_WDOG.write(PM_WDOG_MAGIC | 10);
            self.PM_RSTC.write(PM_WDOG_MAGIC | PM_RSTC_FULLRST);
        }
    }
}
