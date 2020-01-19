# Rust 开发树莓派操作系统教程

![](https://github.com/rust-embedded/rust-raspi3-OS-tutorials/workflows/BSP-RPi3/badge.svg) ![](https://github.com/rust-embedded/rust-raspi3-OS-tutorials/workflows/BSP-RPi4/badge.svg) ![](https://github.com/rust-embedded/rust-raspi3-OS-tutorials/workflows/Unit-Tests/badge.svg) ![](https://github.com/rust-embedded/rust-raspi3-OS-tutorials/workflows/Integration-Tests/badge.svg) ![](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue)

<br/>

<img src="doc/header.jpg" height="390"> <img src="doc/minipush_demo_frontpage.gif" height="390">

## ℹ️ 介绍

该系列教程适用于 ARM64 位[ARMv8-A 架构]的业余操系统开发者。该教程会从零开始，一步步地指导你如何开发一个[功能健全的]
嵌入式操作系统的内核。包含了实现一般操作系统的任务，例如开发串口控制台，设置虚拟内存和处理硬件异常。
同时利用 Rust 的特性来提供系统的安全和速度。

_Cheers,
[Andre](https://github.com/andre-richter)_

[armv8-a 架构]: https://developer.arm.com/products/architecture/cpu-architecture/a-profile/docs
[功能健全的]: https://en.wikipedia.org/wiki/Monolithic_kernel

## 📑 教程结构

- 每篇教程都包含一个独立可引导的二进制内核文件。
- 每篇新的教程都在之前的基础上扩展。
- 每篇教程的指南里面都有一个简短的章节来总结新增的代码和功能，也会展示源代码的区别，方便检查和同步。
- 部分教程中有除了`tl;dr`章节外还有非常详细、具体的介绍。长期计划是所有的教程都会有详细的文字说明。但是现在我认为教程独特的地方是`tl;dr`和`diff`还不够详细。
- 教程中所用的代码可以在**树莓派 3 和 4**上运行。
  - 教程的第一到五章是基础内容，只能运行在`QEMU`上。
  - 到了[第六章]时(06_drivers_gpio_uart)，你可以在树莓派上加载和运行内核并通过`UART`来观察输出结果。
- 虽然这些教程是以树莓派 3 和 4 为试验对象，但代码是模块化的，所以应该容易移植到其他 CPU 架构的开发板上。
  - 我希望会有人有机会去实现**RISC-v**架构的代码。
- 我推荐使用[Visual Studio Code],配置[Rust Language Server]插件开发代码。
- 除了文本教程之外，也可以用`make doc`命令利用网页的优势来浏览代码。

[visual studio code]: https://code.visualstudio.com
[rust language server]: https://github.com/rust-lang/rls

## 🚀 易用性

这个系列的教程会着重关注用户体验的友好性。因此，我尽量消除嵌入式开发中的最大痛点：工具链的问题。

想要尝试代码的开发者们将不会因为复杂工具链的安装、编译而烦恼。
这是通过尽可能的使用 rust 的标准工具链并由相应的 docker 容器来提供其他所有的工具链实现的。
该容器会在需要的时候自动下载，开发者只需要安装好对应发行版的 docker 即可。

开发环境设置由以下部分组成：

- 编译器，链接器和二进制工具用 Rust 最新版所提供的。
- 额外的操作系统开发工具如：`QEMU`和`GDB`通过[容器](docker/rustembedded-osdev-utils)提供。

如果你想了解更多关于这个容器的信息，可以看看[这里](https://github.com/rust-embedded/rust-raspi3-OS-tutorials/tree/master/docker)

## 🛠 前提条件

在你开始之前，你必须安装一个合适的 Rust 工具链：

```bash
curl https://sh.rustup.rs -sSf             \
    |                                      \
    sh -s --                               \
    --default-toolchain nightly-2019-12-20 \
    --component rust-src llvm-tools-preview rustfmt rls rust-analysis

source $HOME/.cargo/env
cargo install cargo-xbuild cargo-binutils
```

## 📟 USB 串行输出

由于教程中开发的内核是在真实的硬件上运行的，因此强烈建议您使用 USB 串行调试线来进行试验。连接后调试线会为树莓派供电，
所以不需要额外供电。

- 淘宝搜索"USB 转串口"
- 如下图连接 GPIO 串口的 14/15 号引脚
- [第六章](06_drivers_gpio_uart) 是这个设备第一次需要使用的地方。找到如何准备 SD 卡来引导你自制的内核的说明。
- [第七章](07_uart_chainloader)开始，在树莓派上启动内核变得非常舒适。在这章，会开发出一个叫`chainloader`的文件。
  这将是您暂时需要在 SD 卡上手动复制的最后一个文件。这将使您能够在通过 UART 按需引导期间加载教程内核。

![UART wiring diagram](doc/wiring.png)

## 🙌 致谢

这个教程最初是由[Zoltan Baldaszti](https://github.com/bztsrc)的[项目](https://github.com/bztsrc/raspi3-tutorial)衍生出来的，感谢它给我开了一个头。

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### 贡献

除非您明确声明，否则有意提交给您的任何贡献（包括 Apache-2.0 许可中定义的）均应按上述双重许可，且无任何附加条款或条件。
