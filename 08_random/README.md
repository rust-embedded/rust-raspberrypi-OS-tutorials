# Tutorial 08 - Hardware Random Number Generator

This going to be an easy tutorial. We query a number from the (undocumented)
hardware random number generator. You can use this to implement a simple, but
accurate dice throw in any game. It is important as without hardware support you
can only generate pseudo-random numbers.

## rand.s

Due to lack of documentation, we [mimic the respective Linux driver]
(https://github.com/torvalds/linux/blob/master/drivers/char/hw_random/bcm2835-rng.c).

`Rng::init(&self)` initializes the hardware.

`Rng::rand(&self, min: u32, max: u32)` returns a random number between min and
max.

## main.rs

Press a key to query a random value and then display it on the serial console.
