// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! Architectural asynchronous exception handling.

use cortex_a::regs::*;

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

mod daif_bits {
    pub const IRQ: u8 = 0b0010;
}

trait DaifField {
    fn daif_field() -> register::Field<u32, DAIF::Register>;
}

struct Debug;
struct SError;
struct IRQ;
struct FIQ;

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

impl DaifField for Debug {
    fn daif_field() -> register::Field<u32, DAIF::Register> {
        DAIF::D
    }
}

impl DaifField for SError {
    fn daif_field() -> register::Field<u32, DAIF::Register> {
        DAIF::A
    }
}

impl DaifField for IRQ {
    fn daif_field() -> register::Field<u32, DAIF::Register> {
        DAIF::I
    }
}

impl DaifField for FIQ {
    fn daif_field() -> register::Field<u32, DAIF::Register> {
        DAIF::F
    }
}

fn is_masked<T: DaifField>() -> bool {
    DAIF.is_set(T::daif_field())
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Returns whether IRQs are masked on the executing core.
pub fn is_local_irq_masked() -> bool {
    !is_masked::<IRQ>()
}

/// Unmask IRQs on the executing core.
///
/// It is not needed to place an explicit instruction synchronization barrier after the `msr`.
/// Quoting the Architecture Reference Manual for ARMv8-A, section C5.1.3:
///
/// "Writes to PSTATE.{PAN, D, A, I, F} occur in program order without the need for additional
/// synchronization."
///
/// # Safety
///
/// - Changes the HW state of the executing core.
#[inline(always)]
pub unsafe fn local_irq_unmask() {
    #[rustfmt::skip]
    asm!(
        "msr DAIFClr, {arg}",
        arg = const daif_bits::IRQ,
        options(nomem, nostack, preserves_flags)
    );
}

/// Mask IRQs on the executing core.
///
/// # Safety
///
/// - Changes the HW state of the executing core.
#[inline(always)]
pub unsafe fn local_irq_mask() {
    #[rustfmt::skip]
    asm!(
        "msr DAIFSet, {arg}",
        arg = const daif_bits::IRQ,
        options(nomem, nostack, preserves_flags)
    );
}

/// Mask IRQs on the executing core and return the previously saved interrupt mask bits (DAIF).
///
/// # Safety
///
/// - Changes the HW state of the executing core.
#[inline(always)]
pub unsafe fn local_irq_mask_save() -> u32 {
    let saved = DAIF.get();
    local_irq_mask();

    saved
}

/// Restore the interrupt mask bits (DAIF) using the callee's argument.
///
/// # Safety
///
/// - Changes the HW state of the executing core.
/// - No sanity checks on the input.
#[inline(always)]
pub unsafe fn local_irq_restore(saved: u32) {
    DAIF.set(saved);
}

/// Print the AArch64 exceptions status.
#[rustfmt::skip]
pub fn print_state() {
    use crate::info;

    let to_mask_str = |x| -> _ {
        if x { "Masked" } else { "Unmasked" }
    };

    info!("      Debug:  {}", to_mask_str(is_masked::<Debug>()));
    info!("      SError: {}", to_mask_str(is_masked::<SError>()));
    info!("      IRQ:    {}", to_mask_str(is_masked::<IRQ>()));
    info!("      FIQ:    {}", to_mask_str(is_masked::<FIQ>()));
}
