# Bare Metal Rust Programming on Raspberry Pi 3

[![Build Status](https://travis-ci.org/rust-embedded/rust-raspi3-tutorial.svg?branch=master)](https://travis-ci.org/rust-embedded/rust-raspi3-tutorial)

## Introduction

This repository aims to provide easy reference code for programming bare metal
on the Raspberry Pi 3 in the [Rust systems programming language]. Emphasis is on
leveraging Rust's zero-overhead abstractions to compile lean code that is
readable, concise and safe (at least as safe as it gets on bare-metal hardware).

[Rust systems programming language]: https://www.rust-lang.org

The target audience is **hobby OS developers** who are new to this hardware. It
will give you examples on how to do common Operating Systems tasks, like writing
to the serial console, reading keystrokes from it or use various peripherals
like a hardware-backed random number generator.

However, it is *not* a tutorial on how to write a _complete_ OS. I won't cover
topics like advanced memory management and virtual file systems, or how to
implement multi-tasking. Rather, it comprises a set of micro-tutorials that
introduce different topics one after the other. Maybe in the distant future, we
might introduce a meta tutorial that combines all the resources to a full-fleged
kernel that can multi-task some simple userspace processes, but don't take my
word for it.

## Environment

This repo tries to put a focus on user friendliness. Therefore, I made some
efforts to eliminate the biggest painpoint in embedded development: _Toolchain
hassles_.

Users eager to try the code should not be bothered with complicated toolchain
installation/compilation steps. This is achieved by trying to use the standard
Rust toolchain as much as possible, and bridge existing gaps with Docker
containers. Please [install Docker for your distro].

The setup consists of the following components:
1. Compiler, linker and binutils are used from Rust nightly.
2. QEMU will be used for emulation, but RPi3 support in QEMU is very fresh and has not landed in most of the pre-packaged versions of popular distributions. [This] container will provide it ready to go.

Please notice that you won't need to download or prepare the containers
upfront. As long as you have docker installed, they will be pulled automatically
the first time the Makefile needs them.

[install Docker for your distro]: https://www.docker.com/community-edition#/download
[This]: https://github.com/andre-richter/docker-raspi3-qemu

For now, only a few basic tutorials are ready, but more will be ported over
time.

## Prerequisites

Before you can start, you'll need a suitable Rust toolchain.
```bash
curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly
rustup component add rust-src llvm-tools-preview
cargo install cargo-xbuild cargo-binutils
rustup component add clippy-preview --toolchain=nightly
```

Additionally, a Micro SD card with [firmware
files](https://github.com/raspberrypi/firmware/tree/master/boot) on a FAT
filesystem.

I recommend to get a [Micro SD card USB
adapter](http://media.kingston.com/images/products/prodReader-FCR-MRG2-img.jpg)
(many manufacturers ship SD cards with such an adapter), so that you can connect
the card to any desktop computer just like an USB stick, no special card reader
interface required (although many laptops have those these days).

You can create an MBR partitioning scheme on the SD card with an LBA FAT32 (type
0x0C) partition, format it and copy `bootcode.bin`, `start.elf` and `fixup.dat`
onto it. **Delete all other files or booting might not work**. Or alternatively
you can download a raspbian image, `dd` it to the SD card, mount it and delete
the unnecessary .img files. Whichever you prefer. What's important, you'll
create `kernel8.img` with these tutorials which must be copied to the root
directory on the SD card, and no other `.img` files should exists there.

I'd also recommend to get an [USB serial debug
cable](https://www.adafruit.com/product/954). You connect it to the GPIO pins
14/15.

![UART wiring diagram](doc/wiring.png)

Then, run `screen` on your desktop computer like

```bash
sudo screen /dev/ttyUSB0 115200
```

Exit screen again by pressing <kbd>ctrl-a</kbd> <kbd>ctrl-d</kbd>

## About this repository

The tutorial is basically a combination of two awesome resources.
  1. First of all, it is a fork of [Zoltan Baldaszti]'s awesome [tutorial] on bare metal programming on RPi3 in `C`.
     1. Rust code will be based on his files, READMEs will be adapted, and I might change things here and there if I think it is beneficial. However, credits to this guy plz!
  2. The second props go to [Jorge Aparicio] for ["The Embedonomicon"], from which the boot code is taken.

[Zoltan Baldaszti]: https://github.com/bztsrc
[tutorial]: https://github.com/bztsrc/raspi3-tutorial
[Jorge Aparicio]: https://github.com/japaric
["The Embedonomicon"]: https://rust-embedded.github.io/embedonomicon/

## About the hardware

There are lots of pages on the internet describing the Raspberry Pi 3 hardware
in detail, so I'll be brief and cover only the basics.

The board is shipped with a [BCM2837
SoC](https://github.com/raspberrypi/documentation/tree/master/hardware/raspberrypi/bcm2837)
chip. That includes a

 - VideoCore GPU
 - ARM-Cortex-A53 CPU (ARMv8)
 - Some MMIO mapped pheripherals.

Interestingly the CPU is not the main processor on the board. When it's powered
up, first GPU runs. When it's finished with the initialization by executing the
code in bootcode.bin, it will load and execute the start.elf executable.  That's
not an ARM executable, but compiled for the GPU. What interests us is that
start.elf looks for different ARM executables, all starting with `kernel` and
ending in `.img`. As we're going to program the CPU in AArch64 mode, we'll need
`kernel8.img` only, which is the last to look for. Once it's loaded, the GPU
triggers the reset line on the ARM processor, which starts executing code at
address 0x80000 (or more precisely at 0, but the GPU puts an ARM jump code there
first).

The RAM (1G for the Raspberry Pi 3) is shared among the CPU and the GPU, meaning
one can read what the other has written into memory. To avoid confusion, a well
defined, so called [mailbox
interface](https://github.com/raspberrypi/firmware/wiki/Mailboxes) is
established. The CPU writes a message into the mailbox, and tells the GPU to
read it. The GPU (knowing that the message is entirely in memory) interprets it,
and places a response message at the same address. The CPU has to poll the
memory to know when the GPU is finished, and then it can read the response.

Similarily, all peripherals communicates in memory with the CPU. Each has it's
dedicated memory address starting from 0x3F000000, but it's not in real RAM
(called Memory Mapped IO). Now there's no mailbox for peripherals, instead each
device has it's own protocol. What's common for these devices that their memory
must be read and written in 32 bit units at 4 bytes aligned addresses (so called
words), and each has control/status and data words. Unfortunately Broadcom (the
manufacturer of the SoC chip) is legendary bad at documenting their
products. The best we've got is the BCM2835 documentation, which is close
enough.

There's also a Memory Management Unit in the CPU which allows creating virtual
address spaces. This can be programmed by specific CPU registers, and care must
be taken when you map these MMIO addresses into a virtual address space.

Some of the more interesting MMIO addresses are:
```
0x3F003000 - System Timer
0x3F00B000 - Interrupt controller
0x3F00B880 - VideoCore mailbox
0x3F100000 - Power management
0x3F104000 - Random Number Generator
0x3F200000 - General Purpose IO controller
0x3F201000 - UART0 (serial port, PL011)
0x3F215000 - UART1 (serial port, AUX mini UART)
0x3F300000 - External Mass Media Controller (SD card reader)
0x3F980000 - Universal Serial Bus controller
```
For more information, see Raspberry Pi firmware wiki and documentation on github.

https://github.com/raspberrypi

Good luck and enjoy hacking with your Raspberry! :-)

Andre

## License

Licensed under the MIT license ([LICENSE-MIT](LICENSE) or http://opensource.org/licenses/MIT).
