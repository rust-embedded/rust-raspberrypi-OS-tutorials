# Tutorial 0F - Globals, Synchronization and `println!`

Until now, we use a rather inelegant way of printing messages: We are directly
calling the `UART` device driver's functions for putting and receiving
characters on the serial line, e.g. `uart.puts()`. Also, we have only very
bare-bones implementations for printing hex or decimal integers. This both looks
ugly in the code, and is not very flexible. For example, if at some point we
decide to replace the `UART` as the output device, we have to manually find and
replace all the respective calls, and need to take care that we do not use the
device before it was probed or after it was shut down.

Hence, it is time to get some elegant format-string-based printing going, like
we know it from other languages, e.g. `C`'s `printf()`, and introduce an
abstraction layer that allows us to decouple printing functions from the actual
output device.

On this occasion, we will also learn important lessons about about **mutable
global variables**, which are called **static variables** in Rust, get to know
**trait objects** and hear about Rust's concept of **interior mutability**.

## The Virtual Console

First, we introduce a `Console` type in `src/devics/virt/console.rs`:

```rust
pub struct Console {
    output: Output,
}
```

When everything is finished, this type will be used as a `virtual device` that
forwards calls to printing functions to the currently active output device.

### Code Restructuring

In case you wonder about the path: The introduction of the first `virtual
device` in our code was a good opportunity to introduce a better structure for
our modules. Basically, we differentiate between real (HW) and virtual devices
now:

```console
src
├── devices
│   ├── hw
│   │   ├── gpio.rs
│   │   ├── uart.rs
│   │   └── videocore_mbox.rs
│   ├── hw.rs
│   ├── virt
│   │   └── console.rs
│   └── virt.rs
├── devices.rs
```

### Console Implementation

The `Console` type has a single field of type `Output`:

```rust
/// Possible outputs which the console can store.
pub enum Output {
    None(NullConsole),
    Uart(hw::Uart),
}
```

How will it be used? Let us have a look:

```rust
impl Console {
    pub const fn new() -> Console {
        Console {
            output: Output::None(NullConsole {}),
        }
    }

    #[inline(always)]
    fn current_ptr(&self) -> &dyn ConsoleOps {
        match &self.output {
            Output::None(i) => i,
            Output::Uart(i) => i,
        }
    }

    /// Overwrite the current output. The old output will go out of scope and
    /// it's Drop function will be called.
    pub fn replace_with(&mut self, x: Output) {
        self.current_ptr().flush();

        self.output = x;
    }
```

Basically two things can be done.

