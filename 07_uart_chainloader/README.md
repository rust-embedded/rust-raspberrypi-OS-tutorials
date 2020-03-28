# Tutorial 07 - UART Chainloader

## tl;dr

Running from an SD card was a nice experience, but it would be extremely tedious to do it for every
new binary. Let's write a [chainloader] using [position independent code]. This will be the last
binary you need to put on the SD card. Each following tutorial will provide a `chainboot` target in
the `Makefile` that lets you conveniently load the kernel over `UART`.

[chainloader]: https://en.wikipedia.org/wiki/Chain_loading
[position independent code]: https://en.wikipedia.org/wiki/Position-independent_code

## Install and test it

Our chainloader is called `MiniLoad` and is inspired by [raspbootin].

You can try it with this tutorial already:
1. Depending on your target hardware:`make` or `BSP=rpi4 make`.
2. Copy `kernel8.img` to the SD card.
3. Execute `make chainboot` or `BSP=rpi4 make chainboot`.
4. Now plug in the USB Serial.
5. Observe the loader fetching a kernel over `UART`:

[raspbootin]: https://github.com/mrvn/raspbootin

```console
$ make chainboot
[...]
Minipush 1.0

[MP] ⏳ Waiting for /dev/ttyUSB0
[MP] ✅ Connected
 __  __ _      _ _                 _
|  \/  (_)_ _ (_) |   ___  __ _ __| |
| |\/| | | ' \| | |__/ _ \/ _` / _` |
|_|  |_|_|_||_|_|____\___/\__,_\__,_|

           Raspberry Pi 3

[ML] Requesting binary
[MP] ⏩ Pushing 7 KiB ==========================================🦀 100% 0 KiB/s Time: 00:00:00
[ML] Loaded! Executing the payload now

[0] Booting on: Raspberry Pi 3
[1] Drivers loaded:
      1. BCM GPIO
      2. BCM PL011 UART
[2] Chars written: 93
[3] Echoing input now
```

In this tutorial, a version of the kernel from the previous tutorial is loaded
for demo purposes. In subsequent tuts, it will be the working directory's
kernel.

## Test it

The `Makefile` in this tutorial has an additional target, `qemuasm`, that lets
you nicely observe the jump from the loaded address (`0x80_XXX`) to the
relocated code at (`0x3EFF_0XXX`):

```console
$ make qemuasm
[...]
IN:
0x00080990:  d0000008  adrp     x8, #0x82000
0x00080994:  52800020  movz     w0, #0x1
0x00080998:  f9416908  ldr      x8, [x8, #0x2d0]
0x0008099c:  d63f0100  blr      x8

----------------
IN:
0x3eff0b10:  d0000008  adrp     x8, #0x3eff2000
0x3eff0b14:  d0000009  adrp     x9, #0x3eff2000
0x3eff0b18:  f941ad08  ldr      x8, [x8, #0x358]
0x3eff0b1c:  f941b129  ldr      x9, [x9, #0x360]
0x3eff0b20:  eb08013f  cmp      x9, x8
0x3eff0b24:  540000c2  b.hs     #0x3eff0b3c
[...]
```

## Diff to previous
