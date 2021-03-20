// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2021 Andre Richter <andre.o.richter@gmail.com>

//--------------------------------------------------------------------------------------------------
// Definitions
//--------------------------------------------------------------------------------------------------

.equ _core_id_mask, 0b11

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------
.section .text._start

//------------------------------------------------------------------------------
// fn _start()
//------------------------------------------------------------------------------
_start:
	// Only proceed on the boot core. Park it otherwise.
	mrs	x1, MPIDR_EL1
	and	x1, x1, _core_id_mask
	ldr	x2, BOOT_CORE_ID      // provided by bsp/__board_name__/cpu.rs
	cmp	x1, x2
	b.ne	2f

	// If execution reaches here, it is the boot core.

	// Next, relocate the binary.
	adr	x0, __binary_nonzero_start          // The address the binary got loaded to.
	ldr	x1, =__binary_nonzero_start         // The address the binary was linked to.
	ldr	x2, =__binary_nonzero_end_exclusive

1:	ldr	x3, [x0], #8
	str	x3, [x1], #8
	cmp	x1, x2
	b.lo	1b

	// Set the stack pointer.
	ldr	x0, =__boot_core_stack_end_exclusive
	mov	sp, x0

	// Jump to the relocated Rust code.
	ldr	x1, =_start_rust
	br	x1

	// Infinitely wait for events (aka "park the core").
2:	wfe
	b	2b

.size	_start, . - _start
.type	_start, function
.global	_start
