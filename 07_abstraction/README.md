# Tutorial 07 - Abstraction

This is a short one regarding code changes, but has lots of text because two
important Rust principles are introduced: Abstraction and modularity.

From a functional perspective, this tutorial is the same as `05_uart0`, but with
the key difference that we threw out all manually crafted assembler. Both the
main and the glue crate do not use `#![feature(global_asm)]` or
`#![feature(asm)]` anymore. Instead, we pulled in the [cortex-a][crate] crate,
which now provides `cortex-a` specific features like register access or safe
wrappers around assembly instructions.

[crate]: https://github.com/andre-richter/cortex-a

For single assembler instructions, we now have the `cortex-a::asm` namespace,
e.g. providing `asm::nop()`.

For registers, there is `cortex-a::regs`. The interface is the same as we have
it for MMIO accesses, aka provided by [register-rs][register-rs] and therefore
based on [tock-regs][tock-regs]. For registers like the stack pointer, which are
generally read and written as a whole, there's the common [get()][get] and
[set()][set] functions which take and return primitive integer types.

[register-rs]: https://github.com/rust-osdev/register-rs
[tock-regs]: https://github.com/tock/tock/tree/master/libraries/tock-register-interface
[get]: https://docs.rs/cortex-a/1.0.0/cortex_a/regs/sp/trait.RegisterReadWrite.html#tymethod.get
[set]: https://docs.rs/cortex-a/1.0.0/cortex_a/regs/sp/trait.RegisterReadWrite.html#tymethod.set

Registers that are divided into multiple fields, like `MPIDR_EL1` ([see the ARM
Reference Manual][el1]), on the other hand, are backed by a respective
[type][cntp_type] definition that allow for fine-grained reading and
modifications.

[el1]: http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0500g/BABHBJCI.html
[cntp_bitfields]: https://docs.rs/cortex-a/1.0.0/cortex_a/regs/cntp_ctl_el0/CNTP_CTL_EL0/index.html

The register API is based on the [tock project's][tock] register
interface. Please see [their homepage][tock_registers] for all the details.

[tock]: https://github.com/tock/tock
[tock_register]: https://github.com/tock/tock/tree/master/libraries/tock-register-interface

To some extent, this namespacing also makes our code more portable. For example,
if we want to reuse parts of it on another processor architecture, we could pull
in the respective crate and change our use-clause from `use cortex-a::asm` to
`use new_architecture::asm`. Of course this also demands that both crates adhere
to a common set of wrappers that provide the same functionality. Assembler and
register instructions like we use them here are actually a weak example. Where
this modular approach can really pay off is for common peripherals like timers
or memory management units, where implementations differ between processors, but
usage is often the same (e.g. setting a timer for x amount of microseconds).

In Rust, we have the [Embedded Devices Working
Group](https://github.com/rust-lang-nursery/embedded-wg), which among other
goals, tries to establish a common set of wrapper- and interface-crates that
introduce abstraction on different levels of the system. Check out the [Awesome
Embedded Rust](https://github.com/rust-embedded/awesome-embedded-rust) list for
an overview.

## Glue Code

Like mentioned above, we threw out the `boot_cores.S` assembler file and
replaced it with a Rust function. Why? Because we can, for the fun of it.

```rust
#[link_section = ".text.boot"]
#[no_mangle]
pub extern "C" fn _boot_cores() -> ! {
    use cortex_a::{asm, regs::mpidr_el1::*, regs::sp::*};

    match MPIDR_EL1.get() & 0x3 {
        0 => {
            SP.set(0x80_000);
            unsafe { reset() }
        }
        _ => loop {
            // if not core0, infinitely wait for events
            asm::wfe();
        },
    }
}
```

Since this is the first code that the RPi3 will execute, the stack has not been
set up yet. Actually it is this function that will do it for the first
time. Therefore, it is important to check that code generated from this function
does not call any subroutines that need a working stack themselves.

The `get()` and `asm` wrappers that we use from the `cortex-a` crate are all
inlined, so we fulfill this requirement. The compilation result of this function
should yield something like the following, where you can see that the stack
pointer is not used apart from ourselves setting it.

```bash
[andre:/work] $ cargo objdump --target aarch64-raspi3-none-elf.json -- -disassemble -print-imm-hex kernel8

[...] (Some output omitted)

_boot_cores:
   80000:       a8 00 38 d5     mrs     x8, MPIDR_EL1
   80004:       1f 05 40 f2     tst     x8, #0x3
   80008:       60 00 00 54     b.eq    #0xc
   8000c:       5f 20 03 d5     wfe
   80010:       ff ff ff 17     b       #-0x4
   80014:       e8 03 0d 32     orr     w8, wzr, #0x80000
   80018:       1f 01 00 91     mov     sp, x8
   8001c:       5d 01 00 94     bl      #0x574
```

It is important to always manually check this, and not blindly rely on the
compiler.

## Test it

Since this is the first tutorial after we've written our own bootloader over
serial, you can now for the first time test this convenient interface:

```bash
make raspboot
```
