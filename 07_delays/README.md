Tutorial 07 - Delays
====================

It is very important to wait precise amounts of time while you're interfacing with low level hardware.
In this tutorial we'll cover there ways. One is CPU frequency dependent (and useful if wait time is given
in CPU clock cycles), the other two are microsec (millionth of a second) based.

Delays.h, delays.c
------------------

`wait_cycles(n)` this is a very straightforward thing, we execute the 'nop' instruction n times.

`wait_msec(n)` this implementation uses ARM system registers (available on all AArch64 CPUs).

`wait_msec_st(n)` is a BCM specific implementation, which uses the System Timer peripheral (not available on qemu).

Main
----

We use different wait implementations between printing strings on serial console.
