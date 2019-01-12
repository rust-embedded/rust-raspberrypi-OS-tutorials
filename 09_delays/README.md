# Tutorial 09 - Delays

It is very important to wait precise amounts of time while you are interfacing
with low level hardware. In this tutorial, we'll cover thee ways. One is CPU
frequency dependent (and useful if wait time is given in CPU clock cycles), the
other two are µs based.

## delays.rs

`delays::wait_cycles(cyc: u32)` this is very straightforward, we execute the
`nop` instruction n times.

`delays::wait_usec(n: u32)` this implementation uses ARM system registers
(available on all AArch64 CPUs).

`delays::SysTmr::wait_usec_st(&self, n: u64)` is a BCM specific implementation,
which uses the System Timer peripheral (not available on qemu).

## uart.rs

We can now conveniently use `delays::wait_cycles()` in `Uart::init()`.

## main.rs

We test our different wait implementations.
