// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2022-2023 Andre Richter <andre.o.richter@gmail.com>

//! Architectural backtracing support.
//!
//! # Orientation
//!
//! Since arch modules are imported into generic modules using the path attribute, the path of this
//! file is:
//!
//! crate::backtrace::arch_backtrace

use crate::{
    backtrace::BacktraceItem,
    memory::{Address, Virtual},
};
use aarch64_cpu::registers::*;
use tock_registers::interfaces::Readable;

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

/// A Stack frame record.
///
/// # Note
///
/// The convention is that `previous_record` is valid as long as it contains a non-null value.
/// Therefore, it is possible to type the member as `Option<&StackFrameRecord>` because of Rust's
/// `null-pointer optimization`.
#[repr(C)]
struct StackFrameRecord<'a> {
    previous_record: Option<&'a StackFrameRecord<'a>>,
    link: Address<Virtual>,
}

struct StackFrameRecordIterator<'a> {
    cur: &'a StackFrameRecord<'a>,
}

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

impl<'a> Iterator for StackFrameRecordIterator<'a> {
    type Item = BacktraceItem;

    fn next(&mut self) -> Option<Self::Item> {
        static ABORT_FRAME: StackFrameRecord = StackFrameRecord {
            previous_record: None,
            link: Address::new(0),
        };

        // If previous is None, this is the root frame, so iteration will stop here.
        let previous = self.cur.previous_record?;

        // Need to abort if the pointer to the previous frame record is invalid.
        let prev_addr = Address::<Virtual>::new(previous as *const _ as usize);
        if !prev_addr.is_valid_stack_addr() {
            // This allows to return the error and then stop on the next iteration.
            self.cur = &ABORT_FRAME;
            return Some(BacktraceItem::InvalidFramePointer(prev_addr));
        }

        let ret = if !self.cur.link.is_valid_code_addr() {
            Some(BacktraceItem::InvalidLink(self.cur.link))
        } else {
            // The link points to the instruction to be executed _after_ returning from a branch.
            // However, we want to show the instruction that caused the branch, so subtract by one
            // instruction.
            //
            // This might be called from panic!, so it must not panic itself on the subtraction.
            let link = if self.cur.link >= Address::new(4) {
                self.cur.link - 4
            } else {
                self.cur.link
            };

            Some(BacktraceItem::Link(link))
        };

        // Advance the iterator.
        self.cur = previous;

        ret
    }
}

fn stack_frame_record_iterator<'a>() -> Option<StackFrameRecordIterator<'a>> {
    let fp = Address::<Virtual>::new(FP.get() as usize);
    if !fp.is_valid_stack_addr() {
        return None;
    }

    Some(StackFrameRecordIterator {
        cur: unsafe { &*(fp.as_usize() as *const _) },
    })
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Architectural implementation of the backtrace.
pub fn backtrace(f: impl FnOnce(Option<&mut dyn Iterator<Item = BacktraceItem>>)) {
    f(stack_frame_record_iterator().as_mut().map(|s| s as _))
}

//--------------------------------------------------------------------------------------------------
// Testing
//--------------------------------------------------------------------------------------------------

#[cfg(feature = "test_build")]
#[inline(always)]
/// Hack for corrupting the previous frame address in the current stack frame.
///
/// # Safety
///
/// - To be used only by testing code.
pub unsafe fn corrupt_previous_frame_addr() {
    let sf = FP.get() as *mut usize;
    *sf = 0x123;
}

#[cfg(feature = "test_build")]
#[inline(always)]
/// Hack for corrupting the link in the current stack frame.
///
/// # Safety
///
/// - To be used only by testing code.
pub unsafe fn corrupt_link() {
    let sf = FP.get() as *mut StackFrameRecord;
    (*sf).link = Address::new(0x456);
}
