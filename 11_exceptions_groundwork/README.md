# Tutorial 11 - Exceptions: Groundwork


In this tutorial, we lay the groundwork for taking exceptions, and write a very bare-bones handler for a synchronous exception that happens in `EL1`.

More tutorials on exceptions will follow, implementing and introducing various other aspects of this rather huge topic.

## Exception Types

In `AArch64`, it is differentiated between four types of exceptions. These are:
- Synchronous
  - For example, a `data abort` or a `system call`. They happen in direct consequence of executing a certain instruction, hence _synchronously_.
- Interrupt Request (`IRQ`)
  - For example, an external device, like a timer, is asserting a physical interrupt line. IRQs happen _asynchronously_.
- Fast Interrupt Request (`FIQ`)
  - These are basically interrupts that take priority over normal IRQs and have some more traits that make them suitable to implement super-fast processing. However, this is out of scope for this tutorial. For the sake of keeping these tutorials compact and concise, we will more or less ignore FIQs and only implement a dummy handler that would halt the CPU core.
- System Error (`SError`)
  - Like IRQs, SErrors happen asynchronously and are technically more or less the same. They are intended to signal rather fatal errors in the system, e.g. if a transaction times out on the `SoC` interconnect. They are highly implementation specific and it is up to the SoC designer to decide which events are delivered as SErrors instead of normal IRQs.

## Exception entry

We recommend to read pages 1874-1876 of the [ARMv8 Architecture Reference Manual][ARMv8_Manual] to understand the mechanisms of taking an exception.

Here's an excerpt of important features for this tutorial:
- Exception entry moves the processor to the same or a higher `Exception Level`, but never to a lower `EL`.
- The program status is saved in the `SPSR_ELx` register at the target `EL`.
- The preferred return address is saved in the `ELR_ELx` register.
  - "Preferred" here means that `ELR_ELx` may hold the instruction address of the instructions that caused the exception (`synchronous case`) or the first instruction that did not complete due to an `asynchronous` exception. Details in Chapter D1.10.1 of the [ARMv8 Architecture Reference Manual][ARMv8_Manual].
- All kinds of exceptions are turned off upon taking an exception, so that by default exception handlers can not get interrupted themselves.
- Taking an exception will select the dedicated stack pointer of the target `EL`.
  - For example, if an exception in `EL0` is taken, the Stack Pointer Select register `SPSel` will switch from `0` to `1`, meaning that `SP_EL1` will be used by the exception vector code unless you explicitly change it back to `SP_EL0`.


### Exception Vectors

`AArch64` has a total of 16 exception vectors. There is one for each of the four kinds that were introduced already, and additionally, it is taken into account _where_ the exception was taken from and what the circumstances were.

Here is a copy of the decision table as shown in Chapter D1.10.2 of the [ARMv8 Architecture Reference Manual][ARMv8_Manual]:

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

Since our bare-metal Raspberry code operates in `EL1` using `SP_EL1`, if we'd cause a synchronous exception, the exception vector at offset `0x200` would be executed. But what does that even mean?

## Handler Code and Offsets

In many architectures, Operating Systems register their exception handlers (aka vectors) by compiling an architecturally defined data structure that stores function pointers to the different handlers. This can be as simple as an ordinary array of function pointers. The `base address` of this data structure is then stored into a special purpose register so that the CPU can branch to the respective handler function upon taking an exception. The famous `x86_64` architecture follows this principle, for example.

In `AArch64`, it is a bit different. Here, we have the special purpose register as well, called `VBAR_EL1`: Vector Base Address Register.

However, it does not store the base address of an array of function pointers, but the base address of a **memory location that contains code** for the 16 handlers, one handler back-to-back after the other. Each handler can take a maximum space of `0x80` bytes, aka 128 bytes. That's why the table above shows `offsets`: To indicate at which offset a certain handler starts.

