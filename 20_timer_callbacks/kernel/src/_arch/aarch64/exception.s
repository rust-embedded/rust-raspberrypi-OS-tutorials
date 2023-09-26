// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>

//--------------------------------------------------------------------------------------------------
// Definitions
//--------------------------------------------------------------------------------------------------

/// Call the function provided by parameter `\handler` after saving the exception context. Provide
/// the context as the first parameter to '\handler'.
.macro CALL_WITH_CONTEXT handler is_lower_el is_sync
__vector_\handler:
	// Make room on the stack for the exception context.
	sub	sp,  sp,  #16 * 18

	// Store all general purpose registers on the stack.
	stp	x0,  x1,  [sp, #16 * 0]
	stp	x2,  x3,  [sp, #16 * 1]
	stp	x4,  x5,  [sp, #16 * 2]
	stp	x6,  x7,  [sp, #16 * 3]
	stp	x8,  x9,  [sp, #16 * 4]
	stp	x10, x11, [sp, #16 * 5]
	stp	x12, x13, [sp, #16 * 6]
	stp	x14, x15, [sp, #16 * 7]
	stp	x16, x17, [sp, #16 * 8]
	stp	x18, x19, [sp, #16 * 9]
	stp	x20, x21, [sp, #16 * 10]
	stp	x22, x23, [sp, #16 * 11]
	stp	x24, x25, [sp, #16 * 12]
	stp	x26, x27, [sp, #16 * 13]
	stp	x28, x29, [sp, #16 * 14]

	// Add the exception link register (ELR_EL1), saved program status (SPSR_EL1) and exception
	// syndrome register (ESR_EL1).
	mrs	x1,  ELR_EL1
	mrs	x2,  SPSR_EL1
	mrs	x3,  ESR_EL1

	stp	lr,  x1,  [sp, #16 * 15]
	stp	x2,  x3,  [sp, #16 * 16]

	// Build a stack frame for backtracing.
.if \is_lower_el == 1
	// If we came from a lower EL, make it a root frame (by storing zero) so that the kernel
	// does not attempt to trace into userspace.
	stp	xzr, xzr, [sp, #16 * 17]
.else
	// For normal branches, the link address points to the instruction to be executed _after_
	// returning from a branch. In a backtrace, we want to show the instruction that caused the
	// branch, though. That is why code in backtrace.rs subtracts 4 (length of one instruction)
	// from the link address.
	//
	// Here we have a special case, though, because ELR_EL1 is used instead of LR to build the
	// stack frame, so that it becomes possible to trace beyond an exception. Hence, it must be
	// considered that semantics for ELR_EL1 differ from case to case.
	//
	// Unless an "exception generating instruction" was executed, ELR_EL1 already points to the
	// the correct instruction, and hence the subtraction by 4 in backtrace.rs would yield wrong
	// results. To cover for this, 4 is added to ELR_EL1 below unless the cause of exception was
	// an SVC instruction. BRK and HLT are "exception generating instructions" as well, but they
	// are not expected and therefore left out for now.
	//
	// For reference: Search for "preferred exception return address" in the Architecture
	// Reference Manual for ARMv8-A.
.if \is_sync == 1
	lsr	w3,  w3, {CONST_ESR_EL1_EC_SHIFT}   // w3 = ESR_EL1.EC
	cmp	w3,  {CONST_ESR_EL1_EC_VALUE_SVC64} // w3 == SVC64 ?
	b.eq	1f
.endif
	add	x1,  x1, #4
1:
	stp	x29, x1, [sp, #16 * 17]
.endif

	// Set the frame pointer to the stack frame record.
	add	x29, sp, #16 * 17

	// x0 is the first argument for the function called through `\handler`.
	mov	x0,  sp

	// Call `\handler`.
	bl	\handler

	// After returning from exception handling code, replay the saved context and return via
	// `eret`.
	b	__exception_restore_context

.size	__vector_\handler, . - __vector_\handler
.type	__vector_\handler, function
.endm

.macro FIQ_SUSPEND
1:	wfe
	b	1b
.endm

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------
.section .text

//------------------------------------------------------------------------------
// The exception vector table.
//------------------------------------------------------------------------------

// Align by 2^11 bytes, as demanded by ARMv8-A. Same as ALIGN(2048) in an ld script.
.align 11

// Export a symbol for the Rust code to use.
__exception_vector_start:

// Current exception level with SP_EL0.
//
// .org sets the offset relative to section start.
//
// # Safety
//
// - It must be ensured that `CALL_WITH_CONTEXT` <= 0x80 bytes.
.org 0x000
	CALL_WITH_CONTEXT current_el0_synchronous, 0, 1
.org 0x080
	CALL_WITH_CONTEXT current_el0_irq, 0, 0
.org 0x100
	FIQ_SUSPEND
.org 0x180
	CALL_WITH_CONTEXT current_el0_serror, 0, 0

// Current exception level with SP_ELx, x > 0.
.org 0x200
	CALL_WITH_CONTEXT current_elx_synchronous, 0, 1
.org 0x280
	CALL_WITH_CONTEXT current_elx_irq, 0, 0
.org 0x300
	FIQ_SUSPEND
.org 0x380
	CALL_WITH_CONTEXT current_elx_serror, 0, 0

// Lower exception level, AArch64
.org 0x400
	CALL_WITH_CONTEXT lower_aarch64_synchronous, 1, 1
.org 0x480
	CALL_WITH_CONTEXT lower_aarch64_irq, 1, 0
.org 0x500
	FIQ_SUSPEND
.org 0x580
	CALL_WITH_CONTEXT lower_aarch64_serror, 1, 0

// Lower exception level, AArch32
.org 0x600
	CALL_WITH_CONTEXT lower_aarch32_synchronous, 1, 0
.org 0x680
	CALL_WITH_CONTEXT lower_aarch32_irq, 1, 0
.org 0x700
	FIQ_SUSPEND
.org 0x780
	CALL_WITH_CONTEXT lower_aarch32_serror, 1, 0
.org 0x800

//------------------------------------------------------------------------------
// fn __exception_restore_context()
//------------------------------------------------------------------------------
__exception_restore_context:
	ldr	w19,      [sp, #16 * 16]
	ldp	lr,  x20, [sp, #16 * 15]

	msr	SPSR_EL1, x19
	msr	ELR_EL1,  x20

	ldp	x0,  x1,  [sp, #16 * 0]
	ldp	x2,  x3,  [sp, #16 * 1]
	ldp	x4,  x5,  [sp, #16 * 2]
	ldp	x6,  x7,  [sp, #16 * 3]
	ldp	x8,  x9,  [sp, #16 * 4]
	ldp	x10, x11, [sp, #16 * 5]
	ldp	x12, x13, [sp, #16 * 6]
	ldp	x14, x15, [sp, #16 * 7]
	ldp	x16, x17, [sp, #16 * 8]
	ldp	x18, x19, [sp, #16 * 9]
	ldp	x20, x21, [sp, #16 * 10]
	ldp	x22, x23, [sp, #16 * 11]
	ldp	x24, x25, [sp, #16 * 12]
	ldp	x26, x27, [sp, #16 * 13]
	ldp	x28, x29, [sp, #16 * 14]

	add	sp,  sp,  #16 * 18

	eret

.size	__exception_restore_context, . - __exception_restore_context
.type	__exception_restore_context, function
