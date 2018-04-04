# Tutorial 02 - Multicore Rust

Now let's try something more complex, shall we? By complex I mean stopping the
CPU cores just like in the first tutorial, but this time stop one of them from
**Rust**!

## Glue code

In order to conveniently incorporate Rust code, we are restructuring our crate a
bit.

We reuse a lot of steps that are explained in great detail in [The
Embedonomicon][nom], so please take your time and read up on it. Afterwards, you
can compare to the files in this crate and see what we actually kept to get our
Raspberry Pi 3 tutorial going. Here's a short summary of the new structure of
the crate:

  - `raspi3_glue/`: The extern crate containing glue code as presented in the
    Embedonomicon.
    - In a small deviation to the Embedonomicon, `lib.rs` also includes
      `boot_cores.S` from the previous tutorial, still with the
      [global_asm!][gasm] macro.
    - Therefore, `boot_cores.S` has been moved into `raspi3_glue/src/`.
  - `src`: Source code of our actual Rust code, currently only containing
    `main.rs` executing an endless loop.

[nom]: https://japaric.github.io/embedonomicon/
[gasm]: https://doc.rust-lang.org/unstable-book/language-features/global-asm.html

### Changes to `boot_cores.S`

In contrast to the previous tutorial, we are now [distinguishing the
cores][dist]. To do so, we read the [mpidr_el1][mpdir] system register. If it is
not zero, we enter the former infinite waiting loop, aka stopping the respective
CPU core.

 If the result of the read from `mpidr_el1` is zero, which means we are
 executing on core0, we set up the stack for that core, and afterwards call the
 Rust `reset()` function of the glue code in `raspi3_glue/src/lib.rs`. In case
 the Rust code returns (which it never should), we also jump to the same
 infinite loop the other CPU cores are running.

 The Rust `reset()`, in turn, will then zero-out the `bss section` (the next
 section explains what that is) and finally call our `main()` function from
 `main.rs`.

[dist]: http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.den0024a/CFHCIDCH.html
[mpdir]: http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0500g/BABHBJCI.html

## Changes to `link.ld`

Since we are using a high-level language now in the form of Rust, we also take
precautions to have eventual space reserved in memory for the [bss
segment][bss], which is needed in case zero-initialized static variables are
allocated in the Rust code.

[bss]: https://en.wikipedia.org/wiki/.bss

Therefore, we added the `bss` segment to the linker script and export its
properties via `__bss_start` and `__bss_size`, which will be picked up and
zeroed out by the glue code in `raspi3_glue/src/lib.rs`.

Additionally, there is a [data segment][data] now.

[data]: https://en.wikipedia.org/wiki/Data_segment

Finally, we need to take care that we still start the text segment with the
assembly code and not the newly added Rust code. This is taken care of by
placing the `.text.boot` section before all other new text sections
`KEEP(*(.text.boot)) *(.text .text.* ...`.

This way, the assembly stays at the `0x80_000` address, which is the entry point
of the RPi3 CPU.

## Changes to `Makefile`

We've added one more target:
- [clippy] is Rust's linter, and can give you useful advise to improve your
  code. Invoke with `make clippy`.

[clippy]: https://github.com/rust-lang-nursery/rust-clippy

From now on, we can use the same Makefile for every tutorial, regardless of the
number of Rust sources, and we won't discuss it any further.

## main.rs

Finally, our first Rust code. Just an empty loop, but still! :-)
