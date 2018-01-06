Tutorial 10 - Virtual Memory
============================

We came to the simplest and most difficult tutorial at the same time. It's simple as all we are going to do
is fill up an array, and set some registers. The difficulty lies in knowing what values should we put in that array.

I assume you have a fair knowledge about page translation mechanism on AMD64. If not, I strongly suggest to
do some tutorial on it before you continue. ARMv8's MMU is much much more complex and featureful than it's AMD64
counterpart. It is definitely not good to start with.

As AMD64's address translation is very simple, it has one paging table register, it splits memory into 4k
pages only with 4 levels, and it has one well defined hole in the address space. ARMv8 is much more powerful. You
can set the size of the pageframe, the number of translation levels, you can concatenate translation tables for a
given level, and you can even configure the hole's size. It is impossible to cover all of these in one tutorial.
Therefore what I'm going to do is configuring ARMv8 MMU to be similar to AMD64's as much as possible. That is:
we're going to use 4k pageframes with 2M blocks and 512G address space (3 levels) with the 4th level in two registers.
Think of it this way: on AMD64 you would have a 4th level table, pointed by CR3. On ARMv8, we have TTBR0 register which
holds the first entry of that 4th level table, and TTBR1 which holds the last, 512th entry of the table, therefore we
don't need the 4th level table at all. Everything between (memory mapped by the 2nd-511th entries) is in the hole, with
other words they are non-canonical addresses.

The page translation table looks the same: we have 64 bit entries with a physical address and attribute bits in it
at each level. But in ARMv8 you have far more options. You can set cacheability, shareability and accessibility as
well. You also have a special register holding a page attribute array, and you index that with bits in the page
translation attributes.

We are going to translate virtual address space as follows: lower half will be identity mapped in 2M blocks, except
the first block which will be mapped by 4k frames. In the higher half, at -2M we will map the MMIO of UART0.

Mmu.h, mmu.c
------------

`mmu_init()` function to initialize Memory Management Unit.

Start
-----

This time we also have to grant access to the system control register.

Link.ld
-------

This time we need page alignment for the data and the end label.

Main
----

We set up page translations, and then we print to the console with both identity mapped and higher half mapped MMIO.
