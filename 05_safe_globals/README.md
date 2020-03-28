# Tutorial 05 - Safe Globals

## tl;dr

A pseudo-lock is introduced; It is a first showcase of OS synchronization primitives and enables
safe access to a global data structure.

## Mutable globals in Rust

When we introduced the globally usable `print!` macros in [tutorial 03], we cheated a bit. Calling
`core::fmt`'s `write_fmt()` function, which takes an `&mut self`, was only working because on each
call, a new instance of `QEMUOutput` was created.

If we would want to preserve some state, e.g. statistics about the number of characters written, we
need to make a single global instance of `QEMUOutput` (in Rust, using the `static` keyword).

A `static QEMU_OUTPUT`, however, would not allow to call functions taking `&mut self`. For that, we
would need a `static mut`, but calling functions that mutate state on `static mut`s is unsafe. The
Rust compiler's reasoning for this is that it can then not prevent anymore that multiple
cores/threads are mutating the data concurrently (it is a global, so everyone can reference it from
anywhere. The borrow checker can't help here).

The solution to this problem is to wrap the global into a synchronization primitive. In our case, a
variant of a *MUTual EXclusion* primivite. `Mutex` is introduced as a trait in `synchronization.rs`,
and implemented by the `NullLock` in the same file. In order to make the code lean for teaching
purposes, it leaves out the actual architecture-specific logic for protection against concurrent
access, since we don't need it as long as the kernel only executes on a single core with interrupts
disabled. That is also why it is implemented in the same file as the interface itself. In later
tutorials, an implementation might move to the `_arch` once it pulls in arch-specific code that can
not be further abstracted.

The `NullLock` focuses on showcasing the Rust core concept of [interior mutability]. Make sure to
read up on it. I also recommend to read this article about an [accurate mental model for Rust's
reference types].

If you want to compare the `NullLock` to some real-world mutex implementations, you can check out
implemntations in the [spin crate] or the [parking lot crate].

[tutorial 03]: ../03_hacky_hello_world
[interior mutability]: https://doc.rust-lang.org/std/cell/index.html
[accurate mental model for Rust's reference types]: https://docs.rs/dtolnay/0.0.6/dtolnay/macro._02__reference_types.html
[spin crate]: https://github.com/mvdnes/spin-rs
[parking lot crate]: https://github.com/Amanieu/parking_lot

## Test it

```console
$ make qemu
[...]
[0] Hello from pure Rust!
[1] Chars written: 27
[2] Stopping here.
```

## Diff to previous
