// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

// Rust embedded logo for `make doc`.
#![doc(html_logo_url = "https://git.io/JeGIp")]

//! The `kernel` binary.

#![feature(format_args_nl)]
#![no_main]
#![no_std]

use libkernel::{bsp, cpu, driver, exception, info, memory, state, time, warn};

/// Early init code.
///
/// When this code runs, virtual memory is already enabled.
///
/// # Safety
///
/// - Only a single core must be active and running this function.
/// - Printing will not work until the respective driver's MMIO is remapped.
#[no_mangle]
unsafe fn kernel_init() -> ! {
    use driver::interface::DriverManager;

    exception::handling_init();

    // Add the mapping records for the precomputed entries first, so that they appear on the top of
    // the list.
    bsp::memory::mmu::kernel_add_mapping_records_for_precomputed();

    // Bring up the drivers needed for printing first.
    for i in bsp::driver::driver_manager()
        .early_print_device_drivers()
        .iter()
    {
        // Any encountered errors cannot be printed yet, obviously, so just safely park the CPU.
        i.init().unwrap_or_else(|_| cpu::wait_forever());
    }
    bsp::driver::driver_manager().post_early_print_device_driver_init();
    // Printing available from here on.

    // Now bring up the remaining drivers.
    for i in bsp::driver::driver_manager()
        .non_early_print_device_drivers()
        .iter()
    {
        if let Err(x) = i.init() {
            panic!("Error loading driver: {}: {}", i.compatible(), x);
        }
    }

    // Let device drivers register and enable their handlers with the interrupt controller.
    for i in bsp::driver::driver_manager().all_device_drivers() {
        if let Err(msg) = i.register_and_enable_irq_handler() {
            warn!("Error registering IRQ handler: {}", msg);
        }
    }

    // Unmask interrupts on the boot CPU core.
    exception::asynchronous::local_irq_unmask();

    // Announce conclusion of the kernel_init() phase.
    state::state_manager().transition_to_single_core_main();

    // Transition from unsafe to safe.
    kernel_main()
}

/// The main function running after the early init.
fn kernel_main() -> ! {
    use driver::interface::DriverManager;
    use exception::asynchronous::interface::IRQManager;

    info!("{}", libkernel::version());
    info!("Booting on: {}", bsp::board_name());

    info!("MMU online:");
    memory::mmu::kernel_print_mappings();

    let (_, privilege_level) = exception::current_privilege_level();
    info!("Current privilege level: {}", privilege_level);

    info!("Exception handling state:");
    exception::asynchronous::print_state();

    info!(
        "Architectural timer resolution: {} ns",
        time::time_manager().resolution().as_nanos()
    );

    info!("Drivers loaded:");
    for (i, driver) in bsp::driver::driver_manager()
        .all_device_drivers()
        .iter()
        .enumerate()
    {
        info!("      {}. {}", i + 1, driver.compatible());
    }

    info!("Registered IRQ handlers:");
    bsp::exception::asynchronous::irq_manager().print_handler();

    info!("Echoing input now");
    cpu::wait_forever();
}
