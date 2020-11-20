// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020 Andre Richter <andre.o.richter@gmail.com>

//! A record of mapped pages.

use super::{
    AccessPermissions, Address, AttributeFields, MMIODescriptor, MemAttributes,
    PageSliceDescriptor, Physical, Virtual,
};
use crate::{info, synchronization, synchronization::InitStateLock, warn};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

/// Type describing a virtual memory mapping.
#[allow(missing_docs)]
#[derive(Copy, Clone)]
struct MappingRecordEntry {
    pub users: [Option<&'static str>; 5],
    pub phys_pages: PageSliceDescriptor<Physical>,
    pub virt_start_addr: Address<Virtual>,
    pub attribute_fields: AttributeFields,
}

struct MappingRecord {
    inner: [Option<MappingRecordEntry>; 12],
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
        phys_pages: &PageSliceDescriptor<Physical>,
        virt_pages: &PageSliceDescriptor<Virtual>,
        attr: &AttributeFields,
    ) -> Self {
        Self {
            users: [Some(name), None, None, None, None],
            phys_pages: *phys_pages,
            virt_start_addr: virt_pages.start_addr(),
            attribute_fields: *attr,
        }
    }

    fn find_next_free_user(&mut self) -> Result<&mut Option<&'static str>, &'static str> {
        if let Some(x) = self.users.iter_mut().find(|x| x.is_none()) {
            return Ok(x);
        };

        Err("Storage for user info exhausted")
    }

    pub fn add_user(&mut self, user: &'static str) -> Result<(), &'static str> {
        let x = self.find_next_free_user()?;
        *x = Some(user);
        Ok(())
    }
}

impl MappingRecord {
    pub const fn new() -> Self {
        Self { inner: [None; 12] }
    }

    fn find_next_free(&mut self) -> Result<&mut Option<MappingRecordEntry>, &'static str> {
        if let Some(x) = self.inner.iter_mut().find(|x| x.is_none()) {
            return Ok(x);
        }

        Err("Storage for mapping info exhausted")
    }

    fn find_duplicate(
        &mut self,
        phys_pages: &PageSliceDescriptor<Physical>,
    ) -> Option<&mut MappingRecordEntry> {
        self.inner
            .iter_mut()
            .filter(|x| x.is_some())
            .map(|x| x.as_mut().unwrap())
            .filter(|x| x.attribute_fields.mem_attributes == MemAttributes::Device)
            .find(|x| x.phys_pages == *phys_pages)
    }

    pub fn add(
        &mut self,
        name: &'static str,
        phys_pages: &PageSliceDescriptor<Physical>,
        virt_pages: &PageSliceDescriptor<Virtual>,
        attr: &AttributeFields,
    ) -> Result<(), &'static str> {
        let x = self.find_next_free()?;

        *x = Some(MappingRecordEntry::new(name, phys_pages, virt_pages, attr));
        Ok(())
    }

    pub fn print(&self) {
        const KIB_RSHIFT: u32 = 10; // log2(1024).
        const MIB_RSHIFT: u32 = 20; // log2(1024 * 1024).

        info!("      -----------------------------------------------------------------------------------------------------------------");
        info!(
            "      {:^24}     {:^24}   {:^7}   {:^9}   {:^35}",
            "Virtual", "Physical", "Size", "Attr", "Entity"
        );
        info!("      -----------------------------------------------------------------------------------------------------------------");

        for i in self
            .inner
            .iter()
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
        {
            let virt_start = i.virt_start_addr.into_usize();
            let virt_end_inclusive = virt_start + i.phys_pages.size() - 1;
            let phys_start = i.phys_pages.start_addr().into_usize();
            let phys_end_inclusive = i.phys_pages.end_addr_inclusive().into_usize();
            let size = i.phys_pages.size();

            let (size, unit) = if (size >> MIB_RSHIFT) > 0 {
                (size >> MIB_RSHIFT, "MiB")
            } else if (size >> KIB_RSHIFT) > 0 {
                (size >> KIB_RSHIFT, "KiB")
            } else {
                (size, "Byte")
            };

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
                "      {:#011X}..{:#011X} --> {:#011X}..{:#011X} | \
                        {: >3} {} | {: <3} {} {: <2} | {}",
                virt_start,
                virt_end_inclusive,
                phys_start,
                phys_end_inclusive,
                size,
                unit,
                attr,
                acc_p,
                xn,
                i.users[0].unwrap()
            );

            for k in i.users[1..].iter() {
                if let Some(additional_user) = *k {
                    info!(
                        "                                                                                  | {}",
                        additional_user
                    );
                }
            }
        }

        info!("      -----------------------------------------------------------------------------------------------------------------");
    }
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------
use synchronization::interface::ReadWriteEx;

/// Add an entry to the mapping info record.
pub fn kernel_add(
    name: &'static str,
    phys_pages: &PageSliceDescriptor<Physical>,
    virt_pages: &PageSliceDescriptor<Virtual>,
    attr: &AttributeFields,
) -> Result<(), &'static str> {
    KERNEL_MAPPING_RECORD.write(|mr| mr.add(name, phys_pages, virt_pages, attr))
}

pub fn kernel_find_and_insert_mmio_duplicate(
    phys_mmio_descriptor: &MMIODescriptor<Physical>,
    new_user: &'static str,
) -> Option<Address<Virtual>> {
    let phys_pages: PageSliceDescriptor<Physical> = phys_mmio_descriptor.clone().into();

    KERNEL_MAPPING_RECORD.write(|mr| {
        let dup = mr.find_duplicate(&phys_pages)?;

        if let Err(x) = dup.add_user(new_user) {
            warn!("{}", x);
        }

        Some(dup.virt_start_addr)
    })
}

/// Human-readable print of all recorded kernel mappings.
pub fn kernel_print() {
    KERNEL_MAPPING_RECORD.read(|mr| mr.print());
}
