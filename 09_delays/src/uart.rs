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
use cortex_a::asm;
use gpio;
use mbox;
use volatile_register::*;

const UART_BASE: u32 = MMIO_BASE + 0x20_1000;

// PL011 UART registers
#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    DR: RW<u32>,            // 0x00
    __reserved_0: [u32; 5], // 0x04
    FR: RO<u32>,            // 0x18
    __reserved_1: [u32; 2], // 0x1c
    IBRD: WO<u32>,          // 0x24
    FBRD: WO<u32>,          // 0x28
    LCRH: WO<u32>,          // 0x2C
    CR: WO<u32>,            // 0x30
    __reserved_2: [u32; 4], // 0x34
    ICR: WO<u32>,           // 0x44
}

pub enum UartError {
    MailboxError,
}
pub type Result<T> = ::core::result::Result<T, UartError>;

pub struct Uart;

impl ops::Deref for Uart {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

impl Uart {
    pub fn new() -> Uart {
        Uart
    }

    /// Returns a pointer to the register block
    fn ptr() -> *const RegisterBlock {
        UART_BASE as *const _
    }

    ///Set baud rate and characteristics (115200 8N1) and map to GPIO
    pub fn init(&self, mbox: &mut mbox::Mbox) -> Result<()> {
        // turn off UART0
        unsafe { self.CR.write(0) };

        // set up clock for consistent divisor values
        mbox.buffer[0] = 9 * 4;
        mbox.buffer[1] = mbox::REQUEST;
        mbox.buffer[2] = mbox::tag::SETCLKRATE;
        mbox.buffer[3] = 12;
        mbox.buffer[4] = 8;
        mbox.buffer[5] = mbox::clock::UART; // UART clock
        mbox.buffer[6] = 4_000_000; // 4Mhz
        mbox.buffer[7] = 0; // skip turbo setting
        mbox.buffer[8] = mbox::tag::LAST;

        // Insert a compiler fence that ensures that all stores to the
        // mbox buffer are finished before the GPU is signaled (which
        // is done by a store operation as well).
        compiler_fence(Ordering::Release);

        if mbox.call(mbox::channel::PROP).is_err() {
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
                asm::nop();
            }

            (*gpio::GPPUDCLK0).write((1 << 14) | (1 << 15));
            for _ in 0..150 {
                asm::nop();
            }
            (*gpio::GPPUDCLK0).write(0);

            self.ICR.write(0x7FF); // clear interrupts
            self.IBRD.write(2); // 115200 baud
            self.FBRD.write(0xB);
            self.LCRH.write(0b11 << 5); // 8n1
            self.CR.write(0x301); // enable Tx, Rx, FIFO
        }

        Ok(())
    }

    /// Send a character
    pub fn send(&self, c: char) {
        // wait until we can send
        loop {
            if (self.FR.read() & 0x20) != 0x20 {
                break;
            }

            asm::nop();
        }

        // write the character to the buffer
        unsafe { self.DR.write(c as u32) };
    }

    /// Receive a character
    pub fn getc(&self) -> char {
        // wait until something is in the buffer
        loop {
            if (self.FR.read() & 0x10) != 0x10 {
                break;
            }

            asm::nop();
        }

        // read it and return
        let mut ret = self.DR.read() as u8 as char;

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
}
