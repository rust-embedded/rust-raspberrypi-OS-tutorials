// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2019-2020 Andre Richter <andre.o.richter@gmail.com>

//! Timer sanity tests.

#![feature(custom_test_frameworks)]
#![no_main]
#![no_std]
#![reexport_test_harness_main = "test_main"]
#![test_runner(libkernel::test_runner)]

mod panic_exit_failure;

use core::time::Duration;
use libkernel::{arch, arch::timer, bsp, interface::time::Timer};
use test_macros::kernel_test;

#[no_mangle]
unsafe fn kernel_init() -> ! {
    bsp::qemu_bring_up_console();

    // Depending on CPU arch, some timer bring-up code could go here. Not needed for the RPi.

    test_main();

    arch::qemu_exit_success()
}

/// Simple check that the timer is running.
#[kernel_test]
fn timer_is_counting() {
    assert!(timer().uptime().as_nanos() > 0)
}

/// Timer resolution must be sufficient.
#[kernel_test]
fn timer_resolution_is_sufficient() {
    assert!(timer().resolution().as_nanos() < 100)
}

/// Sanity check spin_for() implementation.
#[kernel_test]
fn spin_accuracy_check_1_second() {
    let t1 = timer().uptime();
    timer().spin_for(Duration::from_secs(1));
    let t2 = timer().uptime();

    assert_eq!((t2 - t1).as_secs(), 1)
}
