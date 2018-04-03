# Tutorial 02 - Multicore Rust

Now let's try something more complex, shall we? By complex I mean stopping the
CPU cores just like in the first tutorial, but this time stop one of them from
**Rust**!

## Glue code

In order to incorporate Rust code, we are setting up a binary crate and add
`#![no_std]` to `main.rs`, since we are writing our own bare metal code and do
not want to rely on the standard library.

However, a lot of steps are needed to make a `no_std` crate build. All of this
and even more is explained in detail in [The Embedonomicon][nom], so please take
your time and read up on it. Afterwards, you can compare to the files in this
crate and see what we actually kept to get our Raspberry Pi 3 tutorial
going. Here's a short summary of the crate's structure:

  - `raspi3_glue/`: The extern crate containing glue code as presented in the
    Embedonomicon.
    - In a small deviation to the Embedonomicon, `lib.rs` also includes
      `_boot_cores.S` from the previous tutorial via the [global_asm!][gasm]
      macro.
    - Therefore, `_boot_cores.S` has been moved into `raspi3_glue/src/`.
  - `src`: Source code of our actual crate, currently only containing `main.rs`
    executing an endless loop.

[nom]: https://japaric.github.io/embedonomicon/
[gasm]: https://doc.rust-lang.org/unstable-book/language-features/global-asm.html

### Changes to `_boot_cores.S`

In contrast to the previous tutorial, we now we have to [distinguish the
 cores][dist]. To do so, we read the [mpidr_el1][mpdir] system register. If it
 is not zero, we'll do the former infinite loop. If it is zero, aka we are
 executing on core0, then we'll call the Rust `reset()` function. For that, we
 also need a proper stack, and have space reserved in memory for the [bss
 segment][bss].

We added the `bss` segment to the linker script and export its properties via
`__bss_start` and `__bss_size`, which will be picked up and zeroed out by the
glue code in `raspi3_glue/src/lib.rs`. Additionally, we set the stack in the
Assembly in `_boot_cores.S`, and then finally call the `reset()` function of the
glue code, which in turn calls `main()` after zeroing `bss`. In case the Rust
code returns (which it never should not), we also jump to the same infinite loop
the other CPU cores running.

[dist]: http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.den0024a/CFHCIDCH.html
[mpdir]: http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0500g/BABHBJCI.html
[bss]: https://en.wikipedia.org/wiki/.bss

## Changes to `Makefile`

It became a bit trickier. We've added more targets:
- `kernel8.img` now depends on `kernel8`, which compiles the crate either in
  release or debug mode. For the latter, add `DEBUG=1` before invoking make,
  e.g. `DEBUG=1 make`
- [clippy] is Rust's linter, and can give you useful advise to improve your
  code. Invoke with `make clippy`.

[clippy]: https://github.com/rust-lang-nursery/rust-clippy

From now on, we can use the same Makefile for every tutorial, regardless of the
number of Rust sources, and we won't discuss it any further.

## Changes to `link.ld`

Apart from the added bss section, it is important to start the text segment with
the Assembly code and not the Rust code, because we set the stack right before
it, hence the `KEEP()`. This way, the assembly stays at 0x80000, the same as
`_boot_cores` label and stack top.

## main.rs

Finally, our first Rust code. Just an empty loop, but still! :-)
