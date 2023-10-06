# Rust 开发树莓派操作系统教程

![](https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials/workflows/BSP-RPi3/badge.svg) ![](https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials/workflows/BSP-RPi4/badge.svg) ![](https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials/workflows/Unit-Tests/badge.svg) ![](https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials/workflows/Integration-Tests/badge.svg) ![](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue)

<br/>

<img src="doc/header.jpg" height="379"> <img src="doc/minipush_demo_frontpage.gif" height="379">

## ℹ️ 介绍

该系列教程适用于 ARM64 位[ARMv8-A 架构]的业余操作系统开发者。该教程会从零开始，一步步地指导你如何开发一个[功能健全的]
嵌入式操作系统的内核。包含了实现一般操作系统的任务，例如开发串口控制台，设置虚拟内存和处理硬件异常。
同时利用 Rust 的特性来提供系统的安全和速度。

祝你玩得开心！

_带上我最诚挚的问候,<br>Andre ([@andre-richter])_

[armv8-a 架构]: https://developer.arm.com/products/architecture/cpu-architecture/a-profile/docs
[功能健全的]: https://en.wikipedia.org/wiki/Monolithic_kernel
[@andre-richter]: https://github.com/andre-richter

## 📑 教程结构

- 每篇教程都包含一个独立可引导的二进制内核文件。
- 每篇新的教程都在之前的基础上扩展。
- 每篇教程的指南里面都有一个简短的章节来总结新增的代码和功能，也会展示源代码的区别，方便检查和同步。
- 部分教程中有除了`tl;dr`章节外还有非常详细、具体的介绍。长期计划是所有的教程都会有详细的文字说明。但是目前只有那些我认为教程的`tl;dr`和`diff`还不够详细的章节会详细说明。
- 教程中所用的代码可以在**树莓派 3 和 4**上运行。
  - 教程的第一到五章是基础内容，只能运行在`QEMU`上。
  - 到了[第六章]时(06_drivers_gpio_uart)，你可以在树莓派上加载和运行内核并通过`UART`来观察输出结果。
- 虽然这些教程是以树莓派 3 和 4 为试验对象，但代码是模块化的，所以应该容易移植到其他 CPU 架构的开发板上。
  - 我希望会有人有机会去实现**RISC-V**架构的代码。
- 我推荐使用[Visual Studio Code],配置[Rust Analyzer]插件开发代码。
- 除了文本教程之外，也可以用`make doc`命令利用网页的优势来浏览代码。

### `make doc` 的输出

![make doc](doc/make_doc.png)

[Visual Studio Code]: https://code.visualstudio.com
[Rust Analyzer]: https://rust-analyzer.github.io

## 🛠 系统要求

本教程主要是面向**Linux**发行版的。理论上，文中大部分内容在其他类Unix系统诸如**macOS**也能正常工作，但请注意，只是理论上。

### 🚀 tl;dr 版本

1. [安装 Docker][install_docker]。
2. **(仅限Linux)** 确保您的用户帐户在 [docker group] 中。
3. 准备Rust工具链。其中大部分将在首次使用时通过[rust-toolchain.toml](rust-toolchain.toml)文件进行处理。我们要做的是：
   1. 如果你已经安装了一个版本的Rust:
      ```bash
      cargo install cargo-binutils rustfilt
      ```

   1. 如果你想要全新安装:
      ```bash
      curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

      source $HOME/.cargo/env
      cargo install cargo-binutils rustfilt
      ```

4. 如果你使用 `Visual Studio Code`，我强烈推荐你安装[Rust Analyzer 扩展]。
5. **(仅限macOS)** 安装一些`Ruby` gems。

这是作者最后一次在`macOS Monterey`上用`Ruby 3.0.2`版本进行测试。如果您正在使用`rbenv`，那么相应的`.ruby-version`文件已经就位。
如果你从未听说过`rbenv`，请尝试使用[这个指南](https://stackoverflow.com/a/68118750)。

在存储库根文件夹中运行此操作：

```bash
bundle config set --local path '.vendor/bundle'
bundle config set --local without 'development'
bundle install
```

[docker group]: https://docs.docker.com/engine/install/linux-postinstall/
[Rust Analyzer 扩展]: https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer


## 🧰 长期版本: 消除工具链烦恼

这个系列的教程会着重关注用户体验的友好性。因此，我尽量消除嵌入式开发中的最大痛点：`Toolchain hassle`。

Rust内置的交叉编译支持在这方面帮了我们大忙。我们只需要使用`rustup`安装目标工具链就可以在`x86`宿主机上交叉编译支持树莓派的目标文件。然而，除了Rust编译器，我们还需要更多的工具。例如：
Rust本身在这方面已经起到了很大的作用，因为它内置了对交叉编译的支持。从`x86`宿主机到树莓派的`AArch64`架构的交叉编译所需的一切都将由`rustup`自动安装。然而，除了Rust编译器，我们还将使用更多的工具。例如：

- 用于在我们的宿主系统上模拟我们内核运行环境的`QEMU`。
- 一个叫`Minipush`的自制工具，可以通过`UART`将内核加载到树莓派上。
- 用于调式目标文件的`OpenOCD`和`GDB`。

在你的宿主机上安装/编译正确版本的上述工具很可能会遇到很多麻烦。举个例子，你的发行版也许并不会提供我们需要的最新版本的软件包。又或者你在编译这些工具时会遇到一些很难处理的依赖问题。

这也是为什么我们要尽可能使用[Docker][install_docker]的原因。我们提供了一个已经预装了所有需要的工具及依赖的容器，当需要的时候它就会被自动拉取。如果你想要了解更多有关Docker和这个容器的细节，请查看本仓库下的[docker](docker) 文件夹。

[install_docker]: https://docs.docker.com/get-docker/

## 📟 USB 串行输出

由于教程中开发的内核是在真实的硬件上运行的，因此强烈建议您使用 USB 串行调试线来进行试验。连接调试线后，树莓派需要通过额外电源供电。

- 您可以在[\[1\]] [\[2\]]中或者[淘宝]上找到USB转串口线，但许多其他线材也可以工作。理想情况下，您的线材应基于`CP2102`芯片。
- 您将其连接到`GND`和GPIO引脚`14/15`，如下所示。
- [教程5](05_drivers_gpio_uart/README.CN.md)是这个设备第一次需要使用的地方。查看它了解如何准备SD卡以从中启动自制内核的说明。
- 从[教程6](06_uart_chainloader/README.CN.md)开始，在树莓派上启动内核变得非常舒适。在本教程中开发了一个所谓的`chainloader`，。
  这将是您暂时需要在SD卡上手动复制的最后一个文件。这将使您能够在通过`UART`按需引导期间加载教程内核。

![UART wiring diagram](doc/wiring.png)

[\[1\]]: https://www.amazon.de/dp/B0757FQ5CX/ref=cm_sw_r_tw_dp_U_x_ozGRDbVTJAG4Q
[\[2\]]: https://www.adafruit.com/product/954
[淘宝]: https://www.taobao.com/

## 🙌 致谢

这个教程最初是由[Zoltan Baldaszti](https://github.com/bztsrc)的[项目](https://github.com/bztsrc/raspi3-tutorial)衍生出来的，感谢它给我开了一个头。

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### 贡献

除非您明确声明，否则有意提交给您的任何贡献（包括 Apache-2.0 许可中定义的）均应按上述双重许可，且无任何附加条款或条件。
