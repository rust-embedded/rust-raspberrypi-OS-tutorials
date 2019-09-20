/*
 * MIT License
 *
 * Copyright (c) 2018 Jorge Aparicio
 * Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>
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

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]

//! Low-level boot of the Raspberry's processor

extern crate panic_abort;

/// Type check the user-supplied entry function.
#[macro_export]
macro_rules! entry {
    ($path:path) => {
        /// # Safety
        ///
        /// - User must ensure to provide a suitable main function for the
        ///   platform.
        #[export_name = "main"]
        pub unsafe fn __main() -> ! {
            // type check the given path
            let f: fn() -> ! = $path;

            f()
        }
    };
}

/// Reset function.
///
/// Initializes the bss section before calling into the user's `main()`.
///
/// # Safety
///
/// - Only a single core must be active and running this function.
unsafe fn reset() -> ! {
    extern "C" {
        // Boundaries of the .bss section, provided by the linker script
        static mut __bss_start: u64;
        static mut __bss_end: u64;
    }

    // Zeroes the .bss section
    r0::zero_bss(&mut __bss_start, &mut __bss_end);

    extern "Rust" {
        fn main() -> !;
    }

    main();
}

/// Entrypoint of the processor.
///
/// Parks all cores except core0, and then jumps to the internal
/// `reset()` function.
///
/// # Safety
///
/// - Linker script must ensure to place this function at `0x80_000`.
#[link_section = ".text.boot"]
#[no_mangle]
pub unsafe extern "C" fn _boot_cores() -> ! {
    use cortex_a::{asm, regs::*};

    const CORE_0: u64 = 0;
    const CORE_MASK: u64 = 0x3;
    const STACK_START: u64 = 0x80_000;

    if CORE_0 == MPIDR_EL1.get() & CORE_MASK {
        SP.set(STACK_START);
        reset()
    } else {
        // if not core0, infinitely wait for events
        loop {
            asm::wfe();
        }
    }
}
