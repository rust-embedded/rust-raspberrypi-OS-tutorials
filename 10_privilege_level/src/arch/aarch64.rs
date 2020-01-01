// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! AArch64.

mod exception;
pub mod sync;
mod time;

use crate::{bsp, interface};
use cortex_a::{asm, regs::*};

/// The entry of the `kernel` binary.
///
/// The function must be named `_start`, because the linker is looking for this exact name.
///
/// # Safety
///
/// - Linker script must ensure to place this function at `0x80_000`.
#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    const CORE_MASK: u64 = 0x3;

    // Expect the boot core to start in EL2.
    if (bsp::BOOT_CORE_ID == MPIDR_EL1.get() & CORE_MASK)
        && (CurrentEL.get() == CurrentEL::EL::EL2.value)
    {
        el2_to_el1_transition()
    } else {
        // If not core0, infinitely wait for events.
        wait_forever()
    }
}

/// Transition from EL2 to EL1.
///
/// # Safety
///
/// - The HW state of EL1 must be prepared in a sound way.
/// - Exception return from EL2 must must continue execution in EL1 with ´runtime_init::init()`.
#[inline(always)]
unsafe fn el2_to_el1_transition() -> ! {
    // Enable timer counter registers for EL1.
    CNTHCTL_EL2.write(CNTHCTL_EL2::EL1PCEN::SET + CNTHCTL_EL2::EL1PCTEN::SET);

    // No offset for reading the counters.
    CNTVOFF_EL2.set(0);

    // Set EL1 execution state to AArch64.
    HCR_EL2.write(HCR_EL2::RW::EL1IsAarch64);

    // Set up a simulated exception return.
    //
    // First, fake a saved program status, where all interrupts were masked and SP_EL1 was used as a
    // stack pointer.
    SPSR_EL2.write(
        SPSR_EL2::D::Masked
            + SPSR_EL2::A::Masked
            + SPSR_EL2::I::Masked
            + SPSR_EL2::F::Masked
            + SPSR_EL2::M::EL1h,
    );

    // Second, let the link register point to init().
    ELR_EL2.set(crate::runtime_init::runtime_init as *const () as u64);

    // Set up SP_EL1 (stack pointer), which will be used by EL1 once we "return" to it.
    SP_EL1.set(bsp::BOOT_CORE_STACK_START);

    // Use `eret` to "return" to EL1. This will result in execution of `reset()` in EL1.
    asm::eret()
}

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

static TIMER: time::Timer = time::Timer;

//--------------------------------------------------------------------------------------------------
// Implementation of the kernel's architecture abstraction code
//--------------------------------------------------------------------------------------------------

pub use asm::nop;

/// Spin for `n` cycles.
pub fn spin_for_cycles(n: usize) {
    for _ in 0..n {
        asm::nop();
    }
}

/// Return a reference to a `interface::time::TimeKeeper` implementation.
pub fn timer() -> &'static impl interface::time::Timer {
    &TIMER
}

/// Pause execution on the calling CPU core.
#[inline(always)]
pub fn wait_forever() -> ! {
    loop {
        asm::wfe()
    }
}

/// Information about the HW state.
pub mod state {
    use crate::arch::PrivilegeLevel;
    use cortex_a::regs::*;

    /// The processing element's current privilege level.
    pub fn current_privilege_level() -> (PrivilegeLevel, &'static str) {
        let el = CurrentEL.read_as_enum(CurrentEL::EL);
        match el {
            Some(CurrentEL::EL::Value::EL2) => (PrivilegeLevel::Hypervisor, "EL2"),
            Some(CurrentEL::EL::Value::EL1) => (PrivilegeLevel::Kernel, "EL1"),
            Some(CurrentEL::EL::Value::EL0) => (PrivilegeLevel::User, "EL0"),
            _ => (PrivilegeLevel::Unknown, "Unknown"),
        }
    }

    /// Print the AArch64 exceptions status.
    #[rustfmt::skip]
    pub fn print_exception_state() {
        use super::{
            exception,
            exception::{Debug, SError, FIQ, IRQ},
        };
        use crate::info;

        let to_mask_str = |x| -> _ {
            if x { "Masked" } else { "Unmasked" }
        };

        info!("      Debug:  {}", to_mask_str(exception::is_masked::<Debug>()));
        info!("      SError: {}", to_mask_str(exception::is_masked::<SError>()));
        info!("      IRQ:    {}", to_mask_str(exception::is_masked::<IRQ>()));
        info!("      FIQ:    {}", to_mask_str(exception::is_masked::<FIQ>()));
    }
}
