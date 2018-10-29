# Tutorial 04 - Mailboxes

The Raspberry Pi 3 also has a more powerful UART, `UART0`, that among other
features, supports programmable clock rates.  Before we can go on with setting
up `UART0`, we need mailboxes. Mailboxes are an interface between the Pi's ARM
CPU cores and the GPU. They will be used as a means to request work from the
GPU, for example, requesting to program a certain clock for `UART0`.

In this tutorial, we'll start slowly and use it to query the Raspberry's serial
number and print that out on the already functional `UART1`.

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

### Synchronization

When signaling the GPU about a new mailbox message, we need to take care that
mailbox buffer setup has really finished. Both setting up mailbox contents and
signaling the GPU is done with store operations to independent memory locations
(RAM and MMIO). Since compilers are free to reorder instructions without
control-flow or data-dependencies for optimization purposes, we need to take
care that signaling the GPU really takes place _after_ all of the contents have
been written to the mailbox buffer.

One way to do this would be to define the whole mailbox buffer as `volatile`, as
well as the location that we write to to signal the GPU. The compiler is not
allowed to reorder memory operations tagged with the `volatile` keyword with
each other. But this is not needed here. We don't care if the compiler optimizes
the buffer setup code as long as signaling the GPU takes place afterwards.

Therefore, we prevent premature signaling by inserting an explicit [compiler
fence] after the buffer preparation code. Since we signal the CPU by calling
another function, the fence would only be effective if that function was a)
inlined and b) the inlined instructions then reordered with buffer setup
code. Otherwise the compiler has to assume that the called function has
dependencies on previous memory operations and not reorder here. Although there
is little chance that the reordering scenario happens, I'll leave the fence
there nonetheless for academic purposes :-)

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

## main.rs

We query the board's serial number and then we display it on the UART.
