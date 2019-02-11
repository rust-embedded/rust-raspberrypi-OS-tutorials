# Tutorial 0C - Virtual Memory

**This is a stub**

TODO: Write rest of tutorial.

Virtual memory is an immensely complex, but exciting topic. In this first
lesson, we start slow and switch on the MMU using handcrafted page tables for
the first `1 GiB` of memory. That is the amount of `DRAM` the usual Raspberry Pi
3 has. As we already know, the upper `16 MiB` of this gigabyte-window are
occupied by the Raspberry's peripherals such as the UART.

The page tables we install alternate between `2 MiB` blocks and `4 KiB` blocks.

The first `2 MiB` of memory are identity mapped, and therefore contain our code
and the stack. We use a single table with a `4 KiB` granule to differentiate
between code, RO-data and RW-data. The linker script was adapted to adhere to
the pagetable sizes.

Next, we map the UART into the second `2 MiB` block to show the effects of
virtual memory.

Everyting else is, for reasons of convenience, again identity mapped using `2
MiB` blocks.

Hopefully, in a later tutorial, we will write or use (e.g. from the `cortex-a`
crate) proper modules for page table handling, that, among others, cover topics
such as using recursive mapping for maintenace.

## Zero-cost abstraction

The MMU init code is a good example to see the great potential of Rust's
zero-cost abstractions[[1]](https://blog.rust-lang.org/2015/05/11/traits.html)[[2]](https://ruudvanasseldonk.com/2016/11/30/zero-cost-abstractions) for embedded programming.

Take this piece of code for setting up the `MAIR_EL1` register using the
[cortex-a](https://crates.io/crates/cortex-a) crate:



```rust
// First, define the two memory types that we will map. Normal DRAM type and
// device.
MAIR_EL1.write(
    // Attribute 1
    MAIR_EL1::Attr1_HIGH::Device
        + MAIR_EL1::Attr1_LOW_DEVICE::Device_nGnRE
        // Attribute 0
        + MAIR_EL1::Attr0_HIGH::Memory_OuterWriteBack_NonTransient_ReadAlloc_WriteAlloc
        + MAIR_EL1::Attr0_LOW_MEMORY::InnerWriteBack_NonTransient_ReadAlloc_WriteAlloc,
);
```

This piece of code is super expressive, and it makes us of `traits`, different
`types` and `constants` to provide type-safe register manipulation.

In the end, this code sets the first four bytes of the register to certain
values according to the data sheet. Looking at the generated code, we can see
that despite all the type-safety and abstractions, we get super lean code:

```text
kernel8::mmu::init::h53df3fab6e51e098:
   ...
   80768:       ed 9f 80 52     mov     w13, #0x4ff
   ...
   80778:       0d a2 18 d5     msr     MAIR_EL1, x13
   ...
```

## Output

```console
ferris@box:~$ make raspboot

[0] UART is live!
[1] Press a key to continue booting... Greetings fellow Rustacean!
[i] MMU: 4 KiB granule supported!
[i] MMU: Up to 40 Bit physical address range supported!
[2] MMU online.

Writing through the virtual mapping at 0x00000000001FF000.

```
