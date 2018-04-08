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

#![no_std]
#![feature(asm)]

extern crate raspi3_glue;
extern crate volatile_register;

const MMIO_BASE: u32 = 0x3F00_0000;

mod gpio;
mod mbox;
mod uart;

fn main() {
    let mut mbox = mbox::Mbox::new();
    let uart = uart::Uart::new();

    // set up serial console
    if uart.init(&mut mbox).is_err() {
        return; // If UART fails, abort early
    }

    let kernel: *mut u8 = 0x80_000 as *mut u8;

    // Say hello
    for c in "RBIN64\r\n".chars() {
        uart.send(c);
    }

    // Notify raspbootcom to send the kernel
    uart.send(3 as char);
    uart.send(3 as char);
    uart.send(3 as char);

    // Read the kernel's size
    let mut size: u32 = uart.getc() as u32;
    size |= (uart.getc() as u32) << 8;
    size |= (uart.getc() as u32) << 16;
    size |= (uart.getc() as u32) << 24;

    // For now, blindly trust it's not too big
    uart.send('O');
    uart.send('K');

    unsafe {
        // Read the kernel byte by byte
        for i in 0..size {
            *kernel.offset(i as isize) = uart.getc();
        }

        // Restore arguments and jump to loaded kernel
        asm!("mov x0, x25      \n\t\
              mov x1, x26      \n\t\
              mov x2, x27      \n\t\
              mov x3, x28      \n\t\
              mov x30, 0x80000 \n\t\
              ret"
              :::: "volatile");
    }
}
