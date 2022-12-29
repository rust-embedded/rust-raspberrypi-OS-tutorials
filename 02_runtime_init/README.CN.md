# 教程 02 - 执行初始化

## tl;dr

- 我们拓展了`boot.S`，在第一次启动的时候调用Rust代码。
  在跳转到rust代码前，对运行时进行了一些初始化工作。
- Rust通过调用`panic()`挂起CPU。
- 再次运行`make qemu`看看新增加的代码是怎么运行的。

## 值得注意的变化

- 链接脚本（linker script）中的变化:
     - 新程序段（sections）: `.rodata`, `.got`, `.data`, `.bss`.
     - 使用一个独立的位置（`.text._start_arguments`）来保存`_start()`引导函数所使用的参数。
- `_start()` in `_arch/__arch_name__/cpu/boot.s`:
     1. 当核心不是`core0`第0号核心的时候，挂起该CPU核心。
     1. 通过清零`.bss`程序段来初始化`DRAM`.
     1. 初始化堆栈指针（`stack pointer`）.
     1. 跳转到`arch/__arch_name__/cpu/boot.rs`文件中定义的`_start_rust()`函数
- `_start_rust()`:
     1. 它调用了`kernel_init()`, 这个函数又调用了`panic!()`, panic函数最终把`core0`和其他核心一样挂起了。
- 目前依赖 [aarch64-cpu] 程序库, 这个库零成本的包装了处理 CPU 资源时的“不安全”部分。
    - 详细请参考 `_arch/__arch_name__/cpu.rs`.

[bss]: https://en.wikipedia.org/wiki/.bss
[aarch64-cpu]: https://github.com/rust-embedded/aarch64-cpu

## 相比之前的变化（diff）
请检查[英文版本](README.md#diff-to-previous)，这是最新的。

