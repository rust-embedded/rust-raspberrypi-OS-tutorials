# 教程 04 - 全局安全

## tl;dr

- 引入了假的锁。
- 这是第一次展示原始操作系统同步，并支持安全访问全局数据结构。

## Rust中的全局可变

当我们引入全局可用的`print!`宏在 [教程03]，我门有一点作弊。 调用
`core::fmt`的`write_fmt()`函数，接受`&mut self`的方法之所以有效，
是因为在每次调用时都会创建一个新的`QEMUOutput`实例。

如果我们想保留一些状态，例如关于写入字符数的统计数据，
我们需要创建`QEMUOutput`的一个全局实例 (在Rust中，使用`static`关键字).

然而`static QEMU_OUTPUT`不允许调用具有`&mut self`的函数。
为此，我们需要`static mut`，但是调用改变`static mut`状态的函数是不安全的。
这个是Rust编译器对此的推理，它无法再阻止核心/线程同时改变数据（它是全局的，所以每个人都可以从任何地方引用它，检查程序借用在这里帮不上忙）。


这个问题的解决方案是将全局封装到原始同步中。在我们的例子中，是一个*MUTual EXclusion*原语的变体。
`Mutex`是`synchronization.rs`中引入的一个特性，并由同一文件中的`NullLock`实现。
为了使代码更易于教学，它省略了用于防止并发访问的实际体系结构特定逻辑，因为只要内核仅在单个内核上执行并禁用中断，我们就不需要它。

`NullLock`侧重于展示Rust内部可变性的核心概念。请务必阅读它。
我们还建议您阅读这篇关于[Rust的引用类型的精确心智模型]文章

如果要将`NullLock`与一些真实的互斥实现进行比较，可以查看
[spin crate]或者[parking lot crate]。

[教程03]: ../03_hacky_hello_world
[内部可变性]: https://doc.rust-lang.org/std/cell/index.html
[Rust的引用类型的精确心智模型]: https://docs.rs/dtolnay/0.0.6/dtolnay/macro._02__reference_types.html
[spin crate]: https://github.com/mvdnes/spin-rs
[parking lot crate]: https://github.com/Amanieu/parking_lot

## 测试

```console
$ make qemu
[...]

[0] Hello from Rust!
[1] Chars written: 22
[2] Stopping here.
```

## 相比之前的变化（diff）
请检查[英文版本](README.md#diff-to-previous)，这是最新的。
