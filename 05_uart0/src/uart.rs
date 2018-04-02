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
use mbox;
use gpio;
use core::sync::atomic::{compiler_fence, Ordering};

const UART_BASE: u32 = MMIO_BASE + 0x201000;

// PL011 UART registers
#[allow(non_snake_case)]
#[repr(C, packed)]
struct Registers {
    DR: RW<u32>,           // 0x00
    reserved0: [u8; 0x14], // 0x04
    FR: RO<u32>,           // 0x18
    reserved1: u64,        // 0x1C
    IBRD: WO<u32>,         // 0x24
    FBRD: WO<u32>,         // 0x28
    LCRH: WO<u32>,         // 0x2C
    CR: WO<u32>,           // 0x30
    reserved3: [u8; 0x10], // 0x34
    ICR: WO<u32>,          // 0x44
}

pub enum UartError {
    MailboxError,
}
pub type Result<T> = ::core::result::Result<T, UartError>;

pub struct Uart {
    registers: *const Registers,
}

impl Uart {
    pub fn new() -> Uart {
        Uart {
            registers: UART_BASE as *const Registers,
        }
    }

    ///Set baud rate and characteristics (115200 8N1) and map to GPIO
    pub fn init(&self, mbox: &mut mbox::Mbox) -> Result<()> {
        // turn off UART0
        unsafe { (*self.registers).CR.write(0) };

        // set up clock for consistent divisor values
        mbox.buffer[0] = 9 * 4;
        mbox.buffer[1] = mbox::REQUEST;
        mbox.buffer[2] = mbox::tag::SETCLKRATE;
        mbox.buffer[3] = 12;
        mbox.buffer[4] = 8;
        mbox.buffer[5] = mbox::clock::UART; // UART clock
        mbox.buffer[6] = 4000000; // 4Mhz
        mbox.buffer[7] = 0; // skip turbo setting
        mbox.buffer[8] = mbox::tag::LAST;

        // Insert a compiler fence that ensures that all stores to the
        // mbox buffer are finished before the GPU is signaled (which
        // is done by a store operation as well).
        compiler_fence(Ordering::SeqCst);

        if let Err(_) = mbox.call(mbox::channel::PROP) {
            return Err(UartError::MailboxError); // Abort if UART clocks couldn't be set
        };

        // map UART0 to GPIO pins
        unsafe {
            (*gpio::GPFSEL1).modify(|x| {
                // Modify with a closure
                let mut ret = x;
                ret &= !((7 << 12) | (7 << 15)); // gpio14, gpio15
                ret |= (4 << 12) | (4 << 15); // alt0

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
            (*gpio::GPPUDCLK0).write(0);

            (*self.registers).ICR.write(0x7FF); // clear interrupts
            (*self.registers).IBRD.write(2); // 115200 baud
            (*self.registers).FBRD.write(0xB);
            (*self.registers).LCRH.write(0b11 << 5); // 8n1
            (*self.registers).CR.write(0x301); // enable Tx, Rx, FIFO
        }

        Ok(())
    }

    /// Send a character
    pub fn send(&self, c: char) {
        unsafe {
            // wait until we can send
            loop {
                if !(((*self.registers).FR.read() & 0x20) == 0x20) {
                    break;
                }
                asm!("nop" :::: "volatile");
            }

            // write the character to the buffer
            (*self.registers).DR.write(c as u32);
        }
    }

    /// Receive a character
    pub fn getc(&self) -> char {
        unsafe {
            // wait until something is in the buffer
            loop {
                if !(((*self.registers).FR.read() & 0x10) == 0x10) {
                    break;
                }
                asm!("nop" :::: "volatile");
            }
        }

        // read it and return
        let mut ret = unsafe { (*self.registers).DR.read() as u8 as char };

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
