/*
 * MIT License
 *
 * Copyright (c) 2018 Jorge Aparicio
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

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]
#![feature(global_asm)]

//! Low-level boot of the Raspberry's processor

extern crate cortex_a;
extern crate panic_abort;
extern crate r0;

#[macro_export]
macro_rules! entry {
    ($path:path) => {
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
#[no_mangle]
pub unsafe extern "C" fn reset() -> ! {
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

// Disable all cores except core 0, and then jump to reset()
global_asm!(include_str!("boot_cores.S"));
