# 教程 08 - 使用JTAG进行硬件调试

## tl;dr

按照以下顺序进行操作：

1. 运行`make jtagboot`并保持终端打开。
2. 连接USB串行设备。
3. 连接`JTAG`调试器的USB设备。
4. 在新的终端中，运行`make openocd`并保持终端打开。
5. 在新的终端中，运行`make gdb`或者运行`make gdb-opt0`。

![Demo](../doc/09_demo.gif)

## 目录

- [简介](#简介)
- [大纲](#大纲)
- [软件设置](#软件设置)
- [硬件设置](#硬件设置)
  * [线路](#线路)
- [准备连接](#准备连接)
- [OpenOCD](#openocd)
- [GDB](#gdb)
  * [备注](#备注)
    + [优化](#优化)
    + [GDB控制](#GDB控制)
- [关于USB连接限制的注意事项](#关于USB连接限制的注意事项)
- [额外资料](#额外资料)
- [致谢](#致谢)
- [相比之前的变化（diff）](#相比之前的变化（diff）)

## 简介

在即将到来的教程中，我们将涉及RPi的SoC（系统芯片）的敏感区域，这可能会让我们的调试工作变得非常困难。
例如，改变处理器的`Privilege Level`或引入`Virtual Memory`。

硬件调试器有时可以成为寻找棘手错误的最后手段。特别是对于调试复杂的、与体系结构相关的硬件问题，它将非常有用，
因为在这个领域，`QEMU`有时无法提供帮助，因为它对硬件的某些特性进行了抽象，并没有模拟到最后一位。

那么，让我们介绍一下`JTAG`调试。一旦设置好，它将允许我们在真实的硬件上逐步执行我们的内核。这是多么酷啊！

## 大纲

从内核的角度来看，这个教程与之前的教程相同。我们只是在其周围添加了用于JTAG调试的基础设施。

## 软件设置

我们需要在SD卡的`config.txt`文件中添加另一行：

```toml
arm_64bit=1
init_uart_clock=48000000
enable_jtag_gpio=1
```

## 硬件设置

与我们WG的[Embedded Rust Book]书籍中使用的`STM32F3DISCOVERY`等微控制器板不同，RPi没有在其板上内置调试器。
因此，您需要购买一个。

在本教程中，我们将使用OLIMEX的[ARM-USB-TINY-H]。它具有标准的[ARM JTAG 20 connector]。
不幸的是，RPi没有这个连接器，所以我们必须通过跳线连接它。

[Embedded Rust Book]: https://rust-embedded.github.io/book/start/hardware.html
[ARM-USB-TINY-H]: https://www.olimex.com/Products/ARM/JTAG/ARM-USB-TINY-H
[ARM JTAG 20 connector]: http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dui0499dj/BEHEIHCE.html

### 线路

<table>
    <thead>
        <tr>
            <th>GPIO #</th>
			<th>Name</th>
			<th>JTAG #</th>
			<th>Note</th>
			<th width="60%">Diagram</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td></td>
            <td>VTREF</td>
            <td>1</td>
            <td>to 3.3V</td>
            <td rowspan="8"><img src="../doc/09_wiring_jtag.png"></td>
        </tr>
        <tr>
            <td></td>
            <td>GND</td>
            <td>4</td>
            <td>to GND</td>
        </tr>
        <tr>
            <td>22</td>
            <td>TRST</td>
            <td>3</td>
            <td></td>
        </tr>
        <tr>
            <td>26</td>
            <td>TDI</td>
            <td>5</td>
            <td></td>
        </tr>
        <tr>
            <td>27</td>
            <td>TMS</td>
            <td>7</td>
            <td></td>
        </tr>
        <tr>
            <td>25</td>
            <td>TCK</td>
            <td>9</td>
            <td></td>
        </tr>
        <tr>
            <td>23</td>
            <td>RTCK</td>
            <td>11</td>
            <td></td>
        </tr>
        <tr>
            <td>24</td>
            <td>TDO</td>
            <td>13</td>
            <td></td>
        </tr>
    </tbody>
</table>

<p align="center"><img src="../doc/09_image_jtag_connected.jpg" width="50%"></p>

## 准备连接

在启动时，由于我们对`config.txt`进行的更改，RPi的固件将配置相应的GPIO引脚以实现`JTAG`功能。

现在剩下的要做的就是暂停RPi的执行，然后通过`JTAG`进行连接。因此，我们添加了一个新的`Makefile` target，
`make jtagboot`，它使用`chainboot`方法将一个小型辅助二进制文件加载到RPi上，
该文件只是将执行核心置于等待状态。

文件夹中单独[X1_JTAG_boot]文件夹中单独维护，并且是我们迄今为止在教程中使用的内核的修改版本。

[X1_JTAG_boot]: ../X1_JTAG_boot

```console
$ make jtagboot
Minipush 1.0

[MP] ⏳ Waiting for /dev/ttyUSB0
[MP] ✅ Serial connected
[MP] 🔌 Please power the target now
 __  __ _      _ _                 _
|  \/  (_)_ _ (_) |   ___  __ _ __| |
| |\/| | | ' \| | |__/ _ \/ _` / _` |
|_|  |_|_|_||_|_|____\___/\__,_\__,_|

           Raspberry Pi 3

[ML] Requesting binary
[MP] ⏩ Pushing 7 KiB ==========================================🦀 100% 0 KiB/s Time: 00:00:00
[ML] Loaded! Executing the payload now

[    0.394532] Parking CPU core. Please connect over JTAG now.
```

保持USB串口连接和打开运行`jtagboot`的终端非常重要。当我们稍后加载实际的内核时，`UART`输出将显示在这里。

## OpenOCD

接下来，我们需要启动开放式片上调试器 [Open On-Chip Debugger]，也称为`OpenOCD`，以实际连接`JTAG`。

[Open On-Chip Debugger]: http://openocd.org

一如既往，我们的教程力求使开发工具的使用尽可能简单，
这就是为什么我们将所有内容打包到了[dedicated Docker container]中，该容器已经用于链式引导和`QEMU`。

[dedicated Docker container]: ../docker/rustembedded-osdev-utils

连接Olimex USB JTAG调试器，在同一个文件夹中打开一个新的终端窗口，然后按顺序输入
`make openocd`命令。你将会看到一些初始输出：

```console
$ make openocd
[...]
Open On-Chip Debugger 0.10.0
[...]
Info : Listening on port 6666 for tcl connections
Info : Listening on port 4444 for telnet connections
Info : clock speed 1000 kHz
Info : JTAG tap: rpi3.tap tap/device found: 0x4ba00477 (mfg: 0x23b (ARM Ltd.), part: 0xba00, ver: 0x4)
Info : rpi3.core0: hardware has 6 breakpoints, 4 watchpoints
Info : rpi3.core1: hardware has 6 breakpoints, 4 watchpoints
Info : rpi3.core2: hardware has 6 breakpoints, 4 watchpoints
Info : rpi3.core3: hardware has 6 breakpoints, 4 watchpoints
Info : Listening on port 3333 for gdb connections
Info : Listening on port 3334 for gdb connections
Info : Listening on port 3335 for gdb connections
Info : Listening on port 3336 for gdb connections
```

`OpenOCD`已检测到RPi的四个核心，并打开了四个网络端口，`gdb`现在可以连接到这些端口来调试各自的核心。

## GDB

最后，我们需要一个支持`AArch64`的`gdb`版本。你猜对了，它已经打包在osdev容器中。
可以通过`make gdb`命令启动它。

实际上，这个Makefile target做了更多的事情。它构建了一个包含调试信息的特殊版本的内核。
这使得`gdb`能够显示我们当前正在调试的`Rust`源代码行。
它还启动了`gdb`，以便它已经加载了这个调试构建（`kernel_for_jtag`）。

现在我们可以使用`gdb`命令行来进行以下操作：
  1. 在我们的内核中设置断点。
  2. 通过JTAG将内核加载到内存中（请记住，当前RPi仍在执行最小的JTAG引导二进制文件）。
  3. 操纵RPi的程序计数器，使其从我们内核的入口点开始执行。
  4. 逐步执行内核的执行过程。

```console
$ make gdb
[...]
>>> target remote :3333                          # Connect to OpenOCD, core0
>>> load                                         # Load the kernel into the RPi's DRAM over JTAG.
Loading section .text, size 0x2454 lma 0x80000
Loading section .rodata, size 0xa1d lma 0x82460
Loading section .got, size 0x10 lma 0x82e80
Loading section .data, size 0x20 lma 0x82e90
Start address 0x0000000000080000, load size 11937
Transfer rate: 63 KB/sec, 2984 bytes/write.
>>> set $pc = 0x80000                            # Set RPI's program counter to the start of the
                                                 # kernel binary.
>>> break main.rs:158
Breakpoint 1 at 0x8025c: file src/main.rs, line 158.
>>> cont
>>> step                                         # Single-step through the kernel
>>> step
>>> ...
```

### 备注

#### 优化

在调试操作系统二进制文件时，您需要在可以逐步执行源代码粒度和生成的二进制文件的优化级别之间进行权衡。
`make`和`make gdb`targets生成一个`--release`二进制文件，其中包含优化级别为3（`-opt-level=3`）。
然而，在这种情况下，编译器会非常积极地进行内联，并尽可能地将读取和写入操作打包在一起。
因此，不总是能够在源代码文件的特定行上准确命中断点。

因此，Makefile还提供了`make gdb-opt0` target，它使用了`-opt-level=0`。
因此，它将允许您拥有更精细的调试粒度。然而，请记住，当调试与硬件密切相关的代码时，
编译器对易失性寄存器的读取或写入进行压缩的优化可能会对执行产生重大影响。
请注意，上面的演示GIF是使用`gdb-opt0`录制的。

#### GDB控制

在某些情况下，您可能会遇到延迟循环或等待串行输入的代码。在这种情况下，
逐步执行可能不可行或无法正常工作。您可以通过在这些区域之外设置其他断点，从而跳过这些障碍。
并使用`cont`命令到达它们。

在`gdb`中按下`ctrl+c`将再次停止RPi的执行，以防止您在没有进一步断点的情况下继续执行。

## 关于USB连接限制的注意事项

如果您按照教程从头到尾进行操作，关于USB连接的一切应该都没问题。

但是，请注意，根据当前的形式，我们的`Makefile`对连接的USB设备的命名做出了隐含的假设。
它期望`/dev/ttyUSB0`是`UART`设备。

因此，请确保按照以下顺序将设备连接到您的计算机：
  1. 首先连接USB串行设备。
  2. 然后连接Olimex调试器。

这样，主机操作系统会相应地枚举这些设备。这只需要做一次即可。
可以多次断开和连接串行设备，例如在保持调试器连接的情况下启动不同的`make jtagboot`运行。

## 额外资料

- https://metebalci.com/blog/bare-metal-raspberry-pi-3b-jtag
- https://www.suse.com/c/debugging-raspberry-pi-3-with-jtag

## 致谢

感谢[@naotaco](https://github.com/naotaco)为本教程奠定了基础。

## 相比之前的变化（diff）
请检查[英文版本](README.md#diff-to-previous)，这是最新的。
