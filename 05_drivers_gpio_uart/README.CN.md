# 教程 05 - 驱动程序: GPIO和UART

## tl;dr

- 添加了用于真实`UART`和`GPIO`控制器的驱动程序。
- **我们将首次能够在真实硬件上运行代码** (请向下滚动查看说明)。

## 简介

在上一篇教程中，我们启用了全局安全变量，为添加第一个真实设备驱动程序奠定了基础。
我们放弃了神奇的QEMU控制台，并引入了一个`驱动程序管理器`，允许`BSP`将设备驱动程序注册到`内核`中。

## 驱动程序管理器

第一步是向内核添加一个`driver subsystem`。相应的代码将位于`src/driver.rs`中。
该子系统引入了`interface::DeviceDriver`，这是每个设备驱动程序都需要实现的通用特征，并为内核所知。
在同一文件中实例化的全局`DRIVER_MANAGER`实例（类型为`DriverManager`）作为一个中央实体，可以被调用来管理内核中的所有设备驱动程序。
例如，通过使用全局可访问的`crate::driver::driver_manager().register_driver(...)`，任何代码都可以注册一个实现了`interface::DeviceDriver`特征的具有静态生命周期的对象。

在内核初始化期间，调用`crate::driver::driver_manager().init_drivers(...)`将使驱动程序管理器遍历所有已注册的驱动程序，
并启动它们的初始化，并执行可选的`post-init callback`，该回调可以与驱动程序一起注册。
例如，此机制用于在`UART`驱动程序初始化后将其切换为主系统控制台的驱动程序。

## BSP驱动程序实现

在`src/bsp/raspberrypi/driver.rs`中，函数`init()`负责注册`UART`和`GPIO`驱动程序。
因此，在内核初始化期间，按照以下来自`main.rs`的代码，正确的顺序是：
（i）首先初始化BSP驱动程序子系统，然后（ii）调用`driver_manager()`。

```rust
unsafe fn kernel_init() -> ! {
    // Initialize the BSP driver subsystem.
    if let Err(x) = bsp::driver::init() {
        panic!("Error initializing BSP driver subsystem: {}", x);
    }

    // Initialize all device drivers.
    driver::driver_manager().init_drivers();
    // println! is usable from here on.
```



驱动程序本身存储在`src/bsp/device_driver`中，并且可以在不同的`BSP`之间重复使用
在这些教程中添加的第一个驱动程序是`PL011Uart`驱动程序：它实现了`console::interface::*`特征，并且从现在开始用作主系统控制台。
第二个驱动程序是`GPIO`驱动程序，它根据需要将`RPii's`的`UART`映射（即将来自`SoC`内部的信号路由到实际的硬件引脚）。
请注意，`GPIO`驱动程序区分**RPi 3**和**RPi 4**。它们的硬件不同，因此我们必须在软件中进行适配。

现在，`BSP`还包含了一个内存映射表，位于`src/bsp/raspberrypi/memory.rs`中。它提供了树莓派的`MMIO`地址，
`BSP`使用这些地址来实例化相应的设备驱动程序，以便驱动程序代码知道在内存中找到设备的寄存器的位置。

## SD卡启动

由于我们现在有了真实的`UART`输出，我们可以在真实的硬件上运行代码。
由于前面提到的`GPIO`驱动程序的差异，构建过程在**RPi 3**和**RPi 4**之间有所区别。
默认情况下，所有的`Makefile`目标都将为**RPi 3**构建。
为了**RPi 4**构建，需要在每个目标前加上`BSP=rpi4`。例如：

```console
$ BSP=rpi4 make
$ BSP=rpi4 make doc
```

不幸的是，QEMU目前还不支持**RPi 4**，因此`BSP=rpi4 make qemu`无法工作。

**准备SD卡的一些步骤在RPi3和RPi4之间有所不同，请在以下操作中小心。**

### 通用步骤

1. 创建一个名为`boot`的`FAT32`分区。
2. 在SD卡上生成一个名为`config.txt`的文件，并将以下内容写入其中：

```txt
arm_64bit=1
init_uart_clock=48000000
```
### RPi 3

3. 从[Raspberry Pi firmware repo](https://github.com/raspberrypi/firmware/tree/master/boot)中将以下文件复制到SD卡上：
    - [bootcode.bin](https://github.com/raspberrypi/firmware/raw/master/boot/bootcode.bin)
    - [fixup.dat](https://github.com/raspberrypi/firmware/raw/master/boot/fixup.dat)
    - [start.elf](https://github.com/raspberrypi/firmware/raw/master/boot/start.elf)
4. 运行`make`命令。

### RPi 4

3. 从[Raspberry Pi firmware repo](https://github.com/raspberrypi/firmware/tree/master/boot)中将以下文件复制到SD卡上：
    - [fixup4.dat](https://github.com/raspberrypi/firmware/raw/master/boot/fixup4.dat)
    - [start4.elf](https://github.com/raspberrypi/firmware/raw/master/boot/start4.elf)
    - [bcm2711-rpi-4-b.dtb](https://github.com/raspberrypi/firmware/raw/master/boot/bcm2711-rpi-4-b.dtb)
4. 运行`BSP=rpi4 make`命令。


_**注意**: 如果在您的RPi4上无法正常工作，请尝试将`start4.elf`重命名为`start.elf` (不带4)
并复制到SD卡上。_

### 再次通用步骤

5. 将`kernel8.img`复制到SD卡上，并将SD卡插入RPi。
6. 运行`miniterm` target，在主机上打开UART设备：

```console
$ make miniterm
```

> ❗ **注意**: `Miniterm`假设默认的串行设备名称为`/dev/ttyUSB0`。Depending on your
> 根据您的主机操作系统，设备名称可能会有所不同。例如，在`macOS`上，它可能是
> `/dev/tty.usbserial-0001`之类的。在这种情况下，请明确提供设备名称：


```console
$ DEV_SERIAL=/dev/tty.usbserial-0001 make miniterm
```

7. 将USB串口连接到主机PC。
    - 请参考[top-level README](../README.md#-usb-serial-output)中的接线图。
    - **注意**: TX（发送）线连接到RX（接收）引脚。
    - 确保您**没有**连接USB串口的电源引脚，只连接RX/TX和GND引脚。
8. 将RPi连接到（USB）电源线，并观察输出。

```console
Miniterm 1.0

[MT] ⏳ Waiting for /dev/ttyUSB0
[MT] ✅ Serial connected
[0] mingo version 0.5.0
[1] Booting on: Raspberry Pi 3
[2] Drivers loaded:
      1. BCM PL011 UART
      2. BCM GPIO
[3] Chars written: 117
[4] Echoing input now
```

8. 通过按下<kbd>ctrl-c</kbd>退出。

## 相比之前的变化（diff）
请检查[英文版本](README.md#diff-to-previous)，这是最新的。
