# Tutorial 01 - Bare Minimum

Okay, we're not going to do much here, just test our toolchain. The resulting
kernel8.img should boot on the Raspberry Pi 3, and stop all CPU cores in an
infinite waiting loop. You can check that by running

```bash
$ make qemu
... some output removed for clearity: ...
----------------
IN:
0x00080000:  d503205f  wfe
0x00080004:  17ffffff  b        #0x80000
```

## Crate setup

In this tutorial, we are compiling a kernel that is in the end only executing a
single assembly instruction which we program with an assembly file.

However, since we want to use the toolchain that is delivered with `rustup` as
much as possible, we are already setting up a Rust crate. This allows us to use
`rustc` and LLVM's `lld.ld` linker to process our assembly file.

## main.rs

We define the crate to not use the standard library (`#![no_std]`), indicate
that it does not have a main function via `#![no_main]`, and also define a stub
for the `panic_fmt()` handler, which is a requirement for `no_std` crates.

In th end, we (mis)use `main.rs` as a wrapper to process our assembly file via
`rustc`. The assembly file iself is included with the [global_asm!()][gasm]
macro.

[gasm]: https://doc.rust-lang.org/unstable-book/language-features/global-asm.html

## boot_cores.S

When the control is passed to kernel8.img, the environment is not ready yet for
Rust. Therefore we must implement a small preamble in assembly, no Rust for now.

All we do is executing [wfe][wfe], an instruction that puts the CPU cores to
sleep until an asynchronous event occurs. If that happens, we jump right back to
`wfe` again.

[wfe]: http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0360e/CHDBGCFH.html

Note that the CPU has 4 cores. All of them will execute the same infinite loop
for now.

## aarch64-raspi3-none-elf.json

This is our custom target definition of the RPi3 for [Xargo][xargo]. It also
includes a directive to use the `link.ld` linker script.

```json
"pre-link-args": {
    "ld.lld": [
        "--script=link.ld"
    ]
},
```

[xargo]: https://github.com/japaric/xargo

## Makefile

Our Makefile has a few useful targets:
- `kernel8` compiles the crate either in release or debug mode. For the latter,
  add `DEBUG=1` before invoking make, e.g. `DEBUG=1 make`
- `kernel8.img` uses our cross-toolchain's `objcopy` in the docker container to
  generate our kernel binary. Citing the [binutils documentation][butils]:
    - "_When objcopy generates a raw binary file, it will essentially produce a
      memory dump of the contents of the input object file. All symbols and
      relocation information will be discarded. The memory dump will start at
      the load address of the lowest section copied into the output file._"
- `qemu` loads our kernel into an emulated RPi3, and shows as output the
  assembler blocks that are executed. This happens in another docker container.

[butils]: https://sourceware.org/binutils/docs/binutils/objcopy.html

## Linker script `link.ld`

We just set the base address where our kernel8.img will be loaded, and we put
the only section we have there, which is `.text.boot`. Important note, for
AArch64 the load address is **0x80_000**, and not **0x80_00** as with AArch32.
