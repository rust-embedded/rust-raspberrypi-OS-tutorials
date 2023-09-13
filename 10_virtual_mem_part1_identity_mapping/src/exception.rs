// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020-2023 Andre Richter <andre.o.richter@gmail.com>

//! Synchronous and asynchronous exception handling.

#[cfg(target_arch = "aarch64")]
#[path = "_arch/aarch64/exception.rs"]
mod arch_exception;

pub mod asynchronous;

//--------------------------------------------------------------------------------------------------
// Architectural Public Reexports
//--------------------------------------------------------------------------------------------------
pub use arch_exception::current_privilege_level;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Kernel privilege levels.
#[allow(missing_docs)]
#[derive(Eq, PartialEq)]
pub enum PrivilegeLevel {
    User,
    Kernel,
    Hypervisor,
    Unknown,
}
