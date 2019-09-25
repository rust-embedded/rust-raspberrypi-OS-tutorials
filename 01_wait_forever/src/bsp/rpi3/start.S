// SPDX-License-Identifier: MIT
//
// Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>

.section ".text._start"

.global _start

_start:
1:  wfe         // Wait for event
    b       1b  // In case an event happend, jump back to 1
