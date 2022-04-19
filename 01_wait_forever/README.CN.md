# 教程 01 - 一直等待（死循环）

## tl;dr

项目框架已经构建完成；目前代码做的仅仅是挂起CPU核心执行内核代码。

-  `Makefile` 目标项：
    - `doc`: 生成文档。
    - `qemu`: 在 QEMU 中运行 `kernel`。
    - `clippy`
    - `clean`
    - `readelf`: 检查 `ELF` 输出。
    - `objdump`: 检查汇编。
    - `nm`: 检查符号。
- 代码按照 `kernel`， `arch` 和 `BSP` （板级支持包）的形式组织。
    - 条件编译会根据用户提供的参数编译各自的  `arch` 和  `BSP` 的内容。
- 自定义 `kernel.ld` 链接脚本.
    - 载入地址为 `0x80_000`
    - 目前仅有 `.text` 小节（section）。
- `main.rs`: 重要的 [inner attributes]:
    - `#![no_std]`, `#![no_main]`
- 汇编函数 `_start()` 会执行  `wfe` (Wait For Event)， 并挂起所有正在执行  `_start()` 的核心。
- 我们（必须）定义一个 `#[panic_handler]` 函数。
    - 用于等待cpu事件的发生。

[inner attributes]: https://doc.rust-lang.org/reference/attributes.html

### 测试一下！

在项目文件夹下调用 QEMU 并观察在 `wfe` 中CPU核心的运转情况：
```console
» make qemu
[...]
IN:
0x00080000:  d503205f  wfe
0x00080004:  17ffffff  b        #0x80000
```
