// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2019-2023 Andre Richter <andre.o.richter@gmail.com>

/// Overwrites libkernel's `panic_wait::_panic_exit()` with the QEMU-exit version.
#[no_mangle]
fn _panic_exit() -> ! {
    libkernel::cpu::qemu_exit_success()
}
