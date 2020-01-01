// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! Exception handling.

use core::fmt;
use cortex_a::{asm, barrier, regs::*};
use register::InMemoryRegister;

// Assembly counterpart to this file.
global_asm!(include_str!("exception.S"));

/// Wrapper struct for memory copy of SPSR_EL1.
#[repr(transparent)]
struct SpsrEL1(InMemoryRegister<u32, SPSR_EL1::Register>);

/// The exception context as it is stored on the stack on exception entry.
#[repr(C)]
struct ExceptionContext {
    // General Purpose Registers.
    gpr: [u64; 30],
    // The link register, aka x30.
    lr: u64,
    // Exception link register. The program counter at the time the exception happened.
    elr_el1: u64,
    // Saved program status.
    spsr_el1: SpsrEL1,
}

/// Wrapper struct for pretty printing ESR_EL1.
struct EsrEL1;

//--------------------------------------------------------------------------------------------------
// Exception vector implementation
//--------------------------------------------------------------------------------------------------

/// Print verbose information about the exception and the panic.
fn default_exception_handler(e: &ExceptionContext) {
    panic!(
        "\n\nCPU Exception!\n\
         FAR_EL1: {:#018x}\n\
         {}\n\
         {}",
        FAR_EL1.get(),
        EsrEL1 {},
        e
    );
}

//--------------------------------------------------------------------------------------------------
// Current, EL0
//--------------------------------------------------------------------------------------------------

#[no_mangle]
unsafe extern "C" fn current_el0_synchronous(e: &mut ExceptionContext) {
    default_exception_handler(e);
}

#[no_mangle]
unsafe extern "C" fn current_el0_irq(e: &mut ExceptionContext) {
    default_exception_handler(e);
}

#[no_mangle]
unsafe extern "C" fn current_el0_serror(e: &mut ExceptionContext) {
    default_exception_handler(e);
}

//--------------------------------------------------------------------------------------------------
// Current, ELx
//--------------------------------------------------------------------------------------------------

/// Asynchronous exception taken from the current EL, using SP of the current EL.
#[no_mangle]
unsafe extern "C" fn current_elx_synchronous(e: &mut ExceptionContext) {
    let far_el1 = FAR_EL1.get();

    // This catches the demo case for this tutorial. If the fault address happens to be 8 GiB,
    // advance the exception link register for one instruction, so that execution can continue.
    if far_el1 == 8 * 1024 * 1024 * 1024 {
        e.elr_el1 += 4;

        asm::eret()
    }

    default_exception_handler(e);
}

#[no_mangle]
unsafe extern "C" fn current_elx_irq(e: &mut ExceptionContext) {
    default_exception_handler(e);
}

#[no_mangle]
unsafe extern "C" fn current_elx_serror(e: &mut ExceptionContext) {
    default_exception_handler(e);
}

//--------------------------------------------------------------------------------------------------
// Lower, AArch64
//--------------------------------------------------------------------------------------------------

#[no_mangle]
unsafe extern "C" fn lower_aarch64_synchronous(e: &mut ExceptionContext) {
    default_exception_handler(e);
}

#[no_mangle]
unsafe extern "C" fn lower_aarch64_irq(e: &mut ExceptionContext) {
    default_exception_handler(e);
}

#[no_mangle]
unsafe extern "C" fn lower_aarch64_serror(e: &mut ExceptionContext) {
    default_exception_handler(e);
}

//--------------------------------------------------------------------------------------------------
// Lower, AArch32
//--------------------------------------------------------------------------------------------------

#[no_mangle]
unsafe extern "C" fn lower_aarch32_synchronous(e: &mut ExceptionContext) {
    default_exception_handler(e);
}

#[no_mangle]
unsafe extern "C" fn lower_aarch32_irq(e: &mut ExceptionContext) {
    default_exception_handler(e);
}

#[no_mangle]
unsafe extern "C" fn lower_aarch32_serror(e: &mut ExceptionContext) {
    default_exception_handler(e);
}

//--------------------------------------------------------------------------------------------------
// Pretty printing
//--------------------------------------------------------------------------------------------------

