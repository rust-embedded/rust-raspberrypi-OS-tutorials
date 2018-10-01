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

extern crate cortex_a;

#[macro_use]
extern crate raspi3_boot;

#[macro_use]
extern crate register;

const MMIO_BASE: u32 = 0x3F00_0000;

mod gpio;
mod mbox;
mod mmu;
mod uart;
mod benchmark;

fn do_benchmarks(uart: &uart::Uart) {
    const SIZE_2MIB: u64 = 2 * 1024 * 1024;

    // Start of the __SECOND__ virtual 2 MiB block (counting starts at zero).
    // NON-cacheable DRAM memory.
    let non_cacheable_addr: u64 = SIZE_2MIB;

    // Start of the __THIRD__ virtual 2 MiB block.
    // Cacheable DRAM memory
    let cacheable_addr: u64 = 2 * SIZE_2MIB;

    uart.puts("Benchmarking non-cacheable DRAM modifications at virtual 0x");
    uart.hex(non_cacheable_addr as u32);
    uart.puts(", physical 0x");
    uart.hex(2 * SIZE_2MIB as u32);
    uart.puts(":\n");

    let result_nc = benchmark::batch_modify(non_cacheable_addr);
    uart.dec(result_nc);
    uart.puts(" miliseconds.\n\n");

    uart.puts("Benchmarking cacheable DRAM modifications at virtual 0x");
    uart.hex(cacheable_addr as u32);
    uart.puts(", physical 0x");
    uart.hex(2 * SIZE_2MIB as u32);
    uart.puts(":\n");
    let result_c = benchmark::batch_modify(cacheable_addr);
    uart.dec(result_c);
    uart.puts(" miliseconds.\n\n");

    let percent_diff = (result_nc - result_c) * 100 / result_c;

    uart.puts("With caching, the function is ");
    uart.dec(percent_diff);
    uart.puts("% faster!\n");
}

entry!(kernel_entry);

fn kernel_entry() -> ! {
    let mut mbox = mbox::Mbox::new();
    let uart = uart::Uart::new(uart::UART_PHYS_BASE);

    // set up serial console
    if uart.init(&mut mbox).is_err() {
        loop {
            cortex_a::asm::wfe() // If UART fails, abort early
        }
    }

    uart.getc(); // Press a key first before being greeted
    uart.puts("Hello Rustacean!\n\n");

    uart.puts("\nSwitching MMU on now...");

    unsafe { mmu::init() };

    uart.puts("MMU is live \\o/\n\n");

    do_benchmarks(&uart);

    // echo everything back
    loop {
        uart.send(uart.getc());
    }
}
