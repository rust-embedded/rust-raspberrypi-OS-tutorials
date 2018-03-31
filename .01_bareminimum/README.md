Tutorial 01 - Bare Minimum
==========================

Okay, we're not going to do anything here, just test our toolchain. The resulting kernel8.img should
boot on Raspberry Pi, and stop the CPU cores in an infinite loop. You can check that by running

```sh
$ qemu-system-aarch64 -M raspi3 -kernel kernel8.img -d in_asm
        ... output removed for clearity, last line: ...
0x0000000000080004:  17ffffff      b #-0x4 (addr 0x80000)
```

Start
-----

When the control is passed to kernel8.img, the environment is not ready for C. Therefore we must
implement a small preambule in Assembly. As this first tutorial is very simple, that's all we have, no C
for now.

Note that CPU has 4 cores. All of them are running the same infinite loop for now.

Makefile
--------

Our Makefile is very simple. We compile start.S, as this is our only source. Then in linker phase we
link it using the linker.ld script. Finaly we convert the resulting elf executable into a raw image.

Linker script
-------------

Not surpisingly simple too. We just set the base address where our kernel8.img will be loaded, and we
put the only section we have there. Important note, for AArch64 the load address is **0x80000**, and
not **0x8000** as with AArch32.

