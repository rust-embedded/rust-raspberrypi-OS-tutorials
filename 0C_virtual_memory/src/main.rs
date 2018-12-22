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
#![feature(asm)]
#![feature(const_fn)]

const MMIO_BASE: u32 = 0x3F00_0000;

mod gpio;
mod mbox;
mod mmu;
mod uart;

raspi3_boot::entry!(kernel_entry);

fn kernel_entry() -> ! {
    let mut mbox = mbox::Mbox::new();

    {
        // Before the MMU is live, instantiate a UART driver with the physical address
        let uart = uart::Uart::new(uart::UART_PHYS_BASE);

        // set up serial console
        if uart.init(&mut mbox).is_err() {
            loop {
                cortex_a::asm::wfe() // If UART fails, abort early
            }
        }

        uart.getc(); // Press a key first before being greeted
        uart.puts("Hello Rustacean!\n\n");

        mmu::print_features(&uart);

        uart.puts("\nSwitching MMU on now...");
    } // After this closure, the UART instance is not valid anymore.

    unsafe { mmu::init() };

    // Instantiate a new UART using the virtual mapping in the second 2 MiB
    // block. No need to init() again, though.
    const UART_VIRT_BASE: u32 = 2 * 1024 * 1024 + 0x1000;
    let uart = uart::Uart::new(UART_VIRT_BASE);

    uart.puts("MMU is live \\o/\n\nWriting through the virtual mapping at 0x");
    uart.hex(UART_VIRT_BASE);
    uart.puts(".\n");

    // echo everything back
    loop {
        uart.send(uart.getc());
    }
}
