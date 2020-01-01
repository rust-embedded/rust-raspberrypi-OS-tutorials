// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2019-2020 Andre Richter <andre.o.richter@gmail.com>

//! Fix for register-rs.
//!
//! Used temporarily until https://github.com/tock/tock/issues/1482 is resolved.

#![no_std]

/// A temporary overwrite for tock's register_structs! so that it does not emit `#[test]` attributes.
#[macro_export]
macro_rules! register_structs {
    {
        $(
            $(#[$attr:meta])*
            $name:ident {
                $( $fields:tt )*
            }
        ),*
    } => {
        $( register_fields!(@root $(#[$attr])* $name { $($fields)* } ); )*
    };
}
