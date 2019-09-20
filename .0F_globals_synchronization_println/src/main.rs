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
#![feature(const_fn)]
#![feature(custom_attribute)]
#![feature(format_args_nl)]

mod delays;
mod devices;
mod macros;
mod memory;
mod sync;

/// The global console. Output of the print! and println! macros.
static CONSOLE: sync::NullLock<devices::virt::Console> =
    sync::NullLock::new(devices::virt::Console::new());

fn kernel_entry() -> ! {
    use devices::hw;
    use devices::virt::ConsoleOps;

    // This will be invisible, because CONSOLE is dispatching to the NullConsole
    // at this point in time.
    println!("Is there anybody out there?");

    //------------------------------------------------------------
    // Instantiate GPIO device
    //------------------------------------------------------------
    let gpio = hw::GPIO::new(memory::map::physical::GPIO_BASE);

    //------------------------------------------------------------
    // Instantiate Videocore Mailbox
    //------------------------------------------------------------
    let mut v_mbox = hw::VideocoreMbox::new(memory::map::physical::VIDEOCORE_MBOX_BASE);

    //------------------------------------------------------------
    // Instantiate PL011 UART and put it in CONSOLE
    //------------------------------------------------------------
    let uart = hw::Uart::new(memory::map::physical::UART_BASE);

    match uart.init(&mut v_mbox, &gpio) {
        Ok(_) => {
            CONSOLE.lock(|c| {
                // Moves uart into the global CONSOLE. It is not accessible
                // anymore for the remaining parts of kernel_entry().
                c.replace_with(uart.into());
            });

            println!("\n[0] UART is live!");
        }
        Err(_) => loop {
            cortex_a::asm::wfe() // If UART fails, abort early
        },
    }

    //------------------------------------------------------------
    // Greet the user
    //------------------------------------------------------------
    print!("[1] Press a key to continue booting... ");
    CONSOLE.lock(|c| {
        c.getc();
    });
    println!("Greetings fellow Rustacean!");

    //------------------------------------------------------------
    // Bring up memory subsystem
    //------------------------------------------------------------
    if unsafe { memory::mmu::init() }.is_err() {
        println!("[2][Error] Could not set up MMU. Aborting.");
    } else {
        println!("[2] MMU online.");
    }

    memory::print_layout();

    //------------------------------------------------------------
    // Start a command prompt
    //------------------------------------------------------------
    CONSOLE.lock(|c| {
        c.command_prompt();
    })
}

raspi3_boot::entry!(kernel_entry);
