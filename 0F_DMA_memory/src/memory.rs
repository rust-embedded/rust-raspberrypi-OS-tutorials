/*
 * MIT License
 *
 * Copyright (c) 2019 Andre Richter <andre.o.richter@gmail.com>
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

use crate::println;

mod bump_allocator;
pub use bump_allocator::BumpAllocator;

pub mod mmu;

/// The system memory map.
#[rustfmt::skip]
pub mod map {
    pub const KERN_STACK_BOT:      u32 =             0x0000_0000;
    pub const KERN_STACK_TOP:      u32 =             0x0007_FFFF;

    /// The second 2 MiB block.
    pub const DMA_HEAP_START:      u32 =             0x0020_0000;
    pub const DMA_HEAP_END:        u32 =             0x005F_FFFF;

    pub const MMIO_BASE:           u32 =             0x3F00_0000;
    pub const VIDEOCORE_MBOX_BASE: u32 = MMIO_BASE + 0x0000_B880;
    pub const GPIO_BASE:           u32 = MMIO_BASE + 0x0020_0000;
    pub const PL011_UART_BASE:     u32 = MMIO_BASE + 0x0020_1000;
    pub const MINI_UART_BASE:      u32 = MMIO_BASE + 0x0021_5000;

    pub const PHYS_ADDR_MAX:       u32 =             0x3FFF_FFFF;
}

const PAGESIZE: u64 = 4096;

#[inline]
fn aligned_addr_unchecked(addr: usize, alignment: usize) -> usize {
    (addr + (alignment - 1)) & !(alignment - 1)
}

fn get_ro_start_end() -> (u64, u64) {
    // Using the linker script, we ensure that the RO area is consecutive and 4
    // KiB aligned, and we export the boundaries via symbols.
    extern "C" {
        // The inclusive start of the read-only area, aka the address of the
        // first byte of the area.
        static __ro_start: u64;

        // The non-inclusive end of the read-only area, aka the address of the
        // first byte _after_ the RO area.
        static __ro_end: u64;
    }

    unsafe {
        // Notice the subtraction to calculate the last page index of the RO
        // area and not the first page index after the RO area.
        (
            &__ro_start as *const _ as u64,
            &__ro_end as *const _ as u64 - 1,
        )
    }
}

pub fn print_layout() {
    use crate::memory::map::*;

    // log2(1024)
    const KIB_RSHIFT: u32 = 10;

    // log2(1024 * 1024)
    const MIB_RSHIFT: u32 = 20;

    println!("[i] Memory layout:");

    println!(
        "      {:#010X} - {:#010X} | {: >4} KiB | Kernel stack",
        KERN_STACK_BOT,
        KERN_STACK_TOP,
        (KERN_STACK_TOP - KERN_STACK_BOT + 1) >> KIB_RSHIFT
    );

    let (ro_start, ro_end) = get_ro_start_end();
    println!(
        "      {:#010X} - {:#010X} | {: >4} KiB | Kernel code and RO data",
        ro_start,
        ro_end,
        (ro_end - ro_start + 1) >> KIB_RSHIFT
    );

    extern "C" {
        static __bss_end: u64;
    }

    let start = ro_end + 1;
    let end = unsafe { &__bss_end as *const _ as u64 } - 1;
    println!(
        "      {:#010X} - {:#010X} | {: >4} KiB | Kernel data and BSS",
        start,
        end,
        (end - start + 1) >> KIB_RSHIFT
    );

    println!(
        "      {:#010X} - {:#010X} | {: >4} MiB | DMA heap pool",
        DMA_HEAP_START,
        DMA_HEAP_END,
        (DMA_HEAP_END - DMA_HEAP_START + 1) >> MIB_RSHIFT
    );

    println!(
        "      {:#010X} - {:#010X} | {: >4} MiB | Device MMIO",
        MMIO_BASE,
        PHYS_ADDR_MAX,
        (PHYS_ADDR_MAX - MMIO_BASE + 1) >> MIB_RSHIFT
    );
}