1. `output` can be replaced during runtime.
2. Using `current_ptr()`, a reference to the current `output` is returned as a
   [trait object](https://doc.rust-lang.org/edition-guide/rust-2018/trait-system/dyn-trait-for-trait-objects.html)
   that implements the `ConsoleOps` trait. Hence, for the first time in the
   tutorials, Rust's [dynamic dispatch](https://doc.rust-lang.org/book/ch17-02-trait-objects.html#trait-objects-perform-dynamic-dispatch)
   is used.

So what does the `ConsoleOps` trait define?

```rust
pub trait ConsoleOps: Drop {
    fn putc(&self, c: char) {}
    fn puts(&self, string: &str) {}
    fn getc(&self) -> char {
        ' '
    }
    fn flush(&self) {}
}
```

All in all, it is basically the same that is already present in the `UART`
driver: Reading and writing a single character, and writing a whole string. What
is new is the `flush` function, which is meant for devices that implement output
FIFOs.

So any device that can be stored into `output` must implement this trait,
otherwise a compile-time error would occur.

### Dispatching to the Current Output

In order to use the `Console` as a HW-agnostic device for printing, some
dispatching code is needed. Therefore, it implements the `ConsoleOps` trait
itself, and forwards the trait calls during run-time to whatever is stored in
`output`.

```rust
/// Dispatch the respective function to the currently stored output device.
impl ConsoleOps for Console {
    fn putc(&self, c: char) {
        self.current_ptr().putc(c);
    }

    fn puts(&self, string: &str) {
        self.current_ptr().puts(string);
    }

    fn getc(&self) -> char {
        self.current_ptr().getc()
    }

    fn flush(&self) {
        self.current_ptr().flush()
    }
}
```

Congratulations :tada:.

This is not much code, but enough so that you've implemented your first, very
basic kind of [Hardware Abstraction Layer (HAL)](https://en.wikipedia.org/wiki/Hardware_abstraction).

## Making it Static (and Mutable)

Now we need an instance of the virtual console in form of a _static variable_
(remember, this is Rust speak for global) to make our life easier and our code
less bloated. Doing so enables calls to printing functions from every place in
the code, without dragging along references to the console everywhere.

At times, we also want to replace the `output` field of our console variable, so we
need a `mutable` static.

In system programming languages like `C` or `C++`, this would be quite easy. For
example, the declaration below is enough to allow mutation of `console`, since
the language does not have a built-in concept of mutable and immutable types:

```C++
Console console = Console::Console();

int kernel_entry() {
    console.replace_with(...)
}
```

However, in Rust, if you do

```rust
static mut CONSOLE: devices::virt::Console =
    devices::virt::Console::new();

fn kernel_entry() -> ! {
    CONSOLE.replace_with(...) // <-- Compiler: "Where's my unsafe{}?!!"
}
```

the compiler will shout angrily at you whenever you try to use `CONSOLE` that
this is unsafe code, and frankly, that is a good thing.

In contrast to the C-family of languages, Rust is from the ground up designed
with multi-core and multi-threading in mind. Thanks to the **borrow-checker**,
Rust ensures that in safe code, there can ever only exist a single mutable
reference to a variable.

This way, it is ensured at compile time that no situations are created where
code that might execute concurrently (that is, for example, code running at the
same time on different physical processor cores) fiddles with the same data
or resources in an unsychronized way.

By instantiating a **mutable** static variable, we allow all code from every
source-code file to easily operate on this mutable reference. Since the variable
is not instantiated at runtime and explicitly passed on in function calls, it is
not possible for the borrow-checker to draw any conclusions about the number of
mutable references in use. As a result, access to mutable statics needs to be
marked with `unsafe{}` in any case in Rust.

So how can we make this safe again? What we need in this case is a
**synchronization primitive**. You've probably heard of them
before. **Spinlocks** and **mutexes** are two examples. What they do is to
ensure _at runtime_ that there is no concurrent access to the data they protect.

### How to Build a Synchronization Primitive in Rust

In contrast to mutable statics, **immutable statics** are considered safe by
Rust as long as they are marked
[Sync](https://doc.rust-lang.org/std/marker/trait.Sync.html). It is perfectly
fine to share an infinite number of references to them. So here is the strategy:

1. Build a wrapper type that can be instantiated as an **immutable static** and
   that encapsulates the actual mutable data.
2. Provide a function that returns a mutable reference to the wrapped type.
3. This function will need to be marked `unsafe`. In order to consider it safe
   nonetheless, it must feature code that ensures at runtime that only a
   single reference is given out at times.

This is the basic concept of all synchronization primitives in Rust. For
educational purposes, in the tutorials, we will roll our own, and not reuse
stuff from the core library or popular crates like [spin](https://crates.io/crates/spin).

### The `NullLock`

The first implementation will actually be very easy. We do not yet have to worry
that a situation arises where (i) code tries to take the lock while it is
already locked or (ii) where there is contention for the lock. This is because
the kernel is still in a state where everything is executed linearly from start
to finish:

1. Asynchronous exceptions like Interrupts are not enabled yet, so there never is
   any interruption in the program flow.
2. We know that we currently do not have any code yet that raises synchronous exceptions.
2. Only a single core is active, all others are parked. Therefore, no concurrent
   execution of code is happening.

> Hint: You will learn about asynchronous and synchronous exceptions in the
> tutorial after the next.

So all that needs be done is wrapping the data and giving back the mutable
reference. Introducing the `NullLock` in `sync.rs`:

```rust
use core::cell::UnsafeCell;

pub struct NullLock<T> {
    data: UnsafeCell<T>,
}

unsafe impl<T> Sync for NullLock<T> {}

impl<T> NullLock<T> {
    pub const fn new(data: T) -> NullLock<T> {
        NullLock {
            data: UnsafeCell::new(data),
        }
    }
}

impl<T> NullLock<T> {
    pub fn lock<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        // In a real lock, there would be code around this line that ensures
        // that this mutable reference will ever only be given out one at a
        // time.
        f(unsafe { &mut *self.data.get() })
    }
}
```

First, the lock type is marked with the `Sync` [marker trait](https://doc.rust-lang.org/std/marker/trait.Sync.html) to tell the
compiler that it is safe to share references to it between threads. More
literature on this topic in [[1]](https://doc.rust-lang.org/beta/nomicon/send-and-sync.html)[[2]](https://doc.rust-lang.org/book/ch16-04-extensible-concurrency-sync-and-send.html).

Second, a `lock()` function is provided which returns mutable references to the
wrapped data in the
[UnsafeCell](https://doc.rust-lang.org/std/cell/struct.UnsafeCell.html). Quoting
from the UnsafeCell documentation:


> The core primitive for interior mutability in Rust.
>
> UnsafeCell<T> is a type that wraps some T and indicates unsafe interior operations on the wrapped type. Types with an UnsafeCell<T> field are considered to have an 'unsafe interior'. The UnsafeCell<T> type is the only legal way to obtain aliasable data that is considered mutable. In general, transmuting an &T type into an &mut T is considered undefined behavior.
>
> [...]
>
> The UnsafeCell API itself is technically very simple: it gives you a raw pointer *mut T to its contents. It is up to you as the abstraction designer to use that raw pointer correctly.

In upcoming tutorials, when the need arises, the `NullLock` will be gradually
extended to provide proper locking using architectural features the RPi3
provides for this case.

### Closures

The Rust standard library and some popular crates for synchronization primitives
use the concept of returning
[RAII](https://en.wikipedia.org/wiki/Resource_acquisition_is_initialization)
type [guards](https://doc.rust-lang.org/std/sync/struct.Mutex.html#method.lock)
that allow usage of the locked data until the guard goes out of scope.

In the author's opinion, RAII guards have the disadvantage that the user must
explicitly scope their lifetime with braces `{}`, which is prone to being
forgotten. This in turn would lead to the lock being held much longer than
needed. For educational purposes, the `lock()` functions in the tutorials will
therefore take [closures](https://doc.rust-lang.org/book/ch13-01-closures.html)
as arguments. They give better visual cues about the parts of the code during
which the lock is held.

Example:

```rust
static CONSOLE: sync::NullLock<devices::virt::Console> =
    sync::NullLock::new(devices::virt::Console::new());

fn kernel_entry() -> ! {

    ...

    CONSOLE.lock(|c| { //
        c.getc();      // Unlocked only inside here
    });                //

    ...
}
```

> Disclaimer: No investigations have been made if using closures results in
> poorer performance. If so, the hit is taken willingly for said educational
> purposes.

## `print!` and `println!`

In `macros.rs`, printing macros from the Rust core library are reused to empower
the kernel with [all the format-string beauty Rust provides](https://doc.rust-lang.org/std/fmt/). The macros eventually call the
function `_print()`, which redirects to the global `CONSOLE` of the kernel (will
be introduced in a minute):

```rust
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;

    crate::CONSOLE.lock(|c| {
        c.write_fmt(args).unwrap();
    })
}
```

To make this work, the virtual console needs to provide an implementation of
`core::fmt::Write`. In this case, it is as easy as forwarding the
macro-formatted string via `self.current_ptr().puts(s)`.

## Stitching it All Together

In `main.rs`, a static `CONSOLE` is defined:

```rust
/// The global console. Output of the print! and println! macros.
static CONSOLE: sync::NullLock<devices::virt::Console> =
    sync::NullLock::new(devices::virt::Console::new());
```

By default, it encapsulates a `NullConsole` output, which, well, does
nothing. This is just a safety measure to ensure that the print macros can be
called any time, even before a real physical output is available. In `main.rs`,
a respective call is made that will never appear as an output anywhere:

```rust
// This will be invisible, because CONSOLE is dispatching to the NullConsole
// at this point in time.
println!("Is there anybody out there?");
```

After initializing the `GPIO` and `VidecoreMbox` drivers, the `UART` is
initialized and replaces the `NullConsole` as the static output:

```rust
match uart.init(&mut v_mbox, &gpio) {
    Ok(_) => {
        CONSOLE.lock(|c| {
            // Moves uart into the global CONSOLE. It is not accessible
            // anymore for the remaining parts of kernel_entry().
            c.replace_with(uart.into());
        });
     println!("\n[0] UART is live!");
    }
```

Here it becomes clear why the virtual console is designed such that it stores an
output _by value_. It is not possible to safely store a reference to something
that is generated at runtime in a static data structure. This is because the
static has `static` lifetime, aka lives forever. Whereas a reference to
something generated during runtime might become invalid at some point in the
future.

Hence, `move semantics` are used to achieve our goal. Once `uart` has moved into
`CONSOLE`, it will live there until it is replaces again. That is also why the
`ConsoleOps` trait demands that its implementors also implement the `Drop`
trait. When calling `CONSOLE.replace()`, the old output will go out of scope,
and hence its drop function will be called. The drop function can then take care
of gracefully shutting down or disabling the device it belongs to.

While the print macros implicitly call the lock function, there are some places
where it is done explicitly. For example, when querying a keystroke from the
user:

```rust
    print!("[1] Press a key to continue booting... ");
    CONSOLE.lock(|c| {
        c.getc();
    });
    println!("Greetings fellow Rustacean!");

```

## Summary

Lots of things happened in this tutorial:
1. The kernel's code was restructured.
2. The virtual console was introduced as a **Hardware Abstraction Layer**.
  1. **Trait objects** and **dynamic dispatch** were used for the first time.
3. The peculiarities of **mutable static variables** were discussed and what role the **Sync marker trait** plays for them.
4. **Synchronization primitives** were introduced and (a special) one was built.
  1. You learned about **UnsafeCell** and its role in providing **interior mutability**.
  2. You read about **Closures** vs. **RAII guards**.
5. And finally, the `print!` and `println!` macros from the core library are now
   usable in the kernel!
