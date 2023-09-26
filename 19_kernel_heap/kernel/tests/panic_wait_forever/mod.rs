// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2022-2023 Andre Richter <andre.o.richter@gmail.com>

/// Overwrites libkernel's `panic_wait::_panic_exit()` with wait_forever.
#[no_mangle]
fn _panic_exit() -> ! {
    libkernel::cpu::wait_forever()
}
