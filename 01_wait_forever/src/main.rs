// SPDX-License-Identifier: MIT
//
// Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>

//! The `kernel`

#![feature(asm)]
#![feature(global_asm)]
#![no_main]
#![no_std]

// This module conditionally includes the correct `BSP` which provides the
// `_start()` function, the first function to run.
mod bsp;

// Kernel code coming next tutorial.
