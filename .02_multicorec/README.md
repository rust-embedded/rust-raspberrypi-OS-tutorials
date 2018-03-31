Tutorial 02 - Multicore C
=========================

Now let's try something more complex, shall we? By complex I mean stop the CPU cores just like in the first tutorial,
but this time stop one of them from C!

Start
-----

Now we have to distinguish the cores. For that, we read the *mpidr_el1* system register. If it's not zero, we'll
do the former infinite loop. If it's zero, then we'll call a C function. But for that, we need a proper stack, and a
zerod out bss segment in memory before the call instruction. I've added some more code to the Assembly to do all of
that. In case the C code returns (shouldn't), we also jump to the same infinite loop the other CPU cores running.

Makefile
--------

It became a bit trickier. I've added commands for compiling C sources, but in a comform way. From now on, we
can use the same Makefile for every tutorial, regardless of the number of C sources, and I won't discuss it any
further.

Linker script
-------------

Likewise, the linker script became more complex too, as C needs data and bss sections. I've also added a calculation
for the bss size, so that we can refer to it from the Assembly without any further hassle.

It is important to start the text segment with the Assembly code, because we set the stack right before it, hence
the KEEP(). This way our load address is 0x80000, the same as `_start` label and stack top.

Main
----

Finaly, our first C code. Just an empty loop, but still! :-)