Of course, you are not obliged to cram all your handler code into only 128 bytes. You are free to branch off to any other functions at any time. Actually, that is needed in most cases anyways, because the context-saving code alone would take up most of the available space (You'll learn about what context saving is shortly).

Additionally, there is a requirement that the `Vector Base Address` is aligned to `0x800` aka 2048 bytes.

## Rust Implementation

We start by adding a new section to the `link.ld` script, which will contain the exception vector code:

```rust
SECTIONS
{
    .vectors ALIGN(2048):
    {
        *(.vectors)
    }
```

### Context Save and Restore

Exception vectors, just like any other code, use a bunch of commonly shared processor resources. Most of all, the set of `General Purpose Registers` (GPRs) that each core in `AArch64` provides (`X0`-`X30`).

In order to not taint these registers when executing exception vector code, it is general practice to save these shared resources in memory (the stack, to be precise) as the very first action. This is commonly described as *saving the context*. Exception vector code can then use the shared resources in its own code without bothering, and as a last action before returning from exception handling code, restore the context, so that the processor can continue where it left off before taking the exception.

Context save and restore is one of the few places in system software where it is strongly advised to to use some hand-crafted assembly. Introducing `vectors.S`:

```asm
.macro SAVE_CONTEXT_CALL_HANDLER_AND_RESTORE handler
.balign 0x80

    sub    sp,  sp,  #16 * 17

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

    mrs    x1,  SPSR_EL1
    mrs    x2,  ELR_EL1

    stp    x30, x1,  [sp, #16 * 15]
    str    x2,       [sp, #16 * 16]

    mov    x0,  sp
    bl     \handler
    b      __restore_context
.endm
```

First, a macro for saving the context, branching to follow-up handler code, and finally restoring the context. In advance, we reserve space on the stack for the context. That is, the 30 `GPRs` as well as the `saved program status` and the `exception link register` (holding the preferred return address). Afterwards, we store those registers, save the current stack address in `X0` and branch off to follow-up handler-code, whose function name is supplied as an argument to the macro.

The handler code will be written in Rust, but use the platform's `C` ABI. This way, we can define a function signature that has a pointer to the context-data on the stack as its first argument, and know that this argument is expected to be in the `X0` register. We need to use the `C` ABI here because `Rust` has no stable convention ([yet](https://github.com/rust-lang/rfcs/issues/600)).

Also note the `.balign 0x80` which ensure that the code is aligned properly according to the table shown earlier.

### Exception Vector Code

Next, we populate the `.vectors` section of the linker script using our macro (except for the FIQ vectors, for which we insert code that halts the CPU (via another macro):

```asm
.section .vectors, "ax"
.global __exception_vectors_start
__exception_vectors_start:
    SAVE_CONTEXT_CALL_HANDLER_AND_RESTORE current_el0_synchronous   // 0x000
    SAVE_CONTEXT_CALL_HANDLER_AND_RESTORE current_el0_irq           // 0x080
    FIQ_DUMMY                                                       // 0x100
    SAVE_CONTEXT_CALL_HANDLER_AND_RESTORE current_el0_serror        // 0x180

    SAVE_CONTEXT_CALL_HANDLER_AND_RESTORE current_elx_synchronous   // 0x200
    SAVE_CONTEXT_CALL_HANDLER_AND_RESTORE current_elx_irq           // 0x280
    FIQ_DUMMY                                                       // 0x300
    SAVE_CONTEXT_CALL_HANDLER_AND_RESTORE current_elx_serror        // 0x380

    SAVE_CONTEXT_CALL_HANDLER_AND_RESTORE lower_aarch64_synchronous // 0x400
    SAVE_CONTEXT_CALL_HANDLER_AND_RESTORE lower_aarch64_irq         // 0x480
    FIQ_DUMMY                                                       // 0x500
    SAVE_CONTEXT_CALL_HANDLER_AND_RESTORE lower_aarch64_serror      // 0x580

    SAVE_CONTEXT_CALL_HANDLER_AND_RESTORE lower_aarch32_synchronous // 0x600
    SAVE_CONTEXT_CALL_HANDLER_AND_RESTORE lower_aarch32_irq         // 0x680
    FIQ_DUMMY                                                       // 0x700
    SAVE_CONTEXT_CALL_HANDLER_AND_RESTORE lower_aarch32_serror      // 0x780
```

This part introduces various handler function names, which we can now implement using `Rust`.

### Implementing a handler

In `exception.rs`, we implement a handler for a synchronous exception that that will happen in `EL1` using `SP_EL1`:

```rust
#[no_mangle]
unsafe extern "C" fn current_elx_synchronous(e: &mut ExceptionContext) {
    println!("[!] A synchronous exception happened.");
    println!("      ELR_EL1: {:#010X}", e.elr_el1);
    println!(
        "      Incrementing ELR_EL1 by 4 now to continue with the first \
         instruction after the exception!"
    );

    e.elr_el1 += 4;

    println!("      ELR_EL1 modified: {:#010X}", e.elr_el1);
    println!("      Returning from exception...\n");
}
```

The function takes a `struct ExceptionContext` argument, which resembles what we put on the stack before branching to the handler:

```rust
#[repr(C)]
pub struct GPR {
    x: [u64; 31],
}

#[repr(C)]
pub struct ExceptionContext {
    // General Purpose Registers
    gpr: GPR,
    spsr_el1: u64,
    elr_el1: u64,
}
```

Inside the function, for demo purposes, we advance the copy of the `ELR` by 4, which makes it point to the next instruction after the instruction that caused the exception.
When the function returns, execution continues in the assembly macro we introduced before. The macro has only one more line left: `b __restore_context`, which jumps to an assembly function that plays back our saved context before finally executing `eret` to return from the exception.

#### Default handler

In order to spare the work of implementing each and every handler, we define an `extern "C" fn default_exception_handler()`. Using the linker script, we take a shortcut and make all the other handlers point to this function code if it is not implemented explicitly anywhere else:

```rust
PROVIDE(current_el0_synchronous   = default_exception_handler);
PROVIDE(current_el0_irq           = default_exception_handler);
PROVIDE(current_el0_serror        = default_exception_handler);

...(Many more omitted)
```

## Causing an Exception - Testing the Code

After pointing `VBAR_EL1` to our vector code,

```rust
exception::set_vbar_el1_checked(exception_vectors_start)
```
which enables exception handling, we cause a data abort exception by reading from memory address `3 GiB`:

```rust
let big_addr: u64 = 3 * 1024 * 1024 * 1024;
unsafe { core::ptr::read_volatile(big_addr as *mut u64) };
```

Finally, this triggers our exception code, because we try to read from a virtual address for which no address translations have been installed. Remember, we only installed identity-mapped page tables for the first 1 GiB of address space in lesson `0D`.
After the exception handler is finished, it returns to the first instruction
after the memory read that caused the exception.

## Output

```console
ferris@box:~$ make raspboot

[0] MiniUart online.
[1] Press a key to continue booting... Greetings fellow Rustacean!
[2] MMU online.
[i] Kernel memory layout:
      0x00000000 - 0x0007FFFF | 512 KiB | C   RW PXN | Kernel stack
      0x00080000 - 0x00084FFF |  20 KiB | C   RO PX  | Kernel code and RO data
      0x00085000 - 0x0008800F |  12 KiB | C   RW PXN | Kernel data and BSS
      0x00200000 - 0x005FFFFF |   4 MiB | NC  RW PXN | DMA heap pool
      0x3F000000 - 0x3FFFFFFF |  16 MiB | Dev RW PXN | Device MMIO
[i] Global DMA Allocator:
      Allocated Addr 0x00200000 Size 0x90
[3] Videocore Mailbox set up (DMA mem heap allocation successful).
[4] PL011 UART online. Output switched to it.
[5] Exception vectors are set up.
[!] A synchronous exception happened.
      ELR_EL1: 0x00080C20
      Incrementing ELR_EL1 by 4 now to continue with the first instruction after the exception!
      ELR_EL1 modified: 0x00080C24
      Returning from exception...

[i] Whoa! We recovered from an exception.

$>
```
