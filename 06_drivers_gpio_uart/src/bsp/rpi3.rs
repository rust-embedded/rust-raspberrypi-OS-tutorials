// SPDX-License-Identifier: MIT
//
// Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>

//! Board Support Package for the Raspberry Pi 3.

mod driver;
mod memory_map;
mod panic_wait;
mod sync;

use crate::interface;
use cortex_a::{asm, regs::*};
use sync::NullLock;

/// The entry of the `kernel` binary.
///
/// The function must be named `_start`, because the linker is looking for this
/// exact name.
///
/// # Safety
///
/// - Linker script must ensure to place this function at `0x80_000`.
#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    use crate::runtime_init;

    const CORE_0: u64 = 0;
    const CORE_MASK: u64 = 0x3;
    const STACK_START: u64 = 0x80_000;

    if CORE_0 == MPIDR_EL1.get() & CORE_MASK {
        SP.set(STACK_START);
        runtime_init::init()
    } else {
        // if not core0, infinitely wait for events
        loop {
            asm::wfe();
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Global BSP driver instances
////////////////////////////////////////////////////////////////////////////////

static GPIO: driver::GPIO = unsafe { driver::GPIO::new(memory_map::mmio::GPIO_BASE) };
static MINI_UART: driver::MiniUart =
    unsafe { driver::MiniUart::new(memory_map::mmio::MINI_UART_BASE) };

////////////////////////////////////////////////////////////////////////////////
// Implementation of the kernel's BSP calls
////////////////////////////////////////////////////////////////////////////////

/// Park execution on the calling CPU core.
pub fn wait_forever() -> ! {
    loop {
        asm::wfe()
    }
}

/// Return a reference to a `console::All` implementation.
pub fn console() -> &'static impl interface::console::All {
    &MINI_UART
}

/// Return an array of references to all `DeviceDriver` compatible `BSP`
/// drivers.
///
/// # Safety
///
/// The order of devices is the order in which `DeviceDriver::init()` is called.
pub fn device_drivers() -> [&'static dyn interface::driver::DeviceDriver; 2] {
    [&GPIO, &MINI_UART]
}
