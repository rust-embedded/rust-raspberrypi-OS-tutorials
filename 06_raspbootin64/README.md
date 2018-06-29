# Tutorial 06 - Raspbootin64

We are now at a point where we have a running serial connection, but for each
new feature we want to try, we still have to write and exchange the SD card
every time.

As this tends to get very annoying and also to avoid potential SD card damage,
we create a kernel8.img that will load the real kernel8.img over serial.

This tutorial is a rewrite of the well known serial boot loader
[raspbootin][bootin] in 64-bit. We only provide one part of the loader, the
kernel receiver, which runs on the RPi. For the other part, the sender, which
runs on your PC, we will rely on the original [raspbootcom][bootcom] utility.

[bootin]:(https://github.com/mrvn/raspbootin)
[bootcom]:(https://github.com/mrvn/raspbootin/blob/master/raspbootcom/raspbootcom.cc)

For convenience, it is already packaged in our `raspi3-utils` docker
container. So if you are running a Linux host, it will be as easy as calling
another Makefile target. It will be included starting with the next tutorial,
`07_abstraction`. You can invoke it with

```bash
make raspboot
```

If you want to use it with earlier versions of this tutorial, here is a bash
command to invoke it:

```bash
docker run -it --rm \
           --privileged -v /dev/:/dev/ \
           -v $PWD:/work -w /work \
           raspi3-utils \
           raspbootcom /dev/ttyUSB0 kernel8.img
```

In any case, if your USB device is enumerated differently, adapt accordingly.

If you want to send kernels from a Windows machine, I suggest to take a look at
John Cronin's rewrite, [raspbootin-server][w32] which can be compiled for the
Win32 API. Even more, [@milanvidakovic](https://github.com/milanvidakovic) was
so kind to share a [Java version][java] of the kernel sender with you.

[w32]:(https://github.com/jncronin/rpi-boot/blob/master/raspbootin-server.c)
[java]:(https://github.com/milanvidakovic/Raspbootin64Client)

## Chain Loading

In order to load the new kernel to the same address, we have to move ourself out
of the way. It's called `chain loading`: One code loads the next code to the
same position in memory, therefore the latter thinks it was loaded by the
firmware. To implement that, we use a different linking address this time, and
since the GPU loads us to `0x80_000` regardless, we have to copy our code to
that link address. When we're done, the memory at `0x80_000` is free to use. You
can check that with:

```sh
$ cargo nm -- kernel8 | grep reset
000000000007ffc0 T reset
```

We also should minimize the size of the loader, since it will be overwritten by
the newly loaded code anyway. By removing `Uart::puts()` and other functions,
we've managed to shrink the loader's size to 1024 bytes.

## boot_cores.S

First, we have to save the arguments in registers passed by the
firmware. Second, we added a loop to relocate our code to the address it should
have been loaded to. And last, since rustc generates RIP-relative jumps, we must
adjust the branch instruction to jump to the relocated Rust code.

## Linker and Glue Code

We use a different linking address this time. We calculate our code's size to
know how many bytes we have to copy.

Additionally, we can remove the `bss section` entirely, since our loader does
not use any static variables.

## main.rs

We print 'RBIN64', receive the new kernel over serial and save it at the memory
address where the start.elf would have been loaded it. When finished, we restore
the arguments and jump to the new kernel using an absolute address.
