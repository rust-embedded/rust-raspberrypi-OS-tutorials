# Tutorial 03 - Hacky Hello World

## tl;dr

Introducing global `print!()` macros to enable "printf debugging" at the earliest; To keep tutorial
length reasonable, printing functions for now "abuse" a QEMU property that lets us use the RPi's
`UART` without setting it up properly; Using  the real hardware `UART` is enabled step-by-step in
following tutorials.

## Notable additions

- `src/console.rs` introduces interface `Traits` for console commands.
- `src/bsp/rpi.rs` implements the interface for QEMU's emulated UART.
- The panic handler makes use of the new `print!()` to display user error messages.

## Test it

QEMU is no longer running in assembly mode. It will from now on show the output of the `console`.

```console
$ make qemu
[...]
Hello from Rust!

Kernel panic: Stopping here.
```

## Diff to previous
