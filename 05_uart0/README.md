# Tutorial 05 - UART0, PL011

Finally, we can set up `UART0` thanks to the mailbox interface.  This tutorial
produces the same output as tutorial 04, but it prints the serial number on
`UART0`.

## uart.rs

In the init function, we use the mailbox to set a base clock for the UART:

```rust
mbox.buffer[0] = 9 * 4;
mbox.buffer[1] = mbox::REQUEST;
mbox.buffer[2] = mbox::tag::SETCLKRATE;
mbox.buffer[3] = 12;
mbox.buffer[4] = 8;
mbox.buffer[5] = mbox::clock::UART; // UART clock
mbox.buffer[6] = 4_000_000; // 4Mhz
mbox.buffer[7] = 0; // skip turbo setting
mbox.buffer[8] = mbox::tag::LAST;

// Insert a compiler fence that ensures that all stores to the
// mbox buffer are finished before the GPU is signaled (which
// is done by a store operation as well).
compiler_fence(Ordering::Release);

if mbox.call(mbox::channel::PROP).is_err() {
    return Err(UartError::MailboxError); // Abort if UART clocks couldn't be set
};

```

Afterwards, we can program the rate divisors:

```rust
self.IBRD.write(IBRD::IBRD.val(2)); // Results in 115200 baud
self.FBRD.write(FBRD::FBRD.val(0xB));
```

Baud rate calculation won't be covered in detail here.  Please see [this
reference from ARM](http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0183g/I49493.html)
for details.

The API for using the UART is identical to the `UART1` API.

## main.rs

We query the board's serial number and display it on the serial console.
