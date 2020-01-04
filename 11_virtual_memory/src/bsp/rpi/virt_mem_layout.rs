// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! The virtual memory layout.
//!
//! The layout must contain only special ranges, aka anything that is _not_ normal cacheable DRAM.
//! It is agnostic of the paging granularity that the architecture's MMU will use.

use super::memory_map;
use crate::memory::*;
use core::ops::RangeInclusive;

//--------------------------------------------------------------------------------------------------
// BSP-public
//--------------------------------------------------------------------------------------------------

pub const NUM_MEM_RANGES: usize = 3;

pub static LAYOUT: KernelVirtualLayout<{ NUM_MEM_RANGES }> = KernelVirtualLayout::new(
    memory_map::END_INCLUSIVE,
    [
        RangeDescriptor {
            name: "Kernel code and RO data",
            virtual_range: || {
                // Using the linker script, we ensure that the RO area is consecutive and 64 KiB
                // aligned, and we export the boundaries via symbols:
                //
                // [__ro_start, __ro_end)
                extern "C" {
                    // The inclusive start of the read-only area, aka the address of the first
                    // byte of the area.
                    static __ro_start: usize;

                    // The exclusive end of the read-only area, aka the address of the first
                    // byte _after_ the RO area.
                    static __ro_end: usize;
                }

                unsafe {
                    // Notice the subtraction to turn the exclusive end into an inclusive end.
                    #[allow(clippy::range_minus_one)]
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
        RangeDescriptor {
            name: "Remapped Device MMIO",
            virtual_range: || {
                // The last 64 KiB slot in the first 512 MiB
                RangeInclusive::new(0x1FFF_0000, 0x1FFF_FFFF)
            },
            translation: Translation::Offset(memory_map::mmio::BASE + 0x20_0000),
            attribute_fields: AttributeFields {
                mem_attributes: MemAttributes::Device,
                acc_perms: AccessPermissions::ReadWrite,
                execute_never: true,
            },
        },
        RangeDescriptor {
            name: "Device MMIO",
            virtual_range: || {
                RangeInclusive::new(memory_map::mmio::BASE, memory_map::mmio::END_INCLUSIVE)
            },
            translation: Translation::Identity,
            attribute_fields: AttributeFields {
                mem_attributes: MemAttributes::Device,
                acc_perms: AccessPermissions::ReadWrite,
                execute_never: true,
            },
        },
    ],
);
