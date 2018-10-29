# Tutorial 03 - UART1, Auxilary mini UART

It is time for the famous Hello World example. We're going to write on the UART1
first, as it is easier to program, since it has a fixed clocked frequency.

## gpio.rs

We have a new file that defines the GPIO controller addresses. It is going to be
very popular, as many device will need it in the future.

We are using the [register][register] crate to modify MMIO addresses, because it
allows easy wrapping of addresses to volatile types. It will also be used for
UART registers.

[register]: https://crates.io/crates/register

## uart.rs

A very minimal implementation.

`MiniUart::init(&self)` initializes the device and maps it to the GPIO ports.

`MiniUart::send(&self, c: char)` sends a character over the serial line.

`MiniUart::getc(&self)` receives a character. The carrige return character (13)
will be converted into a newline character (10).

`MiniUart::puts(&self, string: &str)` prints out a string. On newline, a carrige
return character will also be sent (13 + 10).

## main.rs

First, we have to call the uart initialization code. Then we wait for the first
keypress from the user before we say "Hello Rustacean!". If you've purchased an
USB serial cable, you should see it on `screen`'s screen. After that, every
character typed in `screen` will be echoed back. If you haven't turned off local
echo, that means you'll see every pressed key twice.

## Simulation

We can also use `QEMU` to simulate the UART output of our bare-metal binary on
the host PC.

```console
ferris@box:~$ make qemu
<Press any key>
Hello Rustacean!
```

However, let it be said that it is more thrilling to see your first output from
the real hardware target, so don't be shy and grab a USB-serial.
