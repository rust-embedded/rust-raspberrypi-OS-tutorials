// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

// Rust embedded logo for `make doc`.
#![doc(html_logo_url = "https://git.io/JeGIp")]

//! The `kernel`
//!
//! The `kernel` is composed by glueing together code from
//!
//!   - [Hardware-specific Board Support Packages] (`BSPs`).
//!   - [Architecture-specific code].
//!   - HW- and architecture-agnostic `kernel` code.
//!
//! using the [`kernel::interface`] traits.
//!
//! [Hardware-specific Board Support Packages]: bsp/index.html
//! [Architecture-specific code]: arch/index.html
//! [`kernel::interface`]: interface/index.html

#![feature(format_args_nl)]
#![feature(panic_info_message)]
#![feature(trait_alias)]
#![no_main]
#![no_std]

// Conditionally includes the selected `architecture` code, which provides the `_start()` function,
// the first function to run.
mod arch;

// `_start()` then calls `relocate::relocate_self()`.
mod relocate;

// `relocate::relocate_self()` calls `runtime_init()`, which on completion, jumps to
// `kernel_init()`.
mod runtime_init;

// Conditionally includes the selected `BSP` code.
mod bsp;

mod interface;
mod memory;
mod panic_wait;
mod print;

/// Early init code.
///
/// Concerned with with initializing `BSP` and `arch` parts.
///
/// # Safety
///
/// - Only a single core must be active and running this function.
/// - The init calls in this function must appear in the correct order.
unsafe fn kernel_init() -> ! {
    for i in bsp::device_drivers().iter() {
        if let Err(()) = i.init() {
            panic!("Error loading driver: {}", i.compatible())
        }
    }
    bsp::post_driver_init();
    // println! is usable from here on.

    // Transition from unsafe to safe.
    kernel_main()
}

/// The main function running after the early init.
fn kernel_main() -> ! {
    use interface::console::All;

    println!(" __  __ _      _ _                 _ ");
    println!("|  \\/  (_)_ _ (_) |   ___  __ _ __| |");
    println!("| |\\/| | | ' \\| | |__/ _ \\/ _` / _` |");
    println!("|_|  |_|_|_||_|_|____\\___/\\__,_\\__,_|");
    println!();
    println!("{:^37}", bsp::board_name());
    println!();
    println!("[ML] Requesting binary");
    bsp::console().flush();

    // Clear the RX FIFOs, if any, of spurious received characters before starting with the loader
    // protocol.
    bsp::console().clear();

    // Notify `Minipush` to send the binary.
    for _ in 0..3 {
        bsp::console().write_char(3 as char);
    }

    // Read the binary's size.
    let mut size: u32 = u32::from(bsp::console().read_char() as u8);
    size |= u32::from(bsp::console().read_char() as u8) << 8;
    size |= u32::from(bsp::console().read_char() as u8) << 16;
    size |= u32::from(bsp::console().read_char() as u8) << 24;

    // Trust it's not too big.
    bsp::console().write_char('O');
    bsp::console().write_char('K');

    let kernel_addr: *mut u8 = bsp::BOARD_DEFAULT_LOAD_ADDRESS as *mut u8;
    unsafe {
        // Read the kernel byte by byte.
        for i in 0..size {
            *kernel_addr.offset(i as isize) = bsp::console().read_char() as u8;
        }
    }

    println!("[ML] Loaded! Executing the payload now\n");
    bsp::console().flush();

    // Use black magic to get a function pointer.
    let kernel: extern "C" fn() -> ! = unsafe { core::mem::transmute(kernel_addr as *const ()) };

    // Jump to loaded kernel!
    kernel()
}
