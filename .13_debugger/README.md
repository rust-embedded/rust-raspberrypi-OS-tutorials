Tutorial 13 - Debugger
======================

Let's rock by implementing an interactive debugger in our exception handler! :-) Now that we have printf(), it
shouldn't be hard.

```sh
$ qemu-system-aarch64 -M raspi3 -kernel kernel8.img -serial stdio
Synchronous: Breakpoint instruction
> x 
0007FFF0: 13 60 09 00  00 00 00 00  24 10 20 3F  00 00 00 00  .`......$. ?....
> i x30 x30+64 
00080804: D2800000      movz      x0, #0x0
00080808: 94003D1C      bl        0x8FC78
0008080C: 94003ECF      bl        0x90348
00080810: D69F03E0      eret      
00080814: D503201F        27 x nop
>
```

Dbg.h, dbg.c
------------

A very minimal and simple debugger (~300 lines in C).

`breakpoint` a newly defined keyword. We can use this anywhere in our code where we want to invoke the debugger

`dbg_decodeexc()` similar to exc_handler in tutorial 11, decodes the cause of the exception and prints it

`dbg_getline()` yep, another low level library we're missing. We need a way to allow the user to edit command line
and return it as a string when he/she presses <kbd>Enter</kbd>. A minimal implementation, as usual

`dbg_getoffs()` this function parses the command line for arguments. Accepts hex, decimal number in
"register+/-offset" format

`dbg_main()` the main loop of the debugger.

Disasm.h
--------

Because it's small (~72k), extremely easy to integrate, yet it supports all ARMv8.2 instructions, I decided to
use the [Universal Disassembler](https://github.com/bztsrc/udisasm) for this tutorial. If you don't want to
compile a disassembler into your debugger, simply set the DISASSEMBLER define 0 in top of dbg.c.

Start
-----

Our `_vector` table looks different. We have to save registers in memory with `dbg_saveregs`, print out
the cause of the exception, and call our mini-debugger's main loop.

Main
----

We'll test our shiny new `breakpoint` keyword in C.
