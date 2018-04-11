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

For registers, there is `cortex-a::register`. For registers like the stack
pointer, which are generally read and written as a whole, there's simple
[read()][sp_read] and [write()][sp_write] functions which take and return
primitive integer types.

[sp_read]: https://docs.rs/cortex-a/0.1.2/cortex_a/register/sp/fn.read.html
[sp_write]: https://docs.rs/cortex-a/0.1.2/cortex_a/register/sp/fn.write.html

Registers that are divided into multiple fields, e.g. `MPIDR_EL1` ([see the ARM
Reference Manual][el1]), on the other hand, are abstracted into their [own
types][mpidr_type] and offer getter and/or setter methods, respectively.

[el1]: http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0500g/BABHBJCI.html
[mpidr_type]:https://docs.rs/cortex-a/0.1.2/cortex_a/register/mpidr_el1/struct.MPIDR_EL1.html

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
    match register::mpidr_el1::read().core_id() {
        0 => unsafe {
            register::sp::write(0x80_000);
            reset()
        },
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

The `register` and `asm` wrappers that we use from the `cortex-a` crate are all
inlined, so we fulfill this requirement. The compilation result of this function
should yield something like the following, where you can see that the stack
pointer is not used apart from ourselves setting it.

```bash
./dockcross-linux-aarch64 bash
[andre:/work] $ aarch64-linux-gnu-objdump -CD kernel8

[...] (Some output omitted)

0000000000080000 <_boot_cores>:
   80000:       d53800a8        mrs     x8, mpidr_el1
   80004:       f240051f        tst     x8, #0x3
   80008:       54000060        b.eq    80014 <_boot_cores+0x14>
   8000c:       d503205f        wfe
   80010:       17ffffff        b       8000c <_boot_cores+0xc>
   80014:       320d03e8        orr     w8, wzr, #0x80000
   80018:       9100011f        mov     sp, x8
   8001c:       9400016b        bl      805c8 <raspi3_glue::reset::h2a7ad49cd9d2154d>
```

It is important to always manually check this, and not blindly rely on the
compiler.

## Test it

Since this is the first tutorial after we've written our own bootloader over
serial, you can now for the first time test this convenient interface:

```bash
make raspboot
```
