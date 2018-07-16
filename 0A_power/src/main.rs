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

extern crate cortex_a;
extern crate raspi3_glue;

#[macro_use]
extern crate register;

const MMIO_BASE: u32 = 0x3F00_0000;

mod delays;
mod gpio;
mod mbox;
mod power;
mod uart;

fn main() {
    let gpio = gpio::GPIO::new();
    let mut mbox = mbox::Mbox::new();
    let uart = uart::Uart::new();
    let power = power::Power::new();

    // set up serial console
    if uart.init(&mut mbox, &gpio).is_err() {
        return; // If UART fails, abort early
    }

    uart.getc(); // Press a key first before being greeted
    uart.puts("Hello Rustacean!\n\n");

    loop {
        uart.puts("\n 1 - power off\n 2 - reset\nChoose one: ");
        let c = uart.getc();
        uart.send(c);

        match c {
            '1' => {
                if power.off(&mut mbox, &gpio).is_err() {
                    uart.puts("Mailbox error in Power::off()");
                }
            }
            '2' => power.reset(),
            _ => {}
        }
    }
}
