# Operating System development tutorials in Rust on the Raspberry Pi

![](https://github.com/rust-embedded/rust-raspi3-OS-tutorials/workflows/BSP-RPi3/badge.svg) ![](https://github.com/rust-embedded/rust-raspi3-OS-tutorials/workflows/BSP-RPi4/badge.svg)

## Notice

**This is a work-in-progress rewrite, started on September 2019.** You can find the original version of the tutorials
[here](https://github.com/rust-embedded/rust-raspi3-OS-tutorials/tree/original_version).

Some general info:
- Tutorials that existed before the rewrite have a full-fledged tutorial
  text, while most new tutorials will only contain a  short`tl;dr` section for now.
  I plan to provide full-fledged text for all tutorials once the kernel has reached
  a certain milestone.
- The code written in these tutorials supports and runs on the **Raspberry Pi
  3** and the **Raspberry Pi 4**.
  - Tutorials 1 till 5 are groundwork code which only makes sense to run on
    QEMU.
  - Starting with [tutorial 6](06_drivers_gpio_uart), you can load and run the
    kernel on Raspberrys and observe output over UART.
- For editing, I recommend [Visual Studio Code] with the [Rust Language Server]
  extension.
- Check out the `make doc` command to browse the code with HTML goodness.

_Cheers,
[Andre](https://github.com/andre-richter)_


[Visual Studio Code]: https://code.visualstudio.com
[Rust Language Server]: https://github.com/rust-lang/rls

## Introduction

The target audience is hobby OS developers who are new to ARM's 64 bit [ARMv8-A
architecture](https://developer.arm.com/products/architecture/cpu-architecture/a-profile/docs).
The tutorials will give a guided, step-by-step tour of how to write a
[monolithic] Operating System `kernel` for an `embedded system` from scratch.
They cover implementation of common Operating Systems tasks, like writing to
the serial console, setting up virtual memory and exception handling. All while
leveraging Rust's unique features to provide for safety and speed.

[monolithic]: https://en.wikipedia.org/wiki/Monolithic_kernel

While the Raspberry Pi 3 and 4 are the main target boards, the code is written
in a modular fashion which allows for easy porting to other CPU architectures
and/or boards.

I would really love if someone takes a shot at a **RISC-V** implementation.

## Ease of use

This repo tries to put a focus on user friendliness. Therefore, I made some
efforts to eliminate the biggest painpoint in embedded development: Toolchain
hassles.

Users eager to try the code should not be bothered with complicated toolchain
installation/compilation steps. This is achieved by trying to use the standard
Rust toolchain as much as possible, and bridge existing gaps with Docker
containers. [Please install Docker for your
distro](https://docs.docker.com/install/).

The setup consists of the following components:

- Compiler, linker and binutils are used from Rust nightly.
- Additional OS Dev tools, like QEMU, are pre-packaged into [this
  container](docker/rustembedded-osdev-utils).

If you want to know more about docker and peek at the the containers used in
these tutorials, please refer to the repository's docker folder.

## Prerequisites

Before you can start, you'll need a suitable Rust toolchain.

```bash
curl https://sh.rustup.rs -sSf  \
    |                           \
    sh -s --                    \
    --default-toolchain nightly \
    --component rust-src llvm-tools-preview clippy rustfmt rls rust-analysis

cargo install cargo-xbuild cargo-binutils
```

## USB Serial

It is highly recommended to get a USB serial debug cable. It also powers the
Raspberry once you connect it, so you don't need extra power over the dedicated
power-USB. I use a bunch of
[these](https://www.amazon.de/dp/B0757FQ5CX/ref=cm_sw_r_tw_dp_U_x_ozGRDbVTJAG4Q).

You connect it to the GPIO pins 14/15 as shown beyond.

[Tutorial 6](06_drivers_gpio_uart) is the first where you can use it. Go to the
README there for instructions on how to prepare the SD card to run your
self-made kernels from it.

![UART wiring diagram](doc/wiring.png)

## Acknowledgements

The original version of the tutorials started out as a fork of [Zoltan
Baldaszti](https://github.com/bztsrc)'s awesome [tutorials on bare metal
programming on RPi3](https://github.com/bztsrc/raspi3-tutorial) in `C`. Thanks
for giving me a head start!

## License

Licensed under the MIT license ([LICENSE-MIT](LICENSE) or http://opensource.org/licenses/MIT).
