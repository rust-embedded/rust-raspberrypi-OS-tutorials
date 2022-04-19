// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2019-2022 Andre Richter <andre.o.richter@gmail.com>

//! Types for the `custom_test_frameworks` implementation.

#![no_std]

/// Unit test container.
pub struct UnitTest {
    /// Name of the test.
    pub name: &'static str,

    /// Function pointer to the test.
    pub test_func: fn(),
}
