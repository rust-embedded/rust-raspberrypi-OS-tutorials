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
use volatile_register::*;
use gpio;

const MINI_UART_BASE: u32 = MMIO_BASE + 0x215000;

/// Auxilary mini UART registers
#[allow(non_snake_case)]
#[repr(C, packed)]
struct Registers {
    _reserved0: u32,        // 0x00
    ENABLES: RW<u32>,       // 0x04
    _reserved1: [u8; 0x38], // 0x08
    MU_IO: RW<u32>,         // 0x40
    MU_IER: RW<u32>,        // 0x44
    MU_IIR: RW<u32>,        // 0x48
    MU_LCR: RW<u32>,        // 0x4C
    MU_MCR: RW<u32>,        // 0x50
    MU_LSR: RW<u32>,        // 0x54
    MU_MSR: RW<u32>,        // 0x58
    MU_SCRATCH: RW<u32>,    // 0x5C
    MU_CNTL: RW<u32>,       // 0x60
    MU_STAT: RW<u32>,       // 0x64
    MU_BAUD: RW<u32>,       // 0x68
}

pub struct MiniUart {
    registers: *const Registers,
}

impl MiniUart {
    pub fn new() -> MiniUart {
        MiniUart {
            registers: MINI_UART_BASE as *const Registers,
        }
    }

    ///Set baud rate and characteristics (115200 8N1) and map to GPIO
    pub fn init(&self) {
        // initialize UART
        unsafe {
            (*self.registers).ENABLES.modify(|x| x | 1); // enable UART1, AUX mini uart
            (*self.registers).MU_IER.write(0);
            (*self.registers).MU_CNTL.write(0);
            (*self.registers).MU_LCR.write(3); // 8 bits
            (*self.registers).MU_MCR.write(0);
            (*self.registers).MU_IER.write(0);
            (*self.registers).MU_IIR.write(0xC6); // disable interrupts
            (*self.registers).MU_BAUD.write(270); // 115200 baud

            // map UART1 to GPIO pins
            (*gpio::GPFSEL1).modify(|x| {
                // Modify with a closure
                let mut ret = x;
                ret &= !((7 << 12) | (7 << 15)); // gpio14, gpio15
                ret |= (2 << 12) | (2 << 15); // alt5

                ret
            });

            (*gpio::GPPUD).write(0); // enable pins 14 and 15
            for _ in 0..150 {
                asm!("nop" :::: "volatile");
            }

            (*gpio::GPPUDCLK0).write((1 << 14) | (1 << 15));
            for _ in 0..150 {
                asm!("nop" :::: "volatile");
            }
            (*gpio::GPPUDCLK0).write(0); // flush GPIO setup
            (*self.registers).MU_CNTL.write(3); // enable Tx, Rx
        }
    }

    /// Send a character
    pub fn send(&self, c: char) {
        unsafe {
            // wait until we can send
            loop {
                if ((*self.registers).MU_LSR.read() & 0x20) == 0x20 {
                    break;
                }
                asm!("nop" :::: "volatile");
            }

            // write the character to the buffer
            (*self.registers).MU_IO.write(c as u32);
        }
    }

    /// Receive a character
    pub fn getc(&self) -> char {
        unsafe {
            // wait until something is in the buffer
            loop {
                if ((*self.registers).MU_LSR.read() & 0x01) == 0x01 {
                    break;
                }
                asm!("nop" :::: "volatile");
            }
        }

        // read it and return
        let mut ret = unsafe { (*self.registers).MU_IO.read() as u8 as char };

        // convert carrige return to newline
        if ret == '\r' {
            ret = '\n'
        }

        ret
    }

    /// Display a string
    pub fn puts(&self, string: &str) {
        for c in string.chars() {
            // convert newline to carrige return + newline
            if c == '\n' {
                self.send('\r')
            }

            self.send(c);
        }
    }

    /// Display a binary value in hexadecimal
    pub fn hex(&self, d: u32) {
        let mut n;

        for i in 0..8 {
            // get highest tetrad
            n = d.wrapping_shr(28 - i * 4) & 0xF;

            // 0-9 => '0'-'9', 10-15 => 'A'-'F'
            // Add proper offset for ASCII table
            if n > 9 {
                n += 0x37;
            } else {
                n += 0x30;
            }

            self.send(n as u8 as char);
        }
    }
}
