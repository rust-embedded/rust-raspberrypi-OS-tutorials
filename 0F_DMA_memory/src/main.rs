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
#![feature(allocator_api)]
#![feature(const_fn)]
#![feature(custom_attribute)]
#![feature(format_args_nl)]
#![feature(label_break_value)]
#![feature(range_contains)]

mod delays;
mod devices;
mod macros;
mod memory;
mod sync;

/// The global console. Output of the print! and println! macros.
static CONSOLE: sync::NullLock<devices::virt::Console> =
    sync::NullLock::new(devices::virt::Console::new());

/// The global allocator for DMA-able memory. That is, memory which is tagged
/// non-cacheable in the page tables.
static DMA_ALLOCATOR: sync::NullLock<memory::BumpAllocator> =
    sync::NullLock::new(memory::BumpAllocator::new(
        memory::map::virt::DMA_HEAP_START as usize,
        memory::map::virt::DMA_HEAP_END as usize,
        "Global DMA Allocator",
        // Try the following arguments instead to see the PL011 UART init
        // fail. It will cause the allocator to use memory that are marked
        // cacheable and therefore not DMA-safe. The answer from the Videocore
        // won't be received by the CPU because it reads an old cached value
        // that resembles an error case instead.

        // 0x00600000 as usize,
        // 0x007FFFFF as usize,
        // "Global Non-DMA Allocator",
    ));

fn kernel_entry() -> ! {
    use devices::hw;
    use devices::virt::ConsoleOps;

    //------------------------------------------------------------
    // Instantiate GPIO device
    //------------------------------------------------------------
    let gpio = hw::GPIO::new(memory::map::physical::GPIO_BASE);

    //------------------------------------------------------------
    // Instantiate MiniUart
    //------------------------------------------------------------
    let mini_uart = hw::MiniUart::new(memory::map::physical::MINI_UART_BASE);
    mini_uart.init(&gpio);

    CONSOLE.lock(|c| {
        // Moves mini_uart into the global CONSOLE. It is not accessible anymore
        // for the remaining parts of kernel_entry().
        c.replace_with(mini_uart.into());
    });
    println!("\n[0] MiniUart online.");

    //------------------------------------------------------------
    // Greet the user
    //------------------------------------------------------------
    print!("[1] Press a key to continue booting... ");
    CONSOLE.lock(|c| {
        c.getc();
    });
    println!("Greetings fellow Rustacean!");

    // We are now in a state where every next step can fail, but we can handle
    // the error with feedback for the user and fall through to our UART
    // loopback.
    'init: {
        //------------------------------------------------------------
        // Bring up memory subsystem
        //------------------------------------------------------------
        if unsafe { memory::mmu::init() }.is_err() {
            println!("[2][Error] Could not set up MMU. Aborting.");
            break 'init;
        };
        println!("[2] MMU online.");

        memory::print_layout();

        //------------------------------------------------------------
        // Instantiate Videocore Mailbox
        //------------------------------------------------------------
        let mut v_mbox;
        match hw::VideocoreMbox::new(memory::map::physical::VIDEOCORE_MBOX_BASE) {
            Ok(i) => {
                println!("[3] Videocore Mailbox set up (DMA mem heap allocation successful).");
                v_mbox = i;
            }

            Err(_) => {
                println!("[3][Error] Could not set up Videocore Mailbox. Aborting.");
                break 'init;
            }
        }

        //------------------------------------------------------------
        // Instantiate PL011 UART and replace MiniUart with it in CONSOLE
        //------------------------------------------------------------
        let pl011_uart = hw::PL011Uart::new(memory::map::physical::PL011_UART_BASE);

        // uart.init() will reconfigure the GPIO, which causes a race against
        // the MiniUart that is still putting out characters on the physical
        // line that are already buffered in its TX FIFO.
        //
        // To ensure the CPU doesn't rewire the GPIO before the MiniUart has put
        // its last character, explicitly flush it before rewiring.
        //
        // If you switch to an output that happens to not use the same pair of
        // physical wires (e.g. the Framebuffer), you don't need to do this,
        // because flush() is anyways called implicitly by replace_with(). This
        // is just a special case.
        CONSOLE.lock(|c| c.flush());
        match pl011_uart.init(&mut v_mbox, &gpio) {
            Ok(_) => {
                CONSOLE.lock(|c| {
                    c.replace_with(pl011_uart.into());
                });

                println!("[4] PL011 UART online. Output switched to it.");
            }

            Err(_) => println!(
                "[4][Error] PL011 UART init failed. \
                 Trying to continue with MiniUart."
            ),
        }
    }

    //------------------------------------------------------------
    // Start a command prompt
    //------------------------------------------------------------
    CONSOLE.lock(|c| {
        c.command_prompt();
    })
}

raspi3_boot::entry!(kernel_entry);
