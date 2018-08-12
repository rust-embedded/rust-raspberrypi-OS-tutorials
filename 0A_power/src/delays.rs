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
use cortex_a::{
    asm,
    regs::{cntfrq_el0::*, cntp_ctl_el0::*, cntp_tval_el0::*},
};
use register::mmio::*;

const DELAY_BASE: u32 = MMIO_BASE + 0x3004;

/*
 *
 * Using the RPi3 SoC's system timer peripheral
 *
 */
#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    SYSTMR_LO: ReadOnly<u32>, // 0x00
    SYSTMR_HI: ReadOnly<u32>, // 0x04
}

/// Public interface to the BCM System Timer
pub struct SysTmr;

impl ops::Deref for SysTmr {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

impl SysTmr {
    pub fn new() -> SysTmr {
        SysTmr
    }

    /// Returns a pointer to the register block
    fn ptr() -> *const RegisterBlock {
        DELAY_BASE as *const _
    }

    /// Get System Timer's counter
    pub fn get_system_timer(&self) -> u64 {
        // Since it is MMIO, we must emit two separate 32 bit reads
        let mut hi = self.SYSTMR_HI.get();
        let mut lo = self.SYSTMR_LO.get();

        // We have to repeat if high word changed during read. This
        // will emit a clippy warning that needs be ignored, or you
        // lose an MMIO read.
        if hi != self.SYSTMR_HI.get() {
            hi = self.SYSTMR_HI.get();
            lo = self.SYSTMR_LO.get();
        }

        // Compose long int value
        (u64::from(hi) << 32) | u64::from(lo)
    }

    /// Wait N microsec (with BCM System Timer)
    pub fn wait_msec_st(&self, n: u64) {
        let t = self.get_system_timer();

        // We must check if it's non-zero, because qemu does not
        // emulate system timer, and returning constant zero would
        // mean infinite loop
        if t > 0 {
            loop {
                if self.get_system_timer() < (t + n) {
                    break;
                }
            }
        }
    }
}

/*
 *
 * Using the CPU's counter registers
 *
 */
/// Wait N microsec (ARM CPU only)
pub fn wait_msec(n: u32) {
    // Get the counter frequency
    let frq = CNTFRQ_EL0.get();

    // Calculate number of ticks
    let tval = (frq as u32 / 1000) * n;

    // Set the compare value register
    CNTP_TVAL_EL0.set(tval);

    // Kick off the counting                        // Disable timer interrupt
    CNTP_CTL_EL0.modify(CNTP_CTL_EL0::ENABLE::SET + CNTP_CTL_EL0::IMASK::SET);

    loop {
        // ISTATUS will be one when cval ticks have passed. Continuously check it.
        if CNTP_CTL_EL0.is_set(CNTP_CTL_EL0::ISTATUS) {
            break;
        }
    }

    // Disable counting again
    CNTP_CTL_EL0.modify(CNTP_CTL_EL0::ENABLE::CLEAR);
}

/*
 *
 * Using the CPU's cycles
 *
 */
/// Wait N CPU cycles (ARM CPU only)
pub fn wait_cycles(cyc: u32) {
    for _ in 0..cyc {
        asm::nop();
    }
}
