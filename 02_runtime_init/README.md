# Tutorial 02 - Runtime Init

## tl;dr

We extend `cpu.S` to call into Rust code for the first time. There,we zero the [bss] section before
execution is halted with a call to `panic()`. Check out `make qemu` again to see the additional code
run.

## Notable additions

- More sections in linker script:
     - `.rodata`, `.data`
     - `.bss`
- `_start()`:
     - Halt core if core != `core0`.
     - `core0` jumps to the `runtime_init()` Rust function.
- `runtime_init()` in `runtime_init.rs`
     - Zeros the `.bss` section.
     - Calls `kernel_init()`, which calls `panic!()`, which eventually halts `core0` as well.

[bss]: https://en.wikipedia.org/wiki/.bss

## Diff to previous
