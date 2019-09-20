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

const MMIO_BASE: usize = 0x3F00_0000;
const GPIO_BASE: usize = MMIO_BASE + 0x0020_0000;
const MINI_UART_BASE: usize = MMIO_BASE + 0x0021_5000;

mod gpio;
mod mini_uart;

pub fn setup_jtag(gpio: &gpio::GPIO) {
    gpio.GPFSEL2.modify(
        gpio::GPFSEL2::FSEL27::ARM_TMS
            + gpio::GPFSEL2::FSEL26::ARM_TDI
            + gpio::GPFSEL2::FSEL25::ARM_TCK
            + gpio::GPFSEL2::FSEL24::ARM_TDO
            + gpio::GPFSEL2::FSEL23::ARM_RTCK
            + gpio::GPFSEL2::FSEL22::ARM_TRST,
    );
}

fn kernel_entry() -> ! {
    let gpio = gpio::GPIO::new(GPIO_BASE);

    //------------------------------------------------------------
    // Instantiate MiniUart
    //------------------------------------------------------------
    let mini_uart = mini_uart::MiniUart::new(MINI_UART_BASE);
    mini_uart.init(&gpio);

    //------------------------------------------------------------
    // Configure JTAG pins
    //------------------------------------------------------------
    setup_jtag(&gpio);

    mini_uart.puts("\n[i] JTAG is live. Please connect.\n");

    loop {}
}

raspi3_boot::entry!(kernel_entry);
