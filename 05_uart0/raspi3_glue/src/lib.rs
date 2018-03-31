// "Embedded glue code from "The Embedonomicon" by Jorge Aparicio,
// used under CC BY 4.0
//
// Minor changes and additions were made.
//
// Original Author: https://github.com/japaric
// License: https://creativecommons.org/licenses/by/4.0/

#![feature(lang_items)]
#![no_std]
#![feature(global_asm)]

use core::ptr;

#[lang = "panic_fmt"]
unsafe extern "C" fn panic_fmt(
    _args: core::fmt::Arguments,
    _file: &'static str,
    _line: u32,
    _col: u32,
) -> ! {
    loop {}
}

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

#[no_mangle]
pub unsafe extern "C" fn reset() -> ! {
    extern "C" {
        fn main(argc: isize, argv: *const *const u8) -> isize;

        static mut __bss_start: u32;
        static mut __bss_end: u32;
    }

    zero_bss(&mut __bss_start, &mut __bss_end);

    main(0, ptr::null());

    loop {}
}

unsafe fn zero_bss(bss_start: *mut u32, bss_end: *mut u32) {
    let mut bss = bss_start;
    while bss < bss_end {
        // NOTE(ptr::write*) to force aligned stores
        // NOTE(volatile) to prevent the compiler from optimizing this into `memclr`
        ptr::write_volatile(bss, 0);
        bss = bss.offset(1);
    }
}

// Disable all cores except core 0, and then jump to reset()
global_asm!(include_str!("boot_cores.S"));
