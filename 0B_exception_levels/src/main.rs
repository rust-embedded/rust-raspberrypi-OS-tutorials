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

const MMIO_BASE: u32 = 0x3F00_0000;

mod delays;
mod gpio;
mod mbox;
mod uart;

use cortex_a::regs::*;

fn check_timer(uart: &uart::Uart) {
    uart.puts(
        "Testing EL1 access to timer registers.\n\
         Delaying for 3 seconds now.\n",
    );
    delays::wait_msec(1000);
    uart.puts("1..");
    delays::wait_msec(1000);
    uart.puts("2..");
    delays::wait_msec(1000);
    uart.puts(
        "3\n\
         Works!\n\n",
    );
}

fn check_daif(uart: &uart::Uart) {
    uart.puts("Checking interrupt mask bits:\n");

    let daif = DAIF.extract();
    for x in &[
        ("D: ", DAIF::D),
        ("A: ", DAIF::A),
        ("I: ", DAIF::I),
        ("F: ", DAIF::F),
    ] {
        uart.puts(x.0);
        if daif.is_set(x.1) {
            uart.puts("Masked.\n");
        } else {
            uart.puts("Unmasked.\n");
        }
    }
}

raspi3_boot::entry!(kernel_entry);

fn kernel_entry() -> ! {
    let mut mbox = mbox::Mbox::new();
    let uart = uart::Uart::new();

    // set up serial console
    if uart.init(&mut mbox).is_err() {
        loop {
            cortex_a::asm::wfe() // If UART fails, abort early
        }
    }

    uart.getc(); // Press a key first before being greeted
    uart.puts("Hello Rustacean!\n\n");

    uart.puts("Executing in EL: ");
    uart.hex(CurrentEL.read(CurrentEL::EL));
    uart.puts("\n\n");

    check_timer(&uart);
    check_daif(&uart);

    // echo everything back
    loop {
        uart.send(uart.getc());
    }
}
