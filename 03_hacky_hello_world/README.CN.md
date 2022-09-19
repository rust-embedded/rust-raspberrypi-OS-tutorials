# 教程 03 - Hacky Hello World

## tl;dr

- 介绍全局的`println!()`宏以便尽早启用"printf debugging"。
- 为了保持教程长度合理，打印函数目前 "滥用" 了 QEMU 属性，该属性允许我们在没有正确设置的情况下使用树莓派的`UART`。
- 在接下来的教程中将逐步使用真实硬件的`UART`。

## 值得注意的补充

- `src/console.rs`为控制台命令和通过`console::console()`对内核控制台的全局访问引入了接口`Traits`。
- `src/bsp/raspberrypi/console.rs` 实现QEMU仿真UART的接口。
- 紧急处理程序使用新的`println!()`以显示用户错误消息。
- 有一个新的Makefile目录`make test`，用于自动测试。它在`QEMU`中引导编译后的内核，并检查内核生成的预期输出字符串。
  - 在本教程中，它检查字符串`Stopping here`，该字符串由`panic!()`在`main.rs`的末尾。

## 测试一下

QEMU不再以汇编模式运行。从现在起，它将显示`console`的输出。

```console
$ make qemu
[...]

Hello from Rust!
Kernel panic!

Panic location:
      File 'src/main.rs', line 126, column 5

Stopping here.
```

## 相比之前的变化（diff）
请检查[英文版本](README.md#diff-to-previous)，这是最新的。
