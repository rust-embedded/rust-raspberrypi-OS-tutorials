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
#![feature(global_asm)]
#![feature(label_break_value)]
#![feature(naked_functions)]

const MMIO_BASE: u32 = 0x3F00_0000;

mod delays;
mod exception;
mod gpio;
mod mbox;
mod mmu;
mod uart;

static UART: uart::Uart = uart::Uart::new(uart::UART_PHYS_BASE);

fn kernel_entry() -> ! {
    extern "C" {
        static __exception_vectors_start: u64;
    }

    let gpio = gpio::GPIO::new();
    let mut mbox = mbox::Mbox::new();

    // set up serial console
    match UART.init(&mut mbox, &gpio) {
        Ok(_) => UART.puts("\n[0] UART is live!\n"),
        Err(_) => loop {
            cortex_a::asm::wfe() // If UART fails, abort early
        },
    }

    UART.puts("[1] Press a key to continue booting... ");
    UART.getc();
    UART.puts("Greetings fellow Rustacean!\n");

    UART.puts("[2] Switching MMU on now... ");

    unsafe { mmu::init() };

    UART.puts("MMU is live \\o/\n");

    'init: {
        if unsafe {
            let exception_vectors_start: u64 = &__exception_vectors_start as *const _ as u64;

            exception::set_vbar_el1_checked(exception_vectors_start)
        } {
            UART.puts("[3] Exception vectors are set up.\n\n");
        } else {
            UART.puts("[3] Error setting exception vectors. Aborting early.\n");
            break 'init;
        }

        // Cause an exception by accessing a virtual address for which we set
        // the "Access Flag" to zero in the page tables.
        unsafe { core::ptr::read_volatile((2 * 1024 * 1024) as *const u64) };

        UART.puts("Whoa! We recovered from an exception.\n")
    }

    // echo everything back
    loop {
        UART.send(UART.getc());
    }
}

raspi3_boot::entry!(kernel_entry);
