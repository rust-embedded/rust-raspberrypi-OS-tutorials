# Operating System development tutorials in Rust on the Raspberry Pi

![](https://github.com/rust-embedded/rust-raspi3-OS-tutorials/workflows/BSP-RPi3/badge.svg) ![](https://github.com/rust-embedded/rust-raspi3-OS-tutorials/workflows/BSP-RPi4/badge.svg) ![](https://github.com/rust-embedded/rust-raspi3-OS-tutorials/workflows/Unit-Tests/badge.svg) ![](https://github.com/rust-embedded/rust-raspi3-OS-tutorials/workflows/Integration-Tests/badge.svg) ![](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue)

<br/>

<img src="doc/header.jpg" height="390"> <img src="doc/minipush_demo_frontpage.gif" height="390">

## ℹ️ Introduction

This is a tutorial series for hobby OS developers who are new to ARM's 64 bit
[ARMv8-A architecture]. The tutorials will give a guided, step-by-step tour of
how to write a [monolithic] Operating System `kernel` for an `embedded system`
from scratch. They cover implementation of common Operating Systems tasks, like
writing to the serial console, setting up virtual memory and handling HW
exceptions. All while leveraging `Rust`'s unique features to provide for safety
and speed.

_Cheers, Andre ([@andre-richter])_

P.S.: In the future, Chinese :cn: versions of the tutorials will be maintained as [`README.CN.md`](README.CN.md) by [@colachg] and [@readlnh].

[ARMv8-A architecture]: https://developer.arm.com/products/architecture/cpu-architecture/a-profile/docs
[monolithic]: https://en.wikipedia.org/wiki/Monolithic_kernel
[@andre-richter]: https://github.com/andre-richter
[@colachg]: https://github.com/colachg
[@readlnh]: https://github.com/readlnh

## 📑 Organization

- Each tutorial contains a stand-alone, bootable `kernel` binary.
- Each new tutorial extends the previous one.
- Each tutorial `README` will have a short `tl;dr` section giving a brief
  overview of the additions, and show the source code `diff` to the previous
  tutorial, so that you can conveniently inspect the changes/additions.
- Some tutorials have a full-fledged, detailed text in addition to the `tl;dr`
  section. The long-term plan is that all tutorials get a full text, but for now
  this is exclusive to tutorials where I think that `tl;dr` and `diff` are not
  enough to get the idea.
- The code written in these tutorials supports and runs on the **Raspberry Pi
  3** and the **Raspberry Pi 4**.
  - Tutorials 1 till 5 are groundwork code which only makes sense to run in
    `QEMU`.
  - Starting with [tutorial 6](06_drivers_gpio_uart), you can load and run the
    kernel on Raspberrys and observe output over `UART`.
- Although the Raspberry Pi 3 and 4 are the main target boards, the code is
  written in a modular fashion which allows for easy porting to other CPU
  architectures and/or boards.
  - I would really love if someone takes a shot at a **RISC-V** implementation!
- For editing, I recommend [Visual Studio Code] with [Rust Analyzer].
- In addition to the tutorial text, also check out the `make doc` command to
  browse the code with HTML goodness.

[Visual Studio Code]: https://code.visualstudio.com
[Rust Analyzer]: https://rust-analyzer.github.io

## 🚀 Ease of use

This series tries to put a strong focus on user friendliness. Therefore, I made
efforts to eliminate the biggest painpoint in embedded development: Toolchain
hassles.

Users eager to try the code will not be bothered with complicated toolchain
installation/compilation steps. This is achieved by using the standard Rust
toolchain as much as possible, and provide all additional tooling through an
accompanying Docker container. The container will be pulled in automagically
once it is needed. The only requirement is that you have [installed Docker for
your distro](https://docs.docker.com/install/).

The development setup consists of the following components:

- Compiler, linker and binutils are used from Rust nightly.
- Additional OS Dev tools, like `QEMU` or `GDB`, are provided by [this
  container](docker/rustembedded-osdev-utils).

If you want to know more about docker and peek at the the container used for the
tutorials, please refer to the repository's [docker](docker) folder.

## 🛠 Prerequisites

Before you can start, you must install a suitable Rust toolchain:

```bash
curl https://sh.rustup.rs -sSf             \
    |                                      \
    sh -s --                               \
    --default-toolchain nightly-2019-12-20 \
    --component rust-src llvm-tools-preview rustfmt

source $HOME/.cargo/env
cargo install cargo-xbuild cargo-binutils
```

In case you use `Visual Studio Code`, I strongly recommend installing the
[Rust Analyzer extension] as well.

[Rust Analyzer extension]: https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer

## 📟 USB Serial Output

Since the kernel developed in the tutorials runs on the real hardware, it is
highly recommended to get a USB serial debug cable to make the experience.
The cable also powers the Raspberry once you connect it, so you don't need extra
power over the dedicated power-USB.

- I use a bunch of [these serial cables](https://www.amazon.de/dp/B0757FQ5CX/ref=cm_sw_r_tw_dp_U_x_ozGRDbVTJAG4Q).
- You connect it to the GPIO pins `14/15` as shown below.
- [Tutorial 6](06_drivers_gpio_uart) is the first where you can use it.
  Check it out for instructions on how to prepare the SD card to boot your
  self-made kernel from it.
- Starting with [tutorial 7](07_uart_chainloader), booting kernels on your
  Raspberry is getting _really_ comfortable. In this tutorial, a so-called
  `chainloader` is developed, which will be the last file you need to manually
  copy on the SD card for a while. It will enable you to load the tutorial
  kernels during boot on demand over `UART`.

![UART wiring diagram](doc/wiring.png)

## 🙌 Acknowledgements

The original version of the tutorials started out as a fork of [Zoltan
Baldaszti](https://github.com/bztsrc)'s awesome [tutorials on bare metal
programming on RPi3](https://github.com/bztsrc/raspi3-tutorial) in `C`. Thanks
for giving me a head start!

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

