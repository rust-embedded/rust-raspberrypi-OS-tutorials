# Tutorial 03 - UART1, Auxilary mini UART

It is time for the famous Hello World example. We're going to write on the UART1
first, as it's easier to program as it has a fixed clocked frequency.

NOTE: qemu does not redirect UART1 to terminal by default, only UART0!

## gpio.rs

We have a new header file. This defines the base MMIO address, and the GPIO
controller's addresses. This file going to be very popular, as many device needs
it.

We are using the [volatile_register] crate to modify MMIO addresses, because it
allows easy wrapping of addresses to volatile types. It will also be used for
UART registers.

[volatile_register]: https://docs.rs/volatile-register/0.2.0/volatile_register/

## uart.rs

A very minimal implementation.

`MiniUart::init(&self)` initializes the device and maps it to the GPIO ports.

`MiniUart::send(&self, c: char)` sends a character over the serial line.

`MiniUart::getc(&self)` receives a character. The carrige return character (13)
will be converted into a newline character (10).

`MiniUart::puts(&self, string: &str)` prints out a string. On newline, a carrige
return character will also be sent (13 + 10).

## main.rs

First we have to call the uart initialization code. Then we wait for the first
keypress from the user before we say "Hello Rustacean!". If you've purchased an
USB serial cable, you should see it on `screen`'s screen. After that, every
character typed in `screen` will be echoed back. If you haven't turned off local
echo, that means you'll see every pressed key twice.
