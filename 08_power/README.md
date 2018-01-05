Tutorial 08 - Power management
==============================

For embedded systems, power consumption is critical. The Raspberry Pi 3 has a very sophisticated
PM interface. You can turn each device on and off idependently. There's a catch though, the GPIO
VCC pins are hardwired, there's no way to turn them off programatically. This means if you connect
some devices to them, you'll have to implement a way to turn those devices off (with a transistor
connected to a data GPIO pin for example).

Power.h, power.c
----------------

The power management controller is one of the peripherals that are not emulated properly by qemu.
Works on real hardware though.

`power_off()` shutdowns the board to a almost zero power consumption state.

`reset()` reboots the machine. Also handled by the PMC, and since the Raspberry Pi does not have
a hardware reset button, it's very useful.

Main
----

We display a simple menu, and wait for user input. Depending on the input, we reboot the system or
power it off.
