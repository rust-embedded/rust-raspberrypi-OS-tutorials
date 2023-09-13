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
pub use arch_exception::{current_privilege_level, handling_init};

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

//--------------------------------------------------------------------------------------------------
// Testing
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use test_macros::kernel_test;

    /// Libkernel unit tests must execute in kernel mode.
    #[kernel_test]
    fn test_runner_executes_in_kernel_mode() {
        let (level, _) = current_privilege_level();

        assert!(level == PrivilegeLevel::Kernel)
    }
}
