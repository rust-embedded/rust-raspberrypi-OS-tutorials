# Tutorial 12 - CPU Exceptions: Part 1

## tl;dr

We lay the groundwork for all the architectural `CPU exceptions`. For now, only print an elaborate
system state through a `panic!` call, and halt execution; This will help finding bugs during
development and runtime.

For demo purposes, MMU `page faults` are used to demonstrate (i) returning from an exception and
(ii) the default `panic!` behavior.

## Table of Contents

- [Introduction](#introduction)
- [Exception Types](#exception-types)
- [Exception entry](#exception-entry)
  * [Exception Vectors](#exception-vectors)
- [Handler Code and Offsets](#handler-code-and-offsets)
- [Rust and Assembly Implementation](#rust-and-assembly-implementation)
  * [Context Save and Restore](#context-save-and-restore)
  * [Exception Vector Table](#exception-vector-table)
  * [Implementing handlers](#implementing-handlers)
- [Causing an Exception - Testing the Code](#causing-an-exception---testing-the-code)
- [Test it](#test-it)
- [Diff to previous](#diff-to-previous)

## Introduction

Now that we are executing in `EL1`, and have activated the `MMU`, time is due for implementing `CPU
exceptions`. For now, we only set up a scaffold with very basic functionality that will help us to
find bugs along the way. A follow-up `Interrupt` tutorial in the future will continue the work we
start here.

Please note that this tutorial is specific to the `AArch64` architecture. It does not contain any
generic exception handling code yet.

## Exception Types

In `AArch64`, it is differentiated between four types of exceptions. These are:
- Synchronous
  - For example, a `data abort` (e.g. `page fault`) or a `system call`. They happen in direct
    consequence of executing a certain instruction, hence _synchronously_.
- Interrupt Request (`IRQ`)
  - For example, an external device, like a timer, is asserting a physical interrupt line. IRQs
    happen _asynchronously_.
- Fast Interrupt Request (`FIQ`)
  - These are basically interrupts that take priority over normal IRQs and have some more traits
    that make them suitable to implement super-fast processing. However, this is out of scope for
    this tutorial. For the sake of keeping these tutorials compact and concise, we will more or less
    ignore FIQs and only implement a dummy handler that would halt the CPU core.
- System Error (`SError`)
  - Like IRQs, SErrors happen asynchronously and are technically more or less the same. They are
    intended to signal rather fatal errors in the system, e.g. if a transaction times out on the
    `SoC` interconnect. They are very implementation specific and it is up to the SoC vendor to
    decide which events are delivered as SErrors instead of normal IRQs.

## Exception entry

I recommend to read pages 1874-1876 of the [ARMv8 Architecture Reference Manual][ARMv8_Manual] to
understand the mechanisms of taking an exception.

Here's an excerpt of important features for this tutorial:
- Exception entry moves the processor to the same or a higher `Exception Level`, but never to a
  lower `EL`.
- The program status is saved in the `SPSR_ELx` register at the target `EL`.
- The preferred return address is saved in the `ELR_ELx` register.
  - "Preferred" here means that `ELR_ELx` may hold the instruction address of the instructions that
    caused the exception (`synchronous case`) or the first instruction that did not complete due to
    an `asynchronous` exception. Details in Chapter D1.10.1 of the [ARMv8 Architecture Reference
    Manual][ARMv8_Manual].
- All kinds of exceptions are turned off upon taking an exception, so that by default, exception
  handlers can not get interrupted themselves.
- Taking an exception will select the dedicated stack pointer of the target `EL`.
  - For example, if an exception in `EL0` is taken, the Stack Pointer Select register `SPSel` will
    switch from `0` to `1`, meaning that `SP_EL1` will be used by the exception vector code unless
    you explicitly change it back to `SP_EL0`.


### Exception Vectors

`AArch64` has a total of `16` exception vectors. There is one for each of the four kinds that were
introduced already, and additionally, it is taken into account _where_ the exception was taken from
and what the circumstances were.

Here is a copy of the decision table as shown in Chapter D1.10.2 of the [ARMv8 Architecture
Reference Manual][ARMv8_Manual]:

[ARMv8_Manual]: https://developer.arm.com/docs/ddi0487/latest/arm-architecture-reference-manual-armv8-for-armv8-a-architecture-profile

<table>
    <thead>
        <tr>
            <th rowspan=2>Exception taken from </th>
            <th colspan=4>Offset for exception type</th>
        </tr>
        <tr>
            <th>Synchronous</th>
            <th>IRQ or vIRQ</th>
            <th>FIQ or vFIQ</th>
            <th>SError or vSError</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td width="40%">Current Exception level with SP_EL0.</td>
            <td align="center">0x000</td>
            <td align="center">0x080</td>
            <td align="center">0x100</td>
            <td align="center">0x180</td>
        </tr>
        <tr>
            <td>Current Exception level with SP_ELx, x>0.</td>
            <td align="center">0x200</td>
            <td align="center">0x280</td>
            <td align="center">0x300</td>
            <td align="center">0x380</td>
        </tr>
        <tr>
            <td>Lower Exception level, where the implemented level immediately lower than the target level is using AArch64.</td>
            <td align="center">0x400</td>
            <td align="center">0x480</td>
            <td align="center">0x500</td>
            <td align="center">0x580</td>
        </tr>
        <tr>
            <td>Lower Exception level, where the implemented level immediately lower than the target level is using AArch32.</td>
            <td align="center">0x600</td>
            <td align="center">0x680</td>
            <td align="center">0x700</td>
            <td align="center">0x780</td>
        </tr>
    </tbody>
</table>

Since our kernel runs in `EL1`, using `SP_EL1`, if we'd cause a synchronous exception, the exception
vector at offset `0x200` would be executed. But what does that even mean?

## Handler Code and Offsets

In many architectures, Operating Systems register their exception handlers (aka vectors) by
compiling an architecturally defined data structure that stores function pointers to the different
handlers. This can be as simple as an ordinary array of function pointers. The `base address` of
this data structure is then stored into a special purpose register so that the CPU can branch to the
respective handler function upon taking an exception. The classic `x86_64` architecture follows this
principle, for example.

In `AArch64`, it is a bit different. Here, we have the special purpose register as well, called
`VBAR_EL1`: Vector Base Address Register.

However, it does not store the base address of an array of function pointers, but the base address
of a **memory location that contains code** for the 16 handlers, one handler back-to-back after the
other. Each handler can take a maximum space of `0x80` bytes, aka `128` bytes. That's why the table
above shows `offsets`: To indicate at which offset a certain handler starts.

Of course, you are not obliged to cram all your handler code into only 128 bytes. You are free to
branch off to any other functions at any time. Actually, that is needed in most cases anyways,
because the context-saving code alone would take up most of the available space (you'll learn what
context saving is shortly).

Additionally, there is a requirement that the `Vector Base Address` is aligned to `0x800` aka `2048`
bytes.

## Rust and Assembly Implementation

The implementation uses a mix of `Rust` and `Assembly` code.

### Context Save and Restore

Exception vectors, just like any other code, use a bunch of commonly shared processor resources.
Most of all, the set of `General Purpose Registers` (GPRs) that each core in `AArch64` provides
(`x0`-`x30`).

In order to not taint these registers when executing exception vector code, it is general practice
to save these shared resources in memory (the stack, to be precise) as the very first action. This
is commonly described as *saving the context*. Exception vector code can then use the shared
resources in its own code without bothering, and as a last action before returning from exception
handling code, restore the context, so that the processor can continue where it left off before
taking the exception.

Context save and restore is one of the few places in system software where it is strongly advised to
to use some hand-crafted assembly. Introducing `exception.S`:

```asm
/// Call the function provided by parameter `\handler` after saving exception context. Provide the
/// context as the first parameter to '\handler'.
.macro CALL_WITH_CONTEXT handler
    // Make room on the stack for the exception context.
    sub    sp,  sp,  #16 * 17

    // Store all general purpose registers on the stack.
    stp    x0,  x1,  [sp, #16 * 0]
    stp    x2,  x3,  [sp, #16 * 1]
    stp    x4,  x5,  [sp, #16 * 2]
    stp    x6,  x7,  [sp, #16 * 3]
    stp    x8,  x9,  [sp, #16 * 4]
    stp    x10, x11, [sp, #16 * 5]
    stp    x12, x13, [sp, #16 * 6]
    stp    x14, x15, [sp, #16 * 7]
    stp    x16, x17, [sp, #16 * 8]
    stp    x18, x19, [sp, #16 * 9]
    stp    x20, x21, [sp, #16 * 10]
    stp    x22, x23, [sp, #16 * 11]
    stp    x24, x25, [sp, #16 * 12]
    stp    x26, x27, [sp, #16 * 13]
    stp    x28, x29, [sp, #16 * 14]

    // Add the exception link register (ELR_EL1) and the saved program status (SPSR_EL1).
    mrs    x1,  ELR_EL1
    mrs    x2,  SPSR_EL1

    stp    lr,  x1,  [sp, #16 * 15]
    str    w2,       [sp, #16 * 16]

    // x0 is the first argument for the function called through `\handler`.
    mov    x0,  sp

    // Call `\handler`.
    bl     \handler

    // After returning from exception handling code, replay the saved context and return via `eret`.
    b      __exception_restore_context
.endm
```

First, a macro for saving the context is defined. It eventually jumps to follow-up handler code, and
finally restores the context. In advance, we reserve space on the stack for the context. That is,
the 30 `GPRs`, the `link register`, the `saved program status` and the `exception link register`
(holding the preferred return address). Afterwards, we store those registers, save the current stack
address in `x0` and branch off to follow-up handler-code, whose function name is supplied as an
argument to the macro (`\handler`).

The handler code will be written in Rust, but use the platform's `C` ABI. This way, we can define a
function signature that has a pointer to the context-data on the stack as its first argument, and
know that this argument is expected to be in the `x0` register. We need to use the `C` ABI here
because `Rust` has no stable convention ([yet](https://github.com/rust-lang/rfcs/issues/600)).

### Exception Vector Table

Next, we craft the exception vector table:

```asm
.section .exception_vectors, "ax", @progbits

// Align by 2^11 bytes, as demanded by the AArch64 spec. Same as ALIGN(2048) in an ld script.
.align 11

// Export a symbol for the Rust code to use.
__exception_vector_start:

// Current exception level with SP_EL0.
// .org sets the offset relative to section start.
//
// It must be ensured that `CALL_WITH_CONTEXT` <= 0x80 bytes.
.org 0x000
    CALL_WITH_CONTEXT current_el0_synchronous
.org 0x080
    CALL_WITH_CONTEXT current_el0_irq
.org 0x100
    FIQ_SUSPEND
.org 0x180
    CALL_WITH_CONTEXT current_el0_serror

// Current exception level with SP_ELx, x > 0.
.org 0x200
    CALL_WITH_CONTEXT current_elx_synchronous
.org 0x280
    CALL_WITH_CONTEXT current_elx_irq
.org 0x300
    FIQ_SUSPEND
.org 0x380
    CALL_WITH_CONTEXT current_elx_serror

[...]
```

Note how each vector starts at the required offset from the section start using the `.org`
directive. Each macro call introduces an explicit handler function name, which is implemented in
`Rust` in `exception.rs`.

### Implementing handlers

The file `exception.rs` first defines a `struct` of the exception context that is stored on the
stack by the assembly code:

```rust
/// The exception context as it is stored on the stack on exception entry.
#[repr(C)]
struct ExceptionContext {
    // General Purpose Registers.
    gpr: [u64; 30],
    // The link register, aka x30.
    lr: u64,
    // Exception link register. The program counter at the time the exception happened.
    elr_el1: u64,
    // Saved program status.
    spsr_el1: SpsrEL1,
}
```

The handlers take a `struct ExceptionContext` argument. Since we do not plan to implement handlers
for each exception yet, a default handler is provided:

```rust
/// Print verbose information about the exception and the panic.
fn default_exception_handler(e: &ExceptionContext) {
    panic!(
        "\n\nCPU Exception!\n\
         FAR_EL1: {:#018x}\n\
         {}\n\
         {}",
        FAR_EL1.get(),
        EsrEL1 {},
        e
    );
}
```

The actual handlers referenced from the assembly can now branch to it for the time being, e.g.:

```rust
#[no_mangle]
unsafe extern "C" fn current_el0_synchronous(e: &mut ExceptionContext) {
    default_exception_handler(e);
}
```

## Causing an Exception - Testing the Code

We want to see two cases in action:
1. How taking, handling and returning from an exception works.
2. How the `panic!` print for unhandled exceptions looks like.


So after setting up exceptions in `main.rs` by calling

```rust
arch::enable_exception_handling();
```

we cause a data abort exception by reading from memory address `8 GiB`:

```rust
// Cause an exception by accessing a virtual address for which no translation was set up. This
// code accesses the address 8 GiB, which is outside the mapped address space.
//
// For demo purposes, the exception handler will catch the faulting 8 GiB address and allow
// execution to continue.
info!("");
info!("Trying to write to address 8 GiB...");
let mut big_addr: u64 = 8 * 1024 * 1024 * 1024;
unsafe { core::ptr::read_volatile(big_addr as *mut u64) };
```

This triggers our exception code, because we try to read from a virtual address for which no mapping
has been installed. Remember, we only installed identity-mapped page tables for the first `1 GiB`
(RPi3) or `4 GiB` (RPi4) of address space in the previous tutorial.

To survive this exception, the respective handler has a special demo case:

```rust
/// Asynchronous exception taken from the current EL, using SP of the current EL.
#[no_mangle]
unsafe extern "C" fn current_elx_synchronous(e: &mut ExceptionContext) {
    let far_el1 = FAR_EL1.extract().get();

    // This catches the demo case for this tutorial. If the fault address happens to be 8 GiB,
    // advance the exception link register for one instruction, so that execution can continue.
    if far_el1 == 8 * 1024 * 1024 * 1024 {
        e.elr_el1 += 4;

        asm::eret()
    }

    default_exception_handler(e);
}
```

It checks if the faulting address equals `8 GiB`, and if so, advances the copy of the `ELR` by 4,
which makes it point to the next instruction after the instruction that caused the exception. When
this handler returns, execution continues in the assembly macro we introduced before. The macro has
only one more line left: `b __exception_restore_context`, which jumps to an assembly function that
plays back our saved context before finally executing `eret` to return from the exception.

This will kick us back into `main.rs`. But we also want to see the `panic!` print.

Therefore, a second read is done, this time from address `9 GiB`. A case which the handler will not
catch, eventually triggering the `panic!` call from the default handler.

## Test it

Emphasis on the events at timestamps > `6.xxxxxx`.

```console
» make chainboot
[...]
Minipush 1.0

[MP] ⏳ Waiting for /dev/ttyUSB0
[MP] ✅ Connected
 __  __ _      _ _                 _
|  \/  (_)_ _ (_) |   ___  __ _ __| |
| |\/| | | ' \| | |__/ _ \/ _` / _` |
|_|  |_|_|_||_|_|____\___/\__,_\__,_|

           Raspberry Pi 3

[ML] Requesting binary
[MP] ⏩ Pushing 64 KiB ========================================🦀 100% 32 KiB/s Time: 00:00:02
[ML] Loaded! Executing the payload now

[    2.913260] Booting on: Raspberry Pi 3
[    2.914344] MMU online. Special regions:
[    2.916256]       0x00080000 - 0x0008ffff |  64 KiB | C   RO PX  | Kernel code and RO data
[    2.920338]       0x3f000000 - 0x3fffffff |  16 MiB | Dev RW PXN | Device MMIO
[    2.923901] Current privilege level: EL1
[    2.925812] Exception handling state:
[    2.927593]       Debug:  Masked
[    2.929156]       SError: Masked
[    2.930720]       IRQ:    Masked
[    2.932284]       FIQ:    Masked
[    2.933848] Architectural timer resolution: 52 ns
[    2.936150] Drivers loaded:
[    2.937496]       1. GPIO
[    2.938756]       2. PL011Uart
[    2.940233] Timer test, spinning for 1 second
[    3.942362]
[    3.942366] Trying to write to address 8 GiB...
[    3.944531] ************************************************
[    3.947310] Whoa! We recovered from a synchronous exception!
[    3.950091] ************************************************
[    3.952870]
[    3.953566] Let's try again
[    3.954912] Trying to write to address 9 GiB...

Kernel panic:

CPU Exception!
FAR_EL1: 0x0000000240000000
ESR_EL1: 0x96000004
      Exception Class         (EC) : 0x25 - Data Abort, current EL
      Instr Specific Syndrome (ISS): 0x4
ELR_EL1: 0x0000000000080e50
SPSR_EL1: 0x600003c5
      Flags:
            Negative (N): Not set
            Zero     (Z): Set
            Carry    (C): Set
            Overflow (V): Not set
      Exception handling state:
            Debug  (D): Masked
            SError (A): Masked
            IRQ    (I): Masked
            FIQ    (F): Masked
      Illegal Execution State (IL): Not set

General purpose register:
      x0 : 0x0000000000000000         x1 : 0x000000000008594e
      x2 : 0x0000000000000026         x3 : 0x0000000000082b38
      x4 : 0x000000000007fc5c         x5 : 0x0000000000000003
      x6 : 0x0000000000000000         x7 : 0xd3d1c80822850243
      x8 : 0x0000000240000000         x9 : 0x000000000008594e
      x10: 0x0000000000000414         x11: 0x000000003f201000
      x12: 0x0000000000000019         x13: 0x000000000007fc5d
      x14: 0x000000000007fda8         x15: 0x0000000000000040
      x16: 0x0000000000000000         x17: 0x0000000000000040
      x18: 0x9cc47880812f1200         x19: 0x0000000000090008
      x20: 0x000000003b9aca00         x21: 0x00000000000003e8
      x22: 0x0000000000083070         x23: 0x00000000000831e4
      x24: 0x00000000000f4240         x25: 0x00000000000852a8
      x26: 0x0000000000085738         x27: 0x0000000000085818
      x28: 0x00000000000831e4         x29: 0x0000000000085588
      lr : 0x0000000000080e44
```

## Diff to previous
```diff

diff -uNr 11_virtual_memory/src/arch/aarch64/exception.rs 12_cpu_exceptions_part1/src/arch/aarch64/exception.rs
--- 11_virtual_memory/src/arch/aarch64/exception.rs
+++ 12_cpu_exceptions_part1/src/arch/aarch64/exception.rs
@@ -4,12 +4,248 @@

 //! Exception handling.

-use cortex_a::regs::*;
+use core::fmt;
+use cortex_a::{asm, barrier, regs::*};
+use register::InMemoryRegister;
+
+// Assembly counterpart to this file.
+global_asm!(include_str!("exception.S"));
+
+/// Wrapper struct for memory copy of SPSR_EL1.
+#[repr(transparent)]
+struct SpsrEL1(InMemoryRegister<u32, SPSR_EL1::Register>);
+
+/// The exception context as it is stored on the stack on exception entry.
+#[repr(C)]
+struct ExceptionContext {
+    // General Purpose Registers.
+    gpr: [u64; 30],
+    // The link register, aka x30.
+    lr: u64,
+    // Exception link register. The program counter at the time the exception happened.
+    elr_el1: u64,
+    // Saved program status.
+    spsr_el1: SpsrEL1,
+}
+
+/// Wrapper struct for pretty printing ESR_EL1.
+struct EsrEL1;
+
+//--------------------------------------------------------------------------------------------------
+// Exception vector implementation
+//--------------------------------------------------------------------------------------------------
+
+/// Print verbose information about the exception and the panic.
+fn default_exception_handler(e: &ExceptionContext) {
+    panic!(
+        "\n\nCPU Exception!\n\
+         FAR_EL1: {:#018x}\n\
+         {}\n\
+         {}",
+        FAR_EL1.get(),
+        EsrEL1 {},
+        e
+    );
+}
+
+//--------------------------------------------------------------------------------------------------
+// Current, EL0
+//--------------------------------------------------------------------------------------------------
+
+#[no_mangle]
+unsafe extern "C" fn current_el0_synchronous(e: &mut ExceptionContext) {
+    default_exception_handler(e);
+}
+
+#[no_mangle]
+unsafe extern "C" fn current_el0_irq(e: &mut ExceptionContext) {
+    default_exception_handler(e);
+}
+
+#[no_mangle]
+unsafe extern "C" fn current_el0_serror(e: &mut ExceptionContext) {
+    default_exception_handler(e);
+}
+
+//--------------------------------------------------------------------------------------------------
+// Current, ELx
+//--------------------------------------------------------------------------------------------------
+
+/// Asynchronous exception taken from the current EL, using SP of the current EL.
+#[no_mangle]
+unsafe extern "C" fn current_elx_synchronous(e: &mut ExceptionContext) {
+    let far_el1 = FAR_EL1.get();
+
+    // This catches the demo case for this tutorial. If the fault address happens to be 8 GiB,
+    // advance the exception link register for one instruction, so that execution can continue.
+    if far_el1 == 8 * 1024 * 1024 * 1024 {
+        e.elr_el1 += 4;
+
+        asm::eret()
+    }
+
+    default_exception_handler(e);
+}
+
+#[no_mangle]
+unsafe extern "C" fn current_elx_irq(e: &mut ExceptionContext) {
+    default_exception_handler(e);
+}
+
+#[no_mangle]
+unsafe extern "C" fn current_elx_serror(e: &mut ExceptionContext) {
+    default_exception_handler(e);
+}
+
+//--------------------------------------------------------------------------------------------------
+// Lower, AArch64
+//--------------------------------------------------------------------------------------------------
+
+#[no_mangle]
+unsafe extern "C" fn lower_aarch64_synchronous(e: &mut ExceptionContext) {
+    default_exception_handler(e);
+}
+
+#[no_mangle]
+unsafe extern "C" fn lower_aarch64_irq(e: &mut ExceptionContext) {
+    default_exception_handler(e);
+}
+
+#[no_mangle]
+unsafe extern "C" fn lower_aarch64_serror(e: &mut ExceptionContext) {
+    default_exception_handler(e);
+}
+
+//--------------------------------------------------------------------------------------------------
+// Lower, AArch32
+//--------------------------------------------------------------------------------------------------
+
+#[no_mangle]
+unsafe extern "C" fn lower_aarch32_synchronous(e: &mut ExceptionContext) {
+    default_exception_handler(e);
+}
+
+#[no_mangle]
+unsafe extern "C" fn lower_aarch32_irq(e: &mut ExceptionContext) {
+    default_exception_handler(e);
+}
+
+#[no_mangle]
+unsafe extern "C" fn lower_aarch32_serror(e: &mut ExceptionContext) {
+    default_exception_handler(e);
+}
+
+//--------------------------------------------------------------------------------------------------
+// Pretty printing
+//--------------------------------------------------------------------------------------------------
+
+/// Human readable ESR_EL1.
+#[rustfmt::skip]
+impl fmt::Display for EsrEL1 {
+    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
+        let esr_el1 = ESR_EL1.extract();
+
+        // Raw print of whole register.
+        writeln!(f, "ESR_EL1: {:#010x}", esr_el1.get())?;
+
+        // Raw print of exception class.
+        write!(f, "      Exception Class         (EC) : {:#x}", esr_el1.read(ESR_EL1::EC))?;
+
+        // Exception class, translation.
+        let ec_translation = match esr_el1.read_as_enum(ESR_EL1::EC) {
+            Some(ESR_EL1::EC::Value::DataAbortCurrentEL) => "Data Abort, current EL",
+            _ => "N/A",
+        };
+        writeln!(f, " - {}", ec_translation)?;
+
+        // Raw print of instruction specific syndrome.
+        write!(f, "      Instr Specific Syndrome (ISS): {:#x}", esr_el1.read(ESR_EL1::ISS))?;
+
+        Ok(())
+    }
+}
+
+/// Human readable SPSR_EL1.
+#[rustfmt::skip]
+impl fmt::Display for SpsrEL1 {
+    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
+        // Raw value.
+        writeln!(f, "SPSR_EL1: {:#010x}", self.0.get())?;
+
+        let to_flag_str = |x| -> _ {
+            if x { "Set" } else { "Not set" }
+         };
+
+        writeln!(f, "      Flags:")?;
+        writeln!(f, "            Negative (N): {}", to_flag_str(self.0.is_set(SPSR_EL1::N)))?;
+        writeln!(f, "            Zero     (Z): {}", to_flag_str(self.0.is_set(SPSR_EL1::Z)))?;
+        writeln!(f, "            Carry    (C): {}", to_flag_str(self.0.is_set(SPSR_EL1::C)))?;
+        writeln!(f, "            Overflow (V): {}", to_flag_str(self.0.is_set(SPSR_EL1::V)))?;
+
+        let to_mask_str = |x| -> _ {
+            if x { "Masked" } else { "Unmasked" }
+        };
+
+        writeln!(f, "      Exception handling state:")?;
+        writeln!(f, "            Debug  (D): {}", to_mask_str(self.0.is_set(SPSR_EL1::D)))?;
+        writeln!(f, "            SError (A): {}", to_mask_str(self.0.is_set(SPSR_EL1::A)))?;
+        writeln!(f, "            IRQ    (I): {}", to_mask_str(self.0.is_set(SPSR_EL1::I)))?;
+        writeln!(f, "            FIQ    (F): {}", to_mask_str(self.0.is_set(SPSR_EL1::F)))?;
+
+        write!(f, "      Illegal Execution State (IL): {}",
+            to_flag_str(self.0.is_set(SPSR_EL1::IL))
+        )?;
+
+        Ok(())
+    }
+}
+
+/// Human readable print of the exception context.
+impl fmt::Display for ExceptionContext {
+    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
+        writeln!(f, "ELR_EL1: {:#018x}", self.elr_el1)?;
+        writeln!(f, "{}", self.spsr_el1)?;
+        writeln!(f)?;
+        writeln!(f, "General purpose register:")?;
+
+        #[rustfmt::skip]
+        let alternating = |x| -> _ {
+            if x modulo 2 == 0 { "   " } else { "\n" }
+        };
+
+        // Print two registers per line.
+        for (i, reg) in self.gpr.iter().enumerate() {
+            write!(f, "      x{: <2}: {: >#018x}{}", i, reg, alternating(i))?;
+        }
+        write!(f, "      lr : {:#018x}", self.lr)?;
+
+        Ok(())
+    }
+}

 //--------------------------------------------------------------------------------------------------
 // Arch-public
 //--------------------------------------------------------------------------------------------------

+/// Set the exception vector base address register.
+///
+/// # Safety
+///
+/// - The vector table and the symbol `__exception_vector_table_start` from the linker script must
+///   adhere to the alignment and size constraints demanded by the AArch64 spec.
+pub unsafe fn set_vbar_el1() {
+    // Provided by exception.S.
+    extern "C" {
+        static mut __exception_vector_start: u64;
+    }
+    let addr: u64 = &__exception_vector_start as *const _ as u64;
+
+    VBAR_EL1.set(addr);
+
+    // Force VBAR update to complete before next instruction.
+    barrier::isb(barrier::SY);
+}
+
 pub trait DaifField {
     fn daif_field() -> register::Field<u32, DAIF::Register>;
 }

diff -uNr 11_virtual_memory/src/arch/aarch64/exception.S 12_cpu_exceptions_part1/src/arch/aarch64/exception.S
--- 11_virtual_memory/src/arch/aarch64/exception.S
+++ 12_cpu_exceptions_part1/src/arch/aarch64/exception.S
@@ -0,0 +1,133 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
+
+/// Call the function provided by parameter `\handler` after saving exception context. Provide the
+/// context as the first parameter to '\handler'.
+.macro CALL_WITH_CONTEXT handler
+    // Make room on the stack for the exception context.
+    sub    sp,  sp,  #16 * 17
+
+    // Store all general purpose registers on the stack.
+    stp    x0,  x1,  [sp, #16 * 0]
+    stp    x2,  x3,  [sp, #16 * 1]
+    stp    x4,  x5,  [sp, #16 * 2]
+    stp    x6,  x7,  [sp, #16 * 3]
+    stp    x8,  x9,  [sp, #16 * 4]
+    stp    x10, x11, [sp, #16 * 5]
+    stp    x12, x13, [sp, #16 * 6]
+    stp    x14, x15, [sp, #16 * 7]
+    stp    x16, x17, [sp, #16 * 8]
+    stp    x18, x19, [sp, #16 * 9]
+    stp    x20, x21, [sp, #16 * 10]
+    stp    x22, x23, [sp, #16 * 11]
+    stp    x24, x25, [sp, #16 * 12]
+    stp    x26, x27, [sp, #16 * 13]
+    stp    x28, x29, [sp, #16 * 14]
+
+    // Add the exception link register (ELR_EL1) and the saved program status (SPSR_EL1).
+    mrs    x1,  ELR_EL1
+    mrs    x2,  SPSR_EL1
+
+    stp    lr,  x1,  [sp, #16 * 15]
+    str    w2,       [sp, #16 * 16]
+
+    // x0 is the first argument for the function called through `\handler`.
+    mov    x0,  sp
+
+    // Call `\handler`.
+    bl     \handler
+
+    // After returning from exception handling code, replay the saved context and return via `eret`.
+    b      __exception_restore_context
+.endm
+
+.macro FIQ_SUSPEND
+1:  wfe
+    b      1b
+.endm
+
+//--------------------------------------------------------------------------------------------------
+// The exception vector table.
+//--------------------------------------------------------------------------------------------------
+.section .exception_vectors, "ax", @progbits
+
+// Align by 2^11 bytes, as demanded by the AArch64 spec. Same as ALIGN(2048) in an ld script.
+.align 11
+
+// Export a symbol for the Rust code to use.
+__exception_vector_start:
+
+// Current exception level with SP_EL0.
+// .org sets the offset relative to section start.
+//
+// It must be ensured that `CALL_WITH_CONTEXT` <= 0x80 bytes.
+.org 0x000
+    CALL_WITH_CONTEXT current_el0_synchronous
+.org 0x080
+    CALL_WITH_CONTEXT current_el0_irq
+.org 0x100
+    FIQ_SUSPEND
+.org 0x180
+    CALL_WITH_CONTEXT current_el0_serror
+
+// Current exception level with SP_ELx, x > 0.
+.org 0x200
+    CALL_WITH_CONTEXT current_elx_synchronous
+.org 0x280
+    CALL_WITH_CONTEXT current_elx_irq
+.org 0x300
+    FIQ_SUSPEND
+.org 0x380
+    CALL_WITH_CONTEXT current_elx_serror
+
+// Lower exception level, aarch64
+.org 0x400
+    CALL_WITH_CONTEXT lower_aarch64_synchronous
+.org 0x480
+    CALL_WITH_CONTEXT lower_aarch64_irq
+.org 0x500
+    FIQ_SUSPEND
+.org 0x580
+    CALL_WITH_CONTEXT lower_aarch64_serror
+
+// Lower exception level, aarch32
+.org 0x600
+    CALL_WITH_CONTEXT lower_aarch32_synchronous
+.org 0x680
+    CALL_WITH_CONTEXT lower_aarch32_irq
+.org 0x700
+    FIQ_SUSPEND
+.org 0x780
+    CALL_WITH_CONTEXT lower_aarch32_serror
+.org 0x800
+
+//--------------------------------------------------------------------------------------------------
+// Helper functions
+//--------------------------------------------------------------------------------------------------
+__exception_restore_context:
+    ldr    w19,      [sp, #16 * 16]
+    ldp    lr,  x20, [sp, #16 * 15]
+
+    msr    SPSR_EL1, x19
+    msr    ELR_EL1,  x20
+
+    ldp    x0,  x1,  [sp, #16 * 0]
+    ldp    x2,  x3,  [sp, #16 * 1]
+    ldp    x4,  x5,  [sp, #16 * 2]
+    ldp    x6,  x7,  [sp, #16 * 3]
+    ldp    x8,  x9,  [sp, #16 * 4]
+    ldp    x10, x11, [sp, #16 * 5]
+    ldp    x12, x13, [sp, #16 * 6]
+    ldp    x14, x15, [sp, #16 * 7]
+    ldp    x16, x17, [sp, #16 * 8]
+    ldp    x18, x19, [sp, #16 * 9]
+    ldp    x20, x21, [sp, #16 * 10]
+    ldp    x22, x23, [sp, #16 * 11]
+    ldp    x24, x25, [sp, #16 * 12]
+    ldp    x26, x27, [sp, #16 * 13]
+    ldp    x28, x29, [sp, #16 * 14]
+
+    add    sp,  sp,  #16 * 17
+
+    eret

diff -uNr 11_virtual_memory/src/arch/aarch64.rs 12_cpu_exceptions_part1/src/arch/aarch64.rs
--- 11_virtual_memory/src/arch/aarch64.rs
+++ 12_cpu_exceptions_part1/src/arch/aarch64.rs
@@ -106,6 +106,15 @@
     }
 }

+/// Enable exception handling.
+///
+/// # Safety
+///
+/// - Changes the HW state of the processing element.
+pub unsafe fn enable_exception_handling() {
+    exception::set_vbar_el1();
+}
+
 /// Return a reference to an `interface::mm::MMU` implementation.
 pub fn mmu() -> &'static impl interface::mm::MMU {
     &MMU

diff -uNr 11_virtual_memory/src/bsp/rpi/virt_mem_layout.rs 12_cpu_exceptions_part1/src/bsp/rpi/virt_mem_layout.rs
--- 11_virtual_memory/src/bsp/rpi/virt_mem_layout.rs
+++ 12_cpu_exceptions_part1/src/bsp/rpi/virt_mem_layout.rs
@@ -15,7 +15,7 @@
 // BSP-public
 //--------------------------------------------------------------------------------------------------

-pub const NUM_MEM_RANGES: usize = 3;
+pub const NUM_MEM_RANGES: usize = 2;

 pub static LAYOUT: KernelVirtualLayout<{ NUM_MEM_RANGES }> = KernelVirtualLayout::new(
     memory_map::END_INCLUSIVE,
@@ -54,19 +54,6 @@
             },
         },
         RangeDescriptor {
-            name: "Remapped Device MMIO",
-            virtual_range: || {
-                // The last 64 KiB slot in the first 512 MiB
-                RangeInclusive::new(0x1FFF_0000, 0x1FFF_FFFF)
-            },
-            translation: Translation::Offset(memory_map::mmio::BASE + 0x20_0000),
-            attribute_fields: AttributeFields {
-                mem_attributes: MemAttributes::Device,
-                acc_perms: AccessPermissions::ReadWrite,
-                execute_never: true,
-            },
-        },
-        RangeDescriptor {
             name: "Device MMIO",
             virtual_range: || {
                 RangeInclusive::new(memory_map::mmio::BASE, memory_map::mmio::END_INCLUSIVE)

diff -uNr 11_virtual_memory/src/bsp.rs 12_cpu_exceptions_part1/src/bsp.rs
--- 11_virtual_memory/src/bsp.rs
+++ 12_cpu_exceptions_part1/src/bsp.rs
@@ -4,7 +4,7 @@

 //! Conditional exporting of Board Support Packages.

-pub mod driver;
+mod driver;

 #[cfg(any(feature = "bsp_rpi3", feature = "bsp_rpi4"))]
 mod rpi;

diff -uNr 11_virtual_memory/src/main.rs 12_cpu_exceptions_part1/src/main.rs
--- 11_virtual_memory/src/main.rs
+++ 12_cpu_exceptions_part1/src/main.rs
@@ -22,6 +22,7 @@
 #![allow(incomplete_features)]
 #![feature(const_generics)]
 #![feature(format_args_nl)]
+#![feature(global_asm)]
 #![feature(panic_info_message)]
 #![feature(trait_alias)]
 #![no_main]
@@ -57,6 +58,8 @@
 unsafe fn kernel_init() -> ! {
     use interface::mm::MMU;

+    arch::enable_exception_handling();
+
     if let Err(string) = arch::mmu().init() {
         panic!("MMU: {}", string);
     }
@@ -102,13 +105,28 @@
     info!("Timer test, spinning for 1 second");
     arch::timer().spin_for(Duration::from_secs(1));

-    let remapped_uart = unsafe { bsp::driver::PL011Uart::new(0x1FFF_1000) };
-    writeln!(
-        remapped_uart,
-        "[     !!!    ] Writing through the remapped UART at 0x1FFF_1000"
-    )
-    .unwrap();
+    // Cause an exception by accessing a virtual address for which no translation was set up. This
+    // code accesses the address 8 GiB, which is outside the mapped address space.
+    //
+    // For demo purposes, the exception handler will catch the faulting 8 GiB address and allow
+    // execution to continue.
+    info!("");
+    info!("Trying to write to address 8 GiB...");
+    let mut big_addr: u64 = 8 * 1024 * 1024 * 1024;
+    unsafe { core::ptr::read_volatile(big_addr as *mut u64) };
+
+    info!("************************************************");
+    info!("Whoa! We recovered from a synchronous exception!");
+    info!("************************************************");
+    info!("");
+    info!("Let's try again");
+
+    // Now use address 9 GiB. The exception handler won't forgive us this time.
+    info!("Trying to write to address 9 GiB...");
+    big_addr = 9 * 1024 * 1024 * 1024;
+    unsafe { core::ptr::read_volatile(big_addr as *mut u64) };

+    // Will never reach here in this tutorial.
     info!("Echoing input now");
     loop {
         let c = bsp::console().read_char();

```
