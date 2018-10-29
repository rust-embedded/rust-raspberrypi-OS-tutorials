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
`07_abstraction`. You can invoke it with:

```sh
make raspboot
```

If you want to use it with earlier versions of this tutorial, here is a bash
command to invoke it:

```sh
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
firmware. To implement that, we use a different linking address this time (we
subtract `2048` from the original address). You can check that with:

```console
ferris@box:~$ cargo nm -- kernel8 | grep _boot_cores
000000000007f800 T _boot_cores
```

However, since the GPU loads us to `0x80_000` regardless, as a first action in
our binary, we have to copy our code to that link address. This is added to
`boot_cores.S`:

```asm
    // relocate our code from load address to link address
    ldr     x1, =0x80000
    ldr     x2, =_boot_cores //<- actual link addr (0x80000 - 2048) from link.ld
    ldr     w3, =__loader_size
3:  ldr     x4, [x1], #8
    str     x4, [x2], #8
    sub     w3, w3, #1
    cbnz    w3, 3b
```

When we're done, the memory at `0x80_000` is free to use.

We also should minimize the size of the loader, since it will be overwritten by
the newly loaded code anyway. By removing `Uart::puts()` and other functions,
we've managed to shrink the loader's size by some bytes.

## Position Independent Code (PIC)

For reasons stated above, our code will initially execute from address
`0x80_000` despite the binary being actually linked to `0x7f_800`. In order to
ensure that our binary will not reference hardcoded addresses that actually
contain no or wrong data, we need to make this binary `position
independent`. This means that all addresses will always be runtime-computable as
an offset to the current `Program Counter`, and not hardcoded.

To enable PIC for our loader, we add the following line to the compiler flags in
the`.cargo/config`:

```toml
[target.aarch64-unknown-none]
rustflags = [
  "-C", "link-arg=-Tlink.ld",
  "-C", "target-feature=-fp-armv8",
  "-C", "target-cpu=cortex-a53",
  "-C", "relocation-model=pic", # <-- New
]
```

## boot_cores.S

In addition to the relocation copying, we also need to adjust the branch
instruction that jumps to the reset handler, because we want to jump to _the
relocated reset handler_, not the original one.

Since rustc now generates jumps relative to the current instruction due to the
`position independence`, we can leverage this feature and add the same offset
to the reset address that we implicitly used for the relocation copying (`2048`).
This ensures that we jump to the reset handler _in the relocated loader code_.

## Linker and Boot Code

We use a different linking address this time. We calculate our code's size to
know how many bytes we have to copy.

Additionally, we can remove the `bss section` entirely, since our loader does
not use any static variables.

## main.rs

We print 'RBIN64', receive the new kernel over serial and save it at the memory
address where the start.elf would have been loaded it. When finished, we restore
the arguments and jump to the new kernel using an absolute address.
