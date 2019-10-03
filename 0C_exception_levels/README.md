# Tutorial 0C - Exception Levels

In `AArch64`, there are four so-called exception levels:

| Exception Level  | Typically used for |
| ------------- | ------------- |
| EL0 | Userspace applications |
| EL1 | OS Kernel |
| EL2 | Hypervisor |
| EL3 | Low-Level Firmware |

If you are familiar with the `x86` architecture, `ELs` are the counterpart to [privilege rings](https://en.wikipedia.org/wiki/Protection_ring).

At this point, I strongly recommend that you glimpse over `Chapter 3` of the [Programmer’s Guide for ARMv8-A](http://infocenter.arm.com/help/topic/com.arm.doc.den0024a/DEN0024A_v8_architecture_PG.pdf) before you continue.
It gives a concise overview about the topic.

## Scope of this tutorial

If you set up your SD Card exactly like mentioned in the repository's [top-level README](../README.md#prerequisites),
our binary will start executing in `EL2`. Since we have an OS-focus, we will now write code that will cause a transition
into the more appropriate `EL1`.

## Checking for EL2 in the entrypoint

First of all, we need to ensure that we actually run in `EL2` before we can call respective code to transition to EL1:

```rust
/// Entrypoint of the processor.
///
/// Parks all cores except core0 and checks if we started in EL2. If
/// so, proceeds with setting up EL1.
#[link_section = ".text.boot"]
#[no_mangle]
pub unsafe extern "C" fn _boot_cores() -> ! {
    use cortex_a::{asm, regs::*};

    const CORE_0: u64 = 0;
    const CORE_MASK: u64 = 0x3;
    const EL2: u32 = CurrentEL::EL::EL2.value;

    if (CORE_0 == MPIDR_EL1.get() & CORE_MASK) && (EL2 == CurrentEL.get()) {
        setup_and_enter_el1_from_el2()
    }

    // if not core0 or EL != 2, infinitely wait for events
    loop {
        asm::wfe();
    }
}
```

If this is the case, we continue with preparing the `EL2` -> `EL1` transition in `setup_and_enter_el1_from_el2()`.

## Transition preparation

Since `EL2` is more privileged than `EL1`, it has control over various processor features and can allow or disallow
`EL1` code to use them. One such example is access to timer and counter registers. We are already using them since
[tutorial 09_delays](../09_delays/), so we want to keep them. Therefore we set the respective flags in the
[Counter-timer Hypervisor Control register](https://docs.rs/cortex-a/2.4.0/src/cortex_a/regs/cnthctl_el2.rs.html)
and additionally set the virtual offset to zero so that we get the real physical value everytime:

```rust
// Enable timer counter registers for EL1
CNTHCTL_EL2.write(CNTHCTL_EL2::EL1PCEN::SET + CNTHCTL_EL2::EL1PCTEN::SET);

// No offset for reading the counters
CNTVOFF_EL2.set(0);
```

Next, we configure the [Hypervisor Configuration Register](https://docs.rs/cortex-a/2.4.0/src/cortex_a/regs/hcr_el2.rs.html) such that `EL1` should actually run in `AArch64` mode, and not in `AArch32`, which would also be possible.

```rust
// Set EL1 execution state to AArch64
HCR_EL2.write(HCR_EL2::RW::EL1IsAarch64;
```

## Returning from an exception that never happened

There is actually only one way to transition from a higher EL to a lower EL, which is by way of executing
the [ERET](https://docs.rs/cortex-a/2.4.0/src/cortex_a/asm.rs.html#49-62) instruction.

This instruction will copy the contents of the [Saved Program Status Register - EL2](https://docs.rs/cortex-a/2.4.0/src/cortex_a/regs/spsr_el2.rs.html)
to `Current Program Status Register - EL1` and jump to the instruction address that is stored in the [Exception Link Register - EL2](https://docs.rs/cortex-a/2.4.0/src/cortex_a/regs/elr_el2.rs.html).

This is basically the reverse of what is happening when an exception is taken. You'll learn about it in tutorial [10_exception_groundwork](../10_exceptions_groundwork).

```rust
// Set up a simulated exception return.
//
// First, fake a saved program status, where all interrupts were
// masked and SP_EL1 was used as a stack pointer.
SPSR_EL2.write(
    SPSR_EL2::D::Masked
        + SPSR_EL2::A::Masked
        + SPSR_EL2::I::Masked
        + SPSR_EL2::F::Masked
        + SPSR_EL2::M::EL1h,
);

// Second, let the link register point to reset().
ELR_EL2.set(reset as *const () as u64);
```

As you can see, we are populating `ELR_EL2` with the address of the [reset()](raspi3_boot/src/lib.rs#L51) function that we earlier used to call directly from the entrypoint.

Finally, we set the stack pointer for `SP_EL1` and call `ERET`:

```rust
// Set up SP_EL1 (stack pointer), which will be used by EL1 once
// we "return" to it.
SP_EL1.set(STACK_START);

// Use `eret` to "return" to EL1. This will result in execution of
// `reset()` in EL1.
asm::eret()
```

## Are we stackless?

We just wrote a big rust function, `setup_and_enter_el1_from_el2()`, that is executed in a context where we
do not have a stack yet. We should double-check the generated machine code:

```console
ferris@box:~$ make objdump
cargo objdump --target aarch64-unknown-none-softfloat -- -disassemble -print-imm-hex kernel8

kernel8:	file format ELF64-aarch64-little

Disassembly of section .text:
raspi3_boot::setup_and_enter_el1_from_el2::hf5d23e5bead7ee4e:
   808bc:	e8 03 1f aa 	mov	x8, xzr
   808c0:	e9 07 00 32 	orr	w9, wzr, #0x3
   808c4:	09 e1 1c d5 	msr	CNTHCTL_EL2, x9
   808c8:	68 e0 1c d5 	msr	CNTVOFF_EL2, x8
   808cc:	08 00 00 90 	adrp	x8, #0x0
   808d0:	ea 03 01 32 	orr	w10, wzr, #0x80000000
   808d4:	0a 11 1c d5 	msr	HCR_EL2, x10
   808d8:	ab 78 80 52 	mov	w11, #0x3c5
   808dc:	0b 40 1c d5 	msr	SPSR_EL2, x11
   808e0:	ec 03 0d 32 	orr	w12, wzr, #0x80000
   808e4:	08 21 22 91 	add	x8, x8, #0x888
   808e8:	28 40 1c d5 	msr	ELR_EL2, x8
   808ec:	0c 41 1c d5 	msr	SP_EL1, x12
   808f0:	e0 03 9f d6 	eret
```

Looks good! Thanks zero-overhead abstractions in the [cortex-a](https://github.com/rust-embedded/cortex-a) crate! :heart_eyes:

## Testing

In `main.rs`, we added some tests to see if access to the counter timer registers is actually working, and if the mask bits in `SPSR_EL2` made it to `EL1` as well:

```console
ferris@box:~$ make raspboot

[0] UART is live!
[1] Press a key to continue booting... Greetings fellow Rustacean!
[i] Executing in EL: 1

Testing EL1 access to timer registers:
    Delaying for 3 seconds now.
    1..2..3
    Works!

Checking interrupt mask bits:
    D: Masked.
    A: Masked.
    I: Masked.
    F: Masked.
```
