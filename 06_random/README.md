Tutorial 06 - Hardware Random Number Generator
==============================================

This going to be an easy tutorial. We query a number from the (undocumented) hardware random
number generator. You can use this to implement a simple, but accurate dice throw in any game.
It is important as without hardware support you can only generate pseudo-random numbers.

Rand.h, rand.c
--------------

`rand_init()` initializes the hardware.

`rand(min,max)` returns a random number between min and max.

Main
----

We query a random value and then we display it on the serial console.