/// Human readable ESR_EL1.
#[rustfmt::skip]
impl fmt::Display for EsrEL1 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let esr_el1 = ESR_EL1.extract();

        // Raw print of whole register.
        writeln!(f, "ESR_EL1: {:#010x}", esr_el1.get())?;

        // Raw print of exception class.
        write!(f, "      Exception Class         (EC) : {:#x}", esr_el1.read(ESR_EL1::EC))?;

        // Exception class, translation.
        let ec_translation = match esr_el1.read_as_enum(ESR_EL1::EC) {
            Some(ESR_EL1::EC::Value::DataAbortCurrentEL) => "Data Abort, current EL",
            _ => "N/A",
        };
        writeln!(f, " - {}", ec_translation)?;

        // Raw print of instruction specific syndrome.
        write!(f, "      Instr Specific Syndrome (ISS): {:#x}", esr_el1.read(ESR_EL1::ISS))?;

        Ok(())
    }
}

/// Human readable SPSR_EL1.
#[rustfmt::skip]
impl fmt::Display for SpsrEL1 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Raw value.
        writeln!(f, "SPSR_EL1: {:#010x}", self.0.get())?;

        let to_flag_str = |x| -> _ {
            if x { "Set" } else { "Not set" }
         };

        writeln!(f, "      Flags:")?;
        writeln!(f, "            Negative (N): {}", to_flag_str(self.0.is_set(SPSR_EL1::N)))?;
        writeln!(f, "            Zero     (Z): {}", to_flag_str(self.0.is_set(SPSR_EL1::Z)))?;
        writeln!(f, "            Carry    (C): {}", to_flag_str(self.0.is_set(SPSR_EL1::C)))?;
        writeln!(f, "            Overflow (V): {}", to_flag_str(self.0.is_set(SPSR_EL1::V)))?;

        let to_mask_str = |x| -> _ {
            if x { "Masked" } else { "Unmasked" }
        };

        writeln!(f, "      Exception handling state:")?;
        writeln!(f, "            Debug  (D): {}", to_mask_str(self.0.is_set(SPSR_EL1::D)))?;
        writeln!(f, "            SError (A): {}", to_mask_str(self.0.is_set(SPSR_EL1::A)))?;
        writeln!(f, "            IRQ    (I): {}", to_mask_str(self.0.is_set(SPSR_EL1::I)))?;
        writeln!(f, "            FIQ    (F): {}", to_mask_str(self.0.is_set(SPSR_EL1::F)))?;

        write!(f, "      Illegal Execution State (IL): {}",
            to_flag_str(self.0.is_set(SPSR_EL1::IL))
        )?;

        Ok(())
    }
}

/// Human readable print of the exception context.
impl fmt::Display for ExceptionContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "ELR_EL1: {:#018x}", self.elr_el1)?;
        writeln!(f, "{}", self.spsr_el1)?;
        writeln!(f)?;
        writeln!(f, "General purpose register:")?;

        #[rustfmt::skip]
        let alternating = |x| -> _ {
            if x % 2 == 0 { "   " } else { "\n" }
        };

        // Print two registers per line.
        for (i, reg) in self.gpr.iter().enumerate() {
            write!(f, "      x{: <2}: {: >#018x}{}", i, reg, alternating(i))?;
        }
        write!(f, "      lr : {:#018x}", self.lr)?;

        Ok(())
    }
}

//--------------------------------------------------------------------------------------------------
// Arch-public
//--------------------------------------------------------------------------------------------------

/// Set the exception vector base address register.
///
/// # Safety
///
/// - The vector table and the symbol `__exception_vector_table_start` from the linker script must
///   adhere to the alignment and size constraints demanded by the AArch64 spec.
pub unsafe fn set_vbar_el1() {
    // Provided by exception.S.
    extern "C" {
        static mut __exception_vector_start: u64;
    }
    let addr: u64 = &__exception_vector_start as *const _ as u64;

    VBAR_EL1.set(addr);

    // Force VBAR update to complete before next instruction.
    barrier::isb(barrier::SY);
}

pub trait DaifField {
    fn daif_field() -> register::Field<u32, DAIF::Register>;
}

pub struct Debug;
pub struct SError;
pub struct IRQ;
pub struct FIQ;

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

pub fn is_masked<T: DaifField>() -> bool {
    DAIF.is_set(T::daif_field())
}
