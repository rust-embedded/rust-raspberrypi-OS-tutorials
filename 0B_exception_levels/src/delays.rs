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

use cortex_a::regs::*;

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
