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

use core::ops::RangeInclusive;

pub mod mmu;

/// System memory map.
#[rustfmt::skip]
pub mod map {
    pub const START:                   usize =             0x0000_0000;
    pub const END:                     usize =             0x3FFF_FFFF;

    pub mod physical {
        pub const MMIO_BASE:           usize =             0x3F00_0000;
        pub const VIDEOCORE_MBOX_BASE: usize = MMIO_BASE + 0x0000_B880;
        pub const GPIO_BASE:           usize = MMIO_BASE + 0x0020_0000;
        pub const UART_BASE:           usize = MMIO_BASE + 0x0020_1000;
        pub const MMIO_END:            usize =             super::END;
    }

    pub mod virt {
        pub const KERN_STACK_START:    usize =             super::START;
        pub const KERN_STACK_END:      usize =             0x0007_FFFF;

        // The last 4 KiB slot in the first 2 MiB
        pub const REMAPPED_UART_BASE:  usize =             0x001F_F000;
        pub const REMAPPED_UART_END:   usize =             0x001F_FFFF;
    }
}

/// Types used for compiling the virtual memory layout of the kernel using
/// address ranges.
pub mod kernel_mem_range {
    use core::ops::RangeInclusive;

    #[derive(Copy, Clone)]
    pub enum MemAttributes {
        CacheableDRAM,
        Device,
    }

    #[derive(Copy, Clone)]
    pub enum AccessPermissions {
        ReadOnly,
        ReadWrite,
    }

    #[derive(Copy, Clone)]
    pub enum Translation {
        Identity,
        Offset(usize),
    }

    #[derive(Copy, Clone)]
    pub struct AttributeFields {
        pub mem_attributes: MemAttributes,
        pub acc_perms: AccessPermissions,
        pub execute_never: bool,
    }

    impl Default for AttributeFields {
        fn default() -> AttributeFields {
            AttributeFields {
                mem_attributes: MemAttributes::CacheableDRAM,
                acc_perms: AccessPermissions::ReadWrite,
                execute_never: true,
            }
        }
    }

    pub struct Descriptor {
        pub virtual_range: fn() -> RangeInclusive<usize>,
        pub translation: Translation,
        pub attribute_fields: AttributeFields,
    }
}

use kernel_mem_range::*;

/// A virtual memory layout that is agnostic of the paging granularity that the
/// hardware MMU will use.
///
/// Contains only special ranges, aka anything that is _not_ normal cacheable
/// DRAM.
static KERNEL_VIRTUAL_LAYOUT: [Descriptor; 5] = [
    // Kernel stack
    Descriptor {
        virtual_range: || {
            RangeInclusive::new(map::virt::KERN_STACK_START, map::virt::KERN_STACK_END)
        },
        translation: Translation::Identity,
        attribute_fields: AttributeFields {
            mem_attributes: MemAttributes::CacheableDRAM,
            acc_perms: AccessPermissions::ReadWrite,
            execute_never: true,
        },
    },
    // Kernel code and RO data
    Descriptor {
        virtual_range: || {
            // Using the linker script, we ensure that the RO area is consecutive and 4
            // KiB aligned, and we export the boundaries via symbols:
            //
            // [__ro_start, __ro_end)
            extern "C" {
                // The inclusive start of the read-only area, aka the address of the
                // first byte of the area.
                static __ro_start: u64;

                // The exclusive end of the read-only area, aka the address of
                // the first byte _after_ the RO area.
                static __ro_end: u64;
            }

            unsafe {
                // Notice the subtraction to turn the exclusive end into an
                // inclusive end
                RangeInclusive::new(
                    &__ro_start as *const _ as usize,
                    &__ro_end as *const _ as usize - 1,
                )
            }
        },
        translation: Translation::Identity,
        attribute_fields: AttributeFields {
            mem_attributes: MemAttributes::CacheableDRAM,
            acc_perms: AccessPermissions::ReadOnly,
            execute_never: false,
        },
    },
    // Kernel data and BSS
    Descriptor {
        virtual_range: || {
            extern "C" {
                static __ro_end: u64;
                static __bss_end: u64;
            }

            unsafe {
                RangeInclusive::new(
                    &__ro_end as *const _ as usize,
                    &__bss_end as *const _ as usize - 1,
                )
            }
        },
        translation: Translation::Identity,
        attribute_fields: AttributeFields {
            mem_attributes: MemAttributes::CacheableDRAM,
            acc_perms: AccessPermissions::ReadWrite,
            execute_never: true,
        },
    },
    // Remapped UART
    Descriptor {
        virtual_range: || {
            RangeInclusive::new(map::virt::REMAPPED_UART_BASE, map::virt::REMAPPED_UART_END)
        },
        translation: Translation::Offset(map::physical::UART_BASE),
        attribute_fields: AttributeFields {
            mem_attributes: MemAttributes::Device,
            acc_perms: AccessPermissions::ReadWrite,
            execute_never: true,
        },
    },
    // Device MMIO
    Descriptor {
        virtual_range: || RangeInclusive::new(map::physical::MMIO_BASE, map::physical::MMIO_END),
        translation: Translation::Identity,
        attribute_fields: AttributeFields {
            mem_attributes: MemAttributes::Device,
            acc_perms: AccessPermissions::ReadWrite,
            execute_never: true,
        },
    },
];

/// For a given virtual address, find and return the output address and
/// according attributes.
///
/// If the address is not covered in VIRTUAL_LAYOUT, return a default for normal
/// cacheable DRAM.
fn get_virt_addr_properties(virt_addr: usize) -> Result<(usize, AttributeFields), &'static str> {
    if virt_addr > map::END {
        return Err("Address out of range.");
    }

    for i in KERNEL_VIRTUAL_LAYOUT.iter() {
        if (i.virtual_range)().contains(&virt_addr) {
            let output_addr = match i.translation {
                Translation::Identity => virt_addr,
                Translation::Offset(a) => a + (virt_addr - (i.virtual_range)().start()),
            };

            return Ok((output_addr, i.attribute_fields));
        }
    }

    Ok((virt_addr, AttributeFields::default()))
}
