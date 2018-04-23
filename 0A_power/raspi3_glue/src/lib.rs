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

#![feature(lang_items)]
#![no_std]

extern crate cortex_a;
extern crate panic_abort;
extern crate r0;

use core::ptr;
use cortex_a::{asm, register};

#[lang = "start"]
extern "C" fn start<T>(user_main: fn() -> T, _argc: isize, _argv: *const *const u8) -> isize
where
    T: Termination,
{
    user_main().report() as isize
}

#[lang = "termination"]
trait Termination {
    fn report(self) -> i32;
}

impl Termination for () {
    fn report(self) -> i32 {
        0
    }
}

unsafe fn reset() -> ! {
    extern "C" {
        fn main(argc: isize, argv: *const *const u8) -> isize;

        // Boundaries of the .bss section
        static mut __bss_start: u32;
        static mut __bss_end: u32;
    }

    // Zeroes the .bss section
    r0::zero_bss(&mut __bss_start, &mut __bss_end);

    main(0, ptr::null());

    loop {}
}

/// Entrypoint of the RPi3.
///
/// Parks all cores except core0, and then jumps to the internal
/// `reset()` function, which will call the user's `main()` after
/// initializing the `bss` section.
#[link_section = ".text.boot"]
#[no_mangle]
pub extern "C" fn _boot_cores() -> ! {
    match register::MPIDR_EL1::read_raw() & 0x3 {
        0 => unsafe {
            register::SP::write_raw(0x80_000);
            reset()
        },
        _ => loop {
            // if not core0, infinitely wait for events
            asm::wfe();
        },
    }
}
