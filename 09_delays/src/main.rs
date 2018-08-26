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
#![no_main]

extern crate cortex_a;

#[macro_use]
extern crate raspi3_boot;

#[macro_use]
extern crate register;

const MMIO_BASE: u32 = 0x3F00_0000;

mod delays;
mod gpio;
mod mbox;
mod uart;

entry!(kernel_entry);

fn kernel_entry() -> ! {
    let mut mbox = mbox::Mbox::new();
    let uart = uart::Uart::new();

    // set up serial console
    if uart.init(&mut mbox).is_err() {
        loop {
            cortex_a::asm::wfe()
        } // If UART fails, abort early
    }

    uart.getc(); // Press a key first before being greeted
    uart.puts("Hello Rustacean!\n");

    uart.puts("Waiting 1_000_000 CPU cycles (ARM CPU): ");
    delays::wait_cycles(1_000_000);
    uart.puts("OK\n");

    uart.puts("Waiting 1000 microsec (ARM CPU): ");
    delays::wait_msec(1000);
    uart.puts("OK\n");

    let t = delays::SysTmr::new();
    if t.get_system_timer() != 0 {
        uart.puts("Waiting 1000 microsec (BCM System Timer): ");
        t.wait_msec_st(1000);
        uart.puts("OK\n");
    }

    uart.puts("Looping forever now!\n");
    loop {
        delays::wait_msec(1000);
        uart.puts("Tick: 1s\n");
    }
}
