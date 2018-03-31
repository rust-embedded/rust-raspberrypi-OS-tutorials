Tutorial 11 - Exceptions
========================

Last time we have used a very simple translation scheme, but in real life you often need more. And it is not
easy to write the table blindly, so we're going to add exception handlers this time. This will print out some
system registers to identify the problem with our translation tables.

```sh
$ qemu-system-aarch64 -M raspi3 -kernel kernel8.img -serial stdio
Synchronous: Data abort, same EL, Translation fault at level 2:
  ESR_EL1 0000000096000006 ELR_EL1 0000000000080D7C
 SPSR_EL1 00000000200003C4 FAR_EL1 FFFFFFFFFF000000
```

Here ESR_EL1 tells us that it was a Data Abort, caused by a translation table error at level 2. The instruction
triggered it is at address 0x80D7C which tried to access memory at 0xFFFFFFFFFF000000.

The vector is very similar to AMD64's IDT, with two exceptions: first, there's no special instruction (like sidt),
but a system register stores the address. Second, we don't pass a table with addresses rather the address of the
actual code. Therefore each "entry" of the vector table is bigger, small stubs so that you can set up arguments
and jump to a common handler.

On AMD64 you have 32 entry points for each exception. On AArch, you only have one, and you can read the excpetion
code from a system register. Considering that all OS sets a code for the exception and jumps to a common handler,
this makes life easier.

Exc.c
-----

`exc_handler()` a simple exception handler that dumps registers and decodes ESR_EL1 (partially). We simply stop
the CPU for now, as we have no means to recover from an exception yet. The full comprehensive description of
Exception Syndrome Register can be found in ARM DDI0487B_b chapter D10.2.28.

Start
-----

Before we switch to supervisor mode, we set up *vbar_el1*. All handlers must be properly aligned.
Qemu is not so picky, but real hardware is.

`_vectors` the exception handler's vector table with small assembly stubs, each calling `exc_handler()` in C.

Main
----

We set up page translations, and then we deliberatly reference an unmapped address to trigger an exception.
