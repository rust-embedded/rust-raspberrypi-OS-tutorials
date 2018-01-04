Tutorial 03 - UART1, Auxilary mini UART
=======================================

It is time for the famous Hello World example. We're going to write on the UART1 first, as it's easier to program.
NOTE: qemu does not redirect UART1 to terminal by default, only UART0!

Gpio.h
------

We have a new header file. This defines the base MMIO address, and the GPIO controller's addresses. This file
going to be very popular, as many device needs it.

Uart.h, uart.c
--------------

A very minimal implementation.

`uart_init()` initializes the device and maps it to the GPIO ports.

`uart_send(c)` sends a character over the serial line.

`uart_getc()` receives a character. The carrige return character (13) will be converted into a newline character (10).

`uart_puts(s)` prints out a string. On newline, a carrige return character will also be sent (13 + 10).

Main
----

First we have to call the uart initialization code. Then we say "Hello World!". If you've purchased an USB
serial cable, you should see it on minicom's screen. After that every character typed in minicom will be
echoed back. If you haven't turned off local echo, that means you'll see every pressed key twice.

