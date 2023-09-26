// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020-2023 Andre Richter <andre.o.richter@gmail.com>

//! A record of mapped pages.

use super::{
    AccessPermissions, Address, AttributeFields, MMIODescriptor, MemAttributes, MemoryRegion,
    Physical, Virtual,
};
use crate::{bsp, common, info, synchronization, synchronization::InitStateLock};
use alloc::{vec, vec::Vec};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

/// Type describing a virtual memory mapping.
#[allow(missing_docs)]
struct MappingRecordEntry {
    pub users: Vec<&'static str>,
    pub phys_start_addr: Address<Physical>,
    pub virt_start_addr: Address<Virtual>,
    pub num_pages: usize,
    pub attribute_fields: AttributeFields,
}

struct MappingRecord {
    inner: Vec<MappingRecordEntry>,
}

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

static KERNEL_MAPPING_RECORD: InitStateLock<MappingRecord> =
    InitStateLock::new(MappingRecord::new());

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

impl MappingRecordEntry {
    pub fn new(
        name: &'static str,
        virt_region: &MemoryRegion<Virtual>,
        phys_region: &MemoryRegion<Physical>,
        attr: &AttributeFields,
    ) -> Self {
        Self {
            users: vec![name],
            phys_start_addr: phys_region.start_addr(),
            virt_start_addr: virt_region.start_addr(),
            num_pages: phys_region.num_pages(),
            attribute_fields: *attr,
        }
    }

    pub fn add_user(&mut self, user: &'static str) {
        self.users.push(user);
    }
}

impl MappingRecord {
    pub const fn new() -> Self {
        Self { inner: Vec::new() }
    }

    fn sort(&mut self) {
        if !self.inner.is_sorted_by_key(|item| item.virt_start_addr) {
            self.inner.sort_unstable_by_key(|item| item.virt_start_addr)
        }
    }

    fn find_duplicate(
        &mut self,
        phys_region: &MemoryRegion<Physical>,
    ) -> Option<&mut MappingRecordEntry> {
        self.inner
            .iter_mut()
            .filter(|x| x.attribute_fields.mem_attributes == MemAttributes::Device)
            .find(|x| {
                if x.phys_start_addr != phys_region.start_addr() {
                    return false;
                }

                if x.num_pages != phys_region.num_pages() {
                    return false;
                }

                true
            })
    }

    pub fn add(
        &mut self,
        name: &'static str,
        virt_region: &MemoryRegion<Virtual>,
        phys_region: &MemoryRegion<Physical>,
        attr: &AttributeFields,
    ) {
        self.inner.push(MappingRecordEntry::new(
            name,
            virt_region,
            phys_region,
            attr,
        ));

        self.sort();
    }

    pub fn print(&self) {
        info!("      -------------------------------------------------------------------------------------------------------------------------------------------");
        info!(
            "      {:^44}     {:^30}   {:^7}   {:^9}   {:^35}",
            "Virtual", "Physical", "Size", "Attr", "Entity"
        );
        info!("      -------------------------------------------------------------------------------------------------------------------------------------------");

        for i in self.inner.iter() {
            let size = i.num_pages * bsp::memory::mmu::KernelGranule::SIZE;
            let virt_start = i.virt_start_addr;
            let virt_end_inclusive = virt_start + (size - 1);
            let phys_start = i.phys_start_addr;
            let phys_end_inclusive = phys_start + (size - 1);

            let (size, unit) = common::size_human_readable_ceil(size);

            let attr = match i.attribute_fields.mem_attributes {
                MemAttributes::CacheableDRAM => "C",
                MemAttributes::Device => "Dev",
            };

            let acc_p = match i.attribute_fields.acc_perms {
                AccessPermissions::ReadOnly => "RO",
                AccessPermissions::ReadWrite => "RW",
            };

            let xn = if i.attribute_fields.execute_never {
                "XN"
            } else {
                "X"
            };

            info!(
                "      {}..{} --> {}..{} | {:>3} {} | {:<3} {} {:<2} | {}",
                virt_start,
                virt_end_inclusive,
                phys_start,
                phys_end_inclusive,
                size,
                unit,
                attr,
                acc_p,
                xn,
                i.users[0]
            );

            for k in &i.users[1..] {
                info!(
                        "                                                                                                            | {}",
                        k
                    );
            }
        }

        info!("      -------------------------------------------------------------------------------------------------------------------------------------------");
    }
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------
use synchronization::interface::ReadWriteEx;

/// Add an entry to the mapping info record.
pub fn kernel_add(
    name: &'static str,
    virt_region: &MemoryRegion<Virtual>,
    phys_region: &MemoryRegion<Physical>,
    attr: &AttributeFields,
) {
    KERNEL_MAPPING_RECORD.write(|mr| mr.add(name, virt_region, phys_region, attr))
}

pub fn kernel_find_and_insert_mmio_duplicate(
    mmio_descriptor: &MMIODescriptor,
    new_user: &'static str,
) -> Option<Address<Virtual>> {
    let phys_region: MemoryRegion<Physical> = (*mmio_descriptor).into();

    KERNEL_MAPPING_RECORD.write(|mr| {
        let dup = mr.find_duplicate(&phys_region)?;

        dup.add_user(new_user);

        Some(dup.virt_start_addr)
    })
}

/// Human-readable print of all recorded kernel mappings.
pub fn kernel_print() {
    KERNEL_MAPPING_RECORD.read(|mr| mr.print());
}
