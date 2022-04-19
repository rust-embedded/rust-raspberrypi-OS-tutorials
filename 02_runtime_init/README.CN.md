# 教程 02 - 执行初始化

## tl;dr

我们拓展了`boot.S`，在第一次启动的时候调用Rust代码。在Rust的代码中先清零了[bss] section，然后通过调用`panic()`挂起CPU。再次运行`make qemu`看看新增加的代码是怎么运行的。

## 值得注意的变化

- 链接脚本（linker script）中有了更多的section。
     - `.rodata`, `.data`
     - `.bss`
- `_start()`:
     - 当核心不是`core0`第0号核心的时候，挂起该CPU核心。
     - `core0`会调用Rust的函数`runtime_init()`。
- `runtime_init.rs`内的`runtime_init()`
     - 清零了`.bss` section.
     - 它调用了`kernel_init()`, 这个函数又调用了`panic!()`, panic函数最终把`core0`和其他核心一样挂起了。

[bss]: https://en.wikipedia.org/wiki/.bss

## 相比之前的变化（diff）

Please check [the english version](README.md#diff-to-previous), which is kept up-to-date.
