# 教程 07 - 时间戳

## tl;dr

- 我们为计时器硬件添加了抽象，并在`_arch/aarch64`中实现了ARM架构计时器。
- 新的计时器函数用于给UART打印添加时间戳，并且用于消除`GPIO`设备驱动中基于周期的延迟，从而提高准确性。
- 添加了`warn!()`宏。

## 测试它

请通过 chainboot 进行检查（在上一个教程中添加）。
```console
$ make chainboot
[...]
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
[MP] ⏩ Pushing 12 KiB =========================================🦀 100% 0 KiB/s Time: 00:00:00
[ML] Loaded! Executing the payload now

[    0.143123] mingo version 0.7.0
[    0.143323] Booting on: Raspberry Pi 3
[    0.143778] Architectural timer resolution: 52 ns
[    0.144352] Drivers loaded:
[    0.144688]       1. BCM PL011 UART
[    0.145110]       2. BCM GPIO
[W   0.145469] Spin duration smaller than architecturally supported, skipping
[    0.146313] Spinning for 1 second
[    1.146715] Spinning for 1 second
[    2.146938] Spinning for 1 second
```

## 相比之前的变化（diff）
请检查[英文版本](README.md#diff-to-previous)，这是最新的。
