/*
 * MIT License
 *
 * Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>
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

mod delays;
mod gpio;
mod mbox;
mod memory;
mod uart;

fn kernel_entry() -> ! {
    let gpio = gpio::GPIO::new(memory::map::physical::GPIO_BASE);
    let mut mbox = mbox::Mbox::new(memory::map::physical::VIDEOCORE_MBOX_BASE);

    {
        // Before the MMU is live, instantiate a UART driver with the physical address
        let uart = uart::Uart::new(memory::map::physical::UART_BASE);

        // set up serial console
        match uart.init(&mut mbox, &gpio) {
            Ok(_) => uart.puts("\n[0] UART is live!\n"),
            Err(_) => loop {
                cortex_a::asm::wfe() // If UART fails, abort early
            },
        }

        uart.puts("[1] Press a key to continue booting... ");
        uart.getc();
        uart.puts("Greetings fellow Rustacean!\n");

        memory::mmu::print_features(&uart);

        match unsafe { memory::mmu::init() } {
            Err(s) => {
                uart.puts("[2][Error] MMU: ");
                uart.puts(s);
                uart.puts("\n");
            }
            // The following write is already using the identity mapped
            // translation in the LVL2 table.
            Ok(()) => uart.puts("[2] MMU online.\n"),
        }
    } // After this closure, the UART instance is not valid anymore.

    // Instantiate a new UART using the remapped address. No need to init()
    // again, though.
    let uart = uart::Uart::new(memory::map::virt::REMAPPED_UART_BASE);

    uart.puts("\nWriting through the virtual mapping at base address 0x");
    uart.hex(memory::map::virt::REMAPPED_UART_BASE as u64);
    uart.puts(".\n");

    // echo everything back
    loop {
        uart.send(uart.getc());
    }
}

raspi3_boot::entry!(kernel_entry);
