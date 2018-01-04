Tutorial 04 - Mailboxes
=======================

Before we could go on with UART0, we need mailboxes. So in this tutorial we introduce the mailbox interface.
We'll use it to query the board's serial number and print that out on UART1.
NOTE: qemu does not redirect UART1 to terminal by default, only UART0!

Uart.h, uart.c
--------------

`uart_hex(d)` prints out a binary value in hexadecimal format.

Mbox.h, mbox.c
--------------

The mailbox interface. First we fill up the message in `mbox` array, then we call
`mbox_call(ch)` to pass it to the GPU, specifying the mailbox channel.
In this example we have used the [property channel](https://github.com/raspberrypi/firmware/wiki/Mailbox-property-interface),
which requires the message to be formatted as:
 0. size of the message in bytes, (x+1)*4
 1. MBOX_REQUEST magic value, indicates request message
 2-x. tags
 x+1. MBOX_TAG_LAST magic value, indicates no more tags

Where each tag looks like:
 n+0. tag identifier
 n+1. value buffer size in bytes
 n+2. must be zero
 n+3. optional value buffer

Main
----

We query the board's serial number and then we display it on the serial console.
