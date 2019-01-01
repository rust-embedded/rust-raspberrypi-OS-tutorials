/*
 * MIT License
 *
 * Copyright (c) 2018 Andre Richter <andre.o.richter@gmail.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

use super::UART;
use cortex_a::{asm, barrier, regs::*};

global_asm!(include_str!("vectors.S"));

pub unsafe fn set_vbar_el1_checked(vec_base_addr: u64) -> bool {
    if vec_base_addr.trailing_zeros() < 11 {
        false
    } else {
        cortex_a::regs::VBAR_EL1.set(vec_base_addr);

        // Force VBAR update to complete before next instruction.
        barrier::isb(barrier::SY);

        true
    }
}

#[repr(C)]
pub struct GPR {
    x: [u64; 31],
}

#[repr(C)]
pub struct ExceptionContext {
    // General Purpose Registers
    gpr: GPR,
    spsr_el1: u64,
    elr_el1: u64,
}

macro_rules! exception_return {
    () => {
        asm! {"RESTORE_CONTEXT"}

        asm::eret();
    };
}

/// The default exception, invoked for every exception type unless the handler
/// is overwritten.
#[naked]
#[no_mangle]
unsafe extern "C" fn default_exception_handler() {
    UART.puts("Unexpected exception. Halting CPU.\n");

    loop {
        cortex_a::asm::wfe()
    }
}

// To implement an exception handler, overwrite it by defining the respective
// function below.
// Don't forget:
//     - The #[naked] attribute
//     - The #[no_mangle] attribute
//     - The exception_return! macro if suitable.
//
// unsafe extern "C" fn current_el0_synchronous(e: &mut ExceptionContext);
// unsafe extern "C" fn current_el0_irq(e: &mut ExceptionContext);
// unsafe extern "C" fn current_el0_serror(e: &mut ExceptionContext);

// unsafe extern "C" fn current_elx_synchronous(e: &mut ExceptionContext);
// unsafe extern "C" fn current_elx_irq(e: &mut ExceptionContext);
// unsafe extern "C" fn current_elx_serror(e: &mut ExceptionContext);

// unsafe extern "C" fn lower_aarch64_synchronous(e: &mut ExceptionContext);
// unsafe extern "C" fn lower_aarch64_irq(e: &mut ExceptionContext);
// unsafe extern "C" fn lower_aarch64_serror(e: &mut ExceptionContext);

// unsafe extern "C" fn lower_aarch32_synchronous(e: &mut ExceptionContext);
// unsafe extern "C" fn lower_aarch32_irq(e: &mut ExceptionContext);
// unsafe extern "C" fn lower_aarch32_serror(e: &mut ExceptionContext);

#[naked]
#[no_mangle]
unsafe extern "C" fn current_elx_synchronous(e: &mut ExceptionContext) {
    UART.puts("A synchronous exception happened.\n");
    UART.puts("    ELR_EL1: 0x");
    UART.hex(e.elr_el1);
    UART.puts(
        "\n    Incrementing ELR_EL1 by 4 now to continue with the first \
         instruction after the exception!\n",
    );

    e.elr_el1 += 4;

    UART.puts("    ELR_EL1 modified: 0x");
    UART.hex(e.elr_el1);
    UART.puts("\n");

    UART.puts("    Returning from exception...\n\n");

    exception_return!();
}
