Tutorial 14 - Raspbootin64
==========================

Because changing SD card is boring and also to avoid potential SD card damage, we create a kernel8.img that will
load the real kernel8.img over serial.

This tutorial is a rewrite of the well known serial boot loader, [raspbootin](https://github.com/mrvn/raspbootin) in 64 bit.
I only provide one part of the loader, the kernel receiver, which runs on the RPi. For the other
part, the sender, which runs on your PC see the original [raspbootcom](https://github.com/mrvn/raspbootin/blob/master/raspbootcom/raspbootcom.cc) utility.
If you want to send kernels from a Windows machine, I suggest to take a look at John Cronin's rewrite,
[raspbootin-server](https://github.com/jncronin/rpi-boot/blob/master/raspbootin-server.c) which can be compiled for the Win32 API.

In order to load the new kernel to the same address, we have to move ourself out of the way. It's called chain
loading: one code loads the next code to the same position in memory, therefore the latter thinks it was loaded
by the firmware. To implement that we use a different linking address this time, and since GPU loads us to 0x80000
regardless, we have to copy our code to that link address. What's important, that we can only use relative addresses
while doing so. When we're done, the memory at 0x80000 must be free to use. You can checked that with:

```sh
$ aarch64-elf-readelf -s kernel8.elf | grep __bss_end
    21: 000000000007ffc0     0 NOTYPE  GLOBAL DEFAULT    4 __bss_end
```

We also should minimize the size of the
loader, since it will be regarded by the newly loaded code anyway. By removing `uart_puts()` I've managed to shrink the
size below 1024 bytes.

Start
-----

Added a loop to relocate our code to the address it should have been loaded to.

Linker
------

We use a different linking address this time. Similarly to bss size calculation, we calculate our code's size to
know how many bytes we have to copy.

Main
----

We print 'RBIN64', receive the new kernel over serial, and jump to it.
