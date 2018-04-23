# Tutorial 0A - Power management

For embedded systems, power consumption is critical. The Raspberry Pi 3 has a
very sophisticated PM interface. You can turn each device on and off
independently. There's a catch though. The GPIO VCC pins are hardwired, there's
no way to turn them off programmatically. This means if you connect some devices
to them, you'll have to implement a way to turn those devices off (with a
transistor connected to a data GPIO pin for example).

## power.rs

The power management controller is one of the peripherals that are not emulated
properly by QEMU. Our implementation works on real hardware though.

`Power::off(&self, mbox: &mut mbox::Mbox, gpio: &gpio::GPIO)` shuts down the
board to an almost zero power consumption state.

`Power::reset(&self)` reboots the machine. Also handled by the PMC, and since
the Raspberry Pi does not have a hardware reset button, it's very useful.

When using `make raspboot` and choosing `reset()`, you can see your code in
action nicely as you generate a boot-loop.


## gpio.rs

We introduce a lot of new GPIO pins. It's a good time to refactor the GPIO MMIO
interface into its own type with the common `RegisterBlock` implementation that
you already know from the other components.

## main.rs

We display a simple menu, and wait for user input. Depending on the input, we
reboot the system or power it off.
