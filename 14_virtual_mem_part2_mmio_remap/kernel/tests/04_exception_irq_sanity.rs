// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020-2022 Andre Richter <andre.o.richter@gmail.com>

//! IRQ handling sanity tests.

#![feature(custom_test_frameworks)]
#![no_main]
#![no_std]
#![reexport_test_harness_main = "test_main"]
#![test_runner(libkernel::test_runner)]

use libkernel::{bsp, cpu, driver, exception, memory};
use test_macros::kernel_test;

#[no_mangle]
unsafe fn kernel_init() -> ! {
    use driver::interface::DriverManager;

    exception::handling_init();

    let phys_kernel_tables_base_addr = match memory::mmu::kernel_map_binary() {
        Err(string) => panic!("Error mapping kernel binary: {}", string),
        Ok(addr) => addr,
    };

    if let Err(e) = memory::mmu::enable_mmu_and_caching(phys_kernel_tables_base_addr) {
        panic!("Enabling MMU failed: {}", e);
    }
    // Printing will silently fail from here on, because the driver's MMIO is not remapped yet.

    memory::mmu::post_enable_init();
    bsp::driver::driver_manager().qemu_bring_up_console();
    // Printing available again from here on.

    exception::asynchronous::local_irq_unmask();

    test_main();

    cpu::qemu_exit_success()
}

/// Check that IRQ masking works.
#[kernel_test]
fn local_irq_mask_works() {
    // Precondition: IRQs are unmasked.
    assert!(exception::asynchronous::is_local_irq_masked());

    unsafe { exception::asynchronous::local_irq_mask() };
    assert!(!exception::asynchronous::is_local_irq_masked());

    // Restore earlier state.
    unsafe { exception::asynchronous::local_irq_unmask() };
}

/// Check that IRQ unmasking works.
#[kernel_test]
fn local_irq_unmask_works() {
    // Precondition: IRQs are masked.
    unsafe { exception::asynchronous::local_irq_mask() };
    assert!(!exception::asynchronous::is_local_irq_masked());

    unsafe { exception::asynchronous::local_irq_unmask() };
    assert!(exception::asynchronous::is_local_irq_masked());
}

/// Check that IRQ mask save is saving "something".
#[kernel_test]
fn local_irq_mask_save_works() {
    // Precondition: IRQs are unmasked.
    assert!(exception::asynchronous::is_local_irq_masked());

    let first = unsafe { exception::asynchronous::local_irq_mask_save() };
    assert!(!exception::asynchronous::is_local_irq_masked());

    let second = unsafe { exception::asynchronous::local_irq_mask_save() };
    assert_ne!(first, second);

    unsafe { exception::asynchronous::local_irq_restore(first) };
    assert!(exception::asynchronous::is_local_irq_masked());
}
