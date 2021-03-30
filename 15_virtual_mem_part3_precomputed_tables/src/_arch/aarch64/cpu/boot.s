// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2021 Andre Richter <andre.o.richter@gmail.com>

//--------------------------------------------------------------------------------------------------
// Definitions
//--------------------------------------------------------------------------------------------------

// Load the address of a symbol into a register, PC-relative.
//
// The symbol must lie within +/- 4 GiB of the Program Counter.
//
// # Resources
//
// - https://sourceware.org/binutils/docs-2.36/as/AArch64_002dRelocations.html
.macro ADR_REL register, symbol
	adrp	\register, \symbol
	add	\register, \register, #:lo12:\symbol
.endm

.equ _EL2, 0x8
.equ _core_id_mask, 0b11

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------
.section .text._start

//------------------------------------------------------------------------------
// fn _start()
//------------------------------------------------------------------------------
_start:
	// Only proceed if the core executes in EL2. Park it otherwise.
	mrs	x0, CurrentEL
	cmp	x0, _EL2
	b.ne	1f

	// Only proceed on the boot core. Park it otherwise.
	mrs	x1, MPIDR_EL1
	and	x1, x1, _core_id_mask
	ldr	x2, BOOT_CORE_ID      // provided by bsp/__board_name__/cpu.rs
	cmp	x1, x2
	b.ne	1f

	// If execution reaches here, it is the boot core. Now, prepare the jump to Rust code.

	// Load the base address of the kernel's translation tables.
	ldr	x0, PHYS_KERNEL_TABLES_BASE_ADDR // provided by bsp/__board_name__/memory/mmu.rs

	// Set the stack pointer. This ensures that any code in EL2 that needs the stack will work.
	ADR_REL	x1, __boot_core_stack_end_exclusive
	mov	sp, x1

	// Jump to Rust code. x0 and x1 hold the function arguments provided to _start_rust().
	b	_start_rust

	// Infinitely wait for events (aka "park the core").
1:	wfe
	b	1b

.size	_start, . - _start
.type	_start, function
.global	_start
