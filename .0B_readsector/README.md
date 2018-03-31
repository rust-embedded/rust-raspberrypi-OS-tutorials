Tutorial 0B - Read Sector
=========================

So far we have linked our data (pixmap, font) to the kernel image. It is time to read data from the
SD card. For this tutorial we're implementing a real driver for read sector function.

Sd.h, sd.c
------------

Well, it would be nice to have a mailbox for reading and writing sectors, but there isn't. So we have to
talk directly to the EMMC, which is tricky and boring. We have to handle all kinds of cards. But finally,
we have two function.

`sd_init()` initialize EMMC for SD card read.

`sd_readblock(lba,buffer,num)` read num blocks (sectors) from the SD card into buffer starting at sector lba.

Main
----

We read a block after the bss segment in memory, and then we dump it to the console. The read function will
display detailed information on the EMMC communication.
