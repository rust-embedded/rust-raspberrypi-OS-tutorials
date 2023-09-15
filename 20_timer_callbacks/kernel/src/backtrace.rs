// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2022-2023 Andre Richter <andre.o.richter@gmail.com>

//! Backtracing support.

#[cfg(target_arch = "aarch64")]
#[path = "_arch/aarch64/backtrace.rs"]
mod arch_backtrace;

use crate::{
    memory::{Address, Virtual},
    symbols,
};
use core::fmt;

//--------------------------------------------------------------------------------------------------
// Architectural Public Reexports
//--------------------------------------------------------------------------------------------------
#[cfg(feature = "test_build")]
pub use arch_backtrace::{corrupt_link, corrupt_previous_frame_addr};

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// A backtrace item.
#[allow(missing_docs)]
pub enum BacktraceItem {
    InvalidFramePointer(Address<Virtual>),
    InvalidLink(Address<Virtual>),
    Link(Address<Virtual>),
}

/// Pseudo-struct for printing a backtrace using its fmt::Display implementation.
pub struct Backtrace;

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

impl fmt::Display for Backtrace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Backtrace:")?;
        writeln!(
            f,
            "      ----------------------------------------------------------------------------------------------"
        )?;
        writeln!(
            f,
            "          Address            Function containing address"
        )?;
        writeln!(
            f,
            "      ----------------------------------------------------------------------------------------------"
        )?;

        let mut fmt_res: fmt::Result = Ok(());
        let trace_formatter =
            |maybe_iter: Option<&mut dyn Iterator<Item = BacktraceItem>>| match maybe_iter {
                None => fmt_res = writeln!(f, "ERROR! No valid stack frame found"),
                Some(iter) => {
                    // Since the backtrace is printed, the first function is always
                    // core::fmt::write. Skip 1 so it is excluded and doesn't bloat the output.
                    for (i, backtrace_res) in iter.skip(1).enumerate() {
                        match backtrace_res {
                            BacktraceItem::InvalidFramePointer(addr) => {
                                fmt_res = writeln!(
                                    f,
                                    "      {:>2}. ERROR! \
                                    Encountered invalid frame pointer ({}) during backtrace",
                                    i + 1,
                                    addr
                                );
                            }
                            BacktraceItem::InvalidLink(addr) => {
                                fmt_res = writeln!(
                                    f,
                                    "      {:>2}. ERROR! \
                                    Link address ({}) is not contained in kernel .text section",
                                    i + 1,
                                    addr
                                );
                            }
                            BacktraceItem::Link(addr) => {
                                fmt_res = writeln!(
                                    f,
                                    "      {:>2}. {:016x} | {:<50}",
                                    i + 1,
                                    addr.as_usize(),
                                    match symbols::lookup_symbol(addr) {
                                        Some(sym) => sym.name(),
                                        _ => "Symbol not found",
                                    }
                                )
                            }
                        };

                        if fmt_res.is_err() {
                            break;
                        }
                    }
                }
            };

        arch_backtrace::backtrace(trace_formatter);
        fmt_res?;

        writeln!(
            f,
            "      ----------------------------------------------------------------------------------------------"
        )
    }
}
