# Tutorial 04 - Mailboxes

Before we could go on with UART0, we need mailboxes. So in this tutorial we
introduce the mailbox interface.  We'll use it to query the board's serial
number and print that out on UART1.

NOTE: qemu does not redirect UART1 to terminal by default, only UART0!

## uart.rs

`MiniUart::hex(&self, d: u32)` prints out a binary value in hexadecimal format.

## mbox.rs

The mailbox interface. First we fill up the message in the `mbox.buffer` array,
then we call `Mbox::call(&mut self, channel: u32)` to pass it to the GPU,
specifying the mailbox channel. In this example we have used the [property
channel], which requires the message to be formatted as:

[property channel]: (https://github.com/raspberrypi/firmware/wiki/Mailbox-property-interface)

```
 0. size of the message in bytes, (x+1)*4
 1. mbox::REQUEST magic value, indicates request message
 2-x. tags
 x+1. mbox::tag::LAST magic value, indicates no more tags
```

Where each tag looks like:

```
 n+0. tag identifier
 n+1. value buffer size in bytes
 n+2. must be zero
 n+3. optional value buffer
```

### rlibc

The mailbox buffer is a fixed array that is zero-initialized. To achieve
zero-initialization, Rust utilizies and links to the `memset()` function, which
is normally provided by `libc`.

Since we are writing a `no_std` crate, we need to explicitly provide it. The
easiest way is pulling in [rlibc] by adding it as an `extern crate` to `main.rs`
and adding the dependency to `Cargo.toml`.

[rlibc]: https://github.com/alexcrichton/rlibc

### Synchronization

When signaling the GPU about a new mailbox message, we need to take care that
mailbox buffer setup has really finished. Both setting up mailbox contents and
signaling the GPU is done with store operations to memory (RAM and MMIO). There
is an unlikely chance that the compiler reorders instructions, resulting in
signaling the GPU _before_ all of the contents have been written to the mailbox
buffer. We prevent this by inserting a [compiler fence].

Please note that such reordering might also be done by CPUs that feature
[out-of-order execution].  Lucky us, although the Rasperry Pi 3 features
`ARMv8.0-A` CPU cores, the `Cortex-A53` variant is used, [which does not support
this feature].  Otherwise, a [fence] that additionally [emits corresponding CPU
instructions] to prevent this behavior would be needed.

[compiler fence]: https://doc.rust-lang.org/beta/core/sync/atomic/fn.compiler_fence.html
[out-of-order execution]: https://en.wikipedia.org/wiki/Out-of-order_execution
[which does not support this feature]: https://en.wikipedia.org/wiki/Comparison_of_ARMv8-A_cores
[fence]: https://doc.rust-lang.org/std/sync/atomic/fn.fence.html
[emits corresponding CPU instructions]: https://developer.arm.com/products/architecture/a-profile/docs/100941/latest/barriers

## Main

We query the board's serial number and then we display it on the serial console.
