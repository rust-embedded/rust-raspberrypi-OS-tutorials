Tutorial 05 - UART0, PL011
==========================

This tutorial does the same as tutorial 04, but it prints the serial number on UART0. As such, it can be used
easily with qemu, like

```sh
$ qemu-system-aarch64 -M raspi3 -kernel kernel8.img -serial stdio
My serial number is: 0000000000000000
```

Uart.h, uart.c
--------------

Before we could use a rate divisor value, we must establish a valid clock rate for the PL011. It's done
via mailboxes, with the same property channel we used earlier. Otherwise this interface is identical to the
UART1 one.

Main
----

We query the board's serial number and then we display it on the serial console.
