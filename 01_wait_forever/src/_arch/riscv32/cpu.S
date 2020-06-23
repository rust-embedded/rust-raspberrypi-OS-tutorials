// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020 Ken Kawamoto <kentaro.kawamoto@gmail.com>

.section ".text._start"

.global _start

_start:
1:  wfi         // Wait for interrupt
    j       1b  // In case an event happened, jump back to 1
