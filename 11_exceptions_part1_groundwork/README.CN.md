# 教程11 - 异常第一部分: 基础工作

## tl;dr

- 我们为所有的架构`CPU exceptions`奠定了基础。
- 目前，仅通过`panic!`调用打印详细的系统状态，并停止执行
- 这将有助于在开发和运行时发现错误。
- 出于演示目的，MMU的`page faults`用于演示(i)从异常返回，以及
  (ii)默认的`panic!`行为。

## 目录

- [介绍](#介绍)
- [异常类型](#异常类型)
- [异常条目](#异常条目)
    * [异常向量](#异常向量)
- [处理程序代码和偏移量](#处理程序代码和偏移量)
- [Rust和Assembly实现](#Rust和Assembly实现)
    * [上下文保存和还原](#上下文保存和还原)
    * [异常矢量表](#异常矢量表)
    * [实现处理程序](#实现处理程序)
- [引发异常 - 测试代码](#引发异常---测试代码)
- [测试](#测试)
- [相比之前的变化](#相比之前的变化)

## 介绍

现在我们正在`EL1`中执行，并且已经激活了`MMU`，是时候实现`CPU exceptions`了。
目前，我们只建立了一个具有非常基本功能的脚手架，它将帮助我们一路发现错误。
后续的`Interrupt`教程将继续我们在这里开始的工作。

请注意，本教程特定于`AArch64`架构。 它还不包含任何通用异常处理代码。

## 异常类型

在`AArch64`中，区分了四种类型的异常。它们是：
- Synchronous
    - 例如，`data abort`、`page fault` 或 `system call`. 它们的发生是执行某个 CPU 指令的直接结果
      因此是*synchronously*的。
- Interrupt Request (`IRQ`)
    - 例如，外部设备（如定时器）正在声明物理中断线。IRQs*asynchronously*发生。
- Fast Interrupt Request (`FIQ`)
    - 这些基本上是优先于普通 IRQ 的中断，并且具有更多特征，使它们适合实现超快速处理。
      但是，这超出了本教程的范围。 为了保持这些教程的紧凑和简洁，我们将或多或少地忽略FIQ，
      并仅实现一个会停止 CPU 内核的虚拟处理程序。
- System Error (`SError`)
    - 与IRQ一样，SErrors也是异步发生的，并且在技术上或多或少是相同的。它们的目的是发出系统中相当致命的错误信号，
      例如，如果`SoC`互相连接的事务超时。它们是非常特定于实现的，由`SoC`供应商决定哪些事件作为SError
      而不是普通的IRQ来传递。

## 异常条目

我建议阅读[ARMv8 架构参考手册][ARMv8_Manual]的第 1874-1876 页来了解异常处理机制。

以下是本教程重要功能的摘录：
- 异常条目将处理器移至相同或更高的`Exception Level`，但绝不会移至较低的`EL`。
- 程序状态保存在目标`EL`处的`SPSR_ELx`寄存器中。
- 首选返回地址保存在`ELR_ELx`寄存器中。
    - 这里的"Preferred"是指`ELR_ELx`可以保存引起异常(`synchronous case`)的指令的指令地址，或者由于`asynchronous`
      异常而未完成的第一条指令的指令地址。详细信息请参见[ARMv8 架构参考手册][ARMv8_Manual]的D1.10.1 章。
- 所有类型的异常都会在发生异常时关闭，因此默认情况下，异常处理程序本身不会被中断。
- 发生异常将选择目标`EL`的专用堆栈指针。
    - 例如，如果`EL0`中发生异常，堆栈指针选择寄存器`SPSel`将从`0`切换到`1`，这意味着除非您明确将其切换回`SP_EL0`，
      否则异常向量代码将使用`SP_EL1`。


### 异常向量

`AArch64`共有`16`个异常向量。已经引入的四种类型中的每一种都有一个，此外，还*考虑*了例外的来源和情况。

以下是[ARMv8 架构参考手册][ARMv8_Manual]的D1.10.2 章中所示决策表的副本：

[ARMv8_Manual]: https://developer.arm.com/docs/ddi0487/latest/arm-architecture-reference-manual-armv8-for-armv8-a-architecture-profile

<table>
    <thead>
        <tr>
            <th rowspan=2>Exception taken from </th>
            <th colspan=4>Offset for exception type</th>
        </tr>
        <tr>
            <th>Synchronous</th>
            <th>IRQ or vIRQ</th>
            <th>FIQ or vFIQ</th>
            <th>SError or vSError</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td width="40%">Current Exception level with SP_EL0.</td>
            <td align="center">0x000</td>
            <td align="center">0x080</td>
            <td align="center">0x100</td>
            <td align="center">0x180</td>
        </tr>
        <tr>
            <td>Current Exception level with SP_ELx, x>0.</td>
            <td align="center">0x200</td>
            <td align="center">0x280</td>
            <td align="center">0x300</td>
            <td align="center">0x380</td>
        </tr>
        <tr>
            <td>Lower Exception level, where the implemented level immediately lower than the target level is using AArch64.</td>
            <td align="center">0x400</td>
            <td align="center">0x480</td>
            <td align="center">0x500</td>
            <td align="center">0x580</td>
        </tr>
        <tr>
            <td>Lower Exception level, where the implemented level immediately lower than the target level is using AArch32.</td>
            <td align="center">0x600</td>
            <td align="center">0x680</td>
            <td align="center">0x700</td>
            <td align="center">0x780</td>
        </tr>
    </tbody>
</table>

由于我们的内核在`EL1`中运行，使用`SP_EL1`，如果我们会导致同步异常，则会执行偏移量为`0x200`的异常向量。
但这到底意味着什么？

## 处理程序代码和偏移量

在许多体系结构中，操作系统通过编译一个体系结构定义的数据结构来注册其异常处理程序（也称为向量），
该数据结构存储指向不同处理程序的函数指针。这可以像普通的函数指针数组一样简单。 然后，该数据结构的`base address`
被存储到专用寄存器中，以便CPU可以在发生异常时跳转到相应的处理函数。例如，经典的`x86_64`架构就遵循这一原则。

在`AArch64`中，情况有点不同。在这里，我们还有一个特殊用途的寄存器，称为`VBAR_EL1`：向量基地址寄存器。

但是，它不存储函数指针数组的基地址，而是存储包含16个处理程序的**内存位置的代码**的内存位置。一个处理程序紧接着
另一个处理程序。每个处理程序最多可以占用`0x80`字节，即`128`字节的空间。这就是为什么上面的表格显示`offsets`：
为了指示某个处理程序从哪个偏移量开始。

当然，您没有义务将所有处理程序代码都塞进128个字节中。您可以随时自由地跳转到任何其他功能。实际上，无论如何，
在大多数情况下这是需要的，因为上下文保存代码本身就会占用大部分可用空间（您很快就会了解什么是上下文保存）。

此外，还要求`Vector Base Address`与`0x800`（即`2048`字节）对齐。

## Rust和Assembly实现

该实现混合使用了`Rust`和`Assembly`代码。

### 上下文保存和还原

与任何其他代码一样，异常向量使用一堆公共共享的处理器资源。最重要的是`AArch64`中每个核心提供的
`General Purpose Registers`(GPRs)集合 (`x0`-`x30`)。

为了在执行异常向量代码时不污染这些寄存器，通常的做法是将这些共享资源保存在内存中（准确地说是堆栈）作为第一个操作。
这通常被描述为*保存上下文*。 然后，异常向量代码可以毫不费力地在自己的代码中使用共享资源，
并且作为从异常处理代码返回之前的最后一个操作，恢复上下文，以便处理器可以在处理异常之前从中断处继续。

上下文保存和恢复是系统软件中为数不多的无法绕过手动组装的地方之一。引入`exception.s`:

```asm
/// Call the function provided by parameter `\handler` after saving the exception context. Provide
/// the context as the first parameter to '\handler'.
.macro CALL_WITH_CONTEXT handler
__vector_\handler:
	// Make room on the stack for the exception context.
	sub	sp,  sp,  #16 * 17

	// Store all general purpose registers on the stack.
	stp	x0,  x1,  [sp, #16 * 0]
	stp	x2,  x3,  [sp, #16 * 1]
	stp	x4,  x5,  [sp, #16 * 2]
	stp	x6,  x7,  [sp, #16 * 3]
	stp	x8,  x9,  [sp, #16 * 4]
	stp	x10, x11, [sp, #16 * 5]
	stp	x12, x13, [sp, #16 * 6]
	stp	x14, x15, [sp, #16 * 7]
	stp	x16, x17, [sp, #16 * 8]
	stp	x18, x19, [sp, #16 * 9]
	stp	x20, x21, [sp, #16 * 10]
	stp	x22, x23, [sp, #16 * 11]
	stp	x24, x25, [sp, #16 * 12]
	stp	x26, x27, [sp, #16 * 13]
	stp	x28, x29, [sp, #16 * 14]

	// Add the exception link register (ELR_EL1), saved program status (SPSR_EL1) and exception
	// syndrome register (ESR_EL1).
	mrs	x1,  ELR_EL1
	mrs	x2,  SPSR_EL1
	mrs	x3,  ESR_EL1

	stp	lr,  x1,  [sp, #16 * 15]
	stp	x2,  x3,  [sp, #16 * 16]

	// x0 is the first argument for the function called through `\handler`.
	mov	x0,  sp

	// Call `\handler`.
	bl	\handler

	// After returning from exception handling code, replay the saved context and return via
	// `eret`.
	b	__exception_restore_context

.size	__vector_\handler, . - __vector_\handler
.type	__vector_\handler, function
.endm
```

首先，定义一个用于保存上下文的宏。 它最终跳转到后续处理程序代码，并最终恢复上下文。事先，我们在堆栈上为上下文预留空间。
也就是说，30个`GPRs`，`link register`，`exception link register`（保存首选返回地址），
`saved program status`和`exception syndrome register`。之后，我们存储这些寄存器，将当前堆栈地址保存在
`x0`中，并跳转到后续处理程序代码，其函数名作为参数提供给宏(`\handler`)。

处理程序代码将用Rust编写，但使用平台的`C` ABI。这样，我们可以定义一个函数签名，函数签名将指向堆栈上的上下文数据
的指针作为其第一个参数，并且知道该参数预计位于`x0`寄存器中。我们需要在这里使用`C` ABI，因为`Rust`没有稳定的实现
参考([Issue](https://github.com/rust-lang/rfcs/issues/600)).

### 异常矢量表

接下来，我们制作异常向量表：

```asm
// Align by 2^11 bytes, as demanded by ARMv8-A. Same as ALIGN(2048) in an ld script.
.align 11

// Export a symbol for the Rust code to use.
__exception_vector_start:

// Current exception level with SP_EL0.
//
// .org sets the offset relative to section start.
//
// # Safety
//
// - It must be ensured that `CALL_WITH_CONTEXT` <= 0x80 bytes.
.org 0x000
	CALL_WITH_CONTEXT current_el0_synchronous
.org 0x080
	CALL_WITH_CONTEXT current_el0_irq
.org 0x100
	FIQ_SUSPEND
.org 0x180
	CALL_WITH_CONTEXT current_el0_serror

// Current exception level with SP_ELx, x > 0.
.org 0x200
	CALL_WITH_CONTEXT current_elx_synchronous
.org 0x280
	CALL_WITH_CONTEXT current_elx_irq
.org 0x300
	FIQ_SUSPEND
.org 0x380
	CALL_WITH_CONTEXT current_elx_serror

[...]
```

请注意每个向量如何使用`.org`指令从节开始处所需的偏移量开始。每个宏调用都会引入一个显式处理函数名称，该函数名称在
`exception.rs`中用`Rust`实现。

### 实现处理程序

文件`exception.rs`首先定义了异常上下文的`struct`，该结构由汇编代码存储在堆栈上:

```rust
/// The exception context as it is stored on the stack on exception entry.
#[repr(C)]
struct ExceptionContext {
    /// General Purpose Registers.
    gpr: [u64; 30],

    /// The link register, aka x30.
    lr: u64,

    /// Exception link register. The program counter at the time the exception happened.
    elr_el1: u64,

    /// Saved program status.
    spsr_el1: SpsrEL1,

    // Exception syndrome register.
    esr_el1: EsrEL1,
}
```

处理程序采用`struct ExceptionContext`参数。由于我们还不打算为每个异常实现处理程序，因此提供了一个默认处理程序:

```rust
/// Prints verbose information about the exception and then panics.
fn default_exception_handler(exc: &ExceptionContext) {
    panic!(
        "CPU Exception!\n\n\
        {}",
        exc
    );
}
```

从程序集中引用的实际处理程序现在可以暂时跳转到它，例如:

```rust
#[no_mangle]
extern "C" fn current_elx_irq(e: &mut ExceptionContext) {
    default_exception_handler(e);
}
```

## 引发异常 - 测试代码

我们希望看到两个实际案例：
1. 异常的获取、处理和返回是如何工作的。
2. 未处理异常的`panic!`宏打印是什么样子的。

因此，通过调用在`main.rs`中设置异常之后的函数来引发异常:

```rust
exception::handling_init();
```

我们通过从内存地址`8 GiB`读取来引发数据中止异常:

```rust
// Cause an exception by accessing a virtual address for which no translation was set up. This
// code accesses the address 8 GiB, which is outside the mapped address space.
//
// For demo purposes, the exception handler will catch the faulting 8 GiB address and allow
// execution to continue.
info!("");
info!("Trying to read from address 8 GiB...");
let mut big_addr: u64 = 8 * 1024 * 1024 * 1024;
unsafe { core::ptr::read_volatile(big_addr as *mut u64) };
```

这会触发我们的异常代码，因为我们尝试从尚未安装映射的虚拟地址读取。请记住，在上一教程中我们仅映射了最多
`4 GiB`的地址空间。

为了避免出现这种异常，相应的处理程序有一个特殊的演示案例：

```rust
#[no_mangle]
extern "C" fn current_elx_synchronous(e: &mut ExceptionContext) {
    if e.fault_address_valid() {
        let far_el1 = FAR_EL1.get();

        // This catches the demo case for this tutorial. If the fault address happens to be 8 GiB,
        // advance the exception link register for one instruction, so that execution can continue.
        if far_el1 == 8 * 1024 * 1024 * 1024 {
            e.elr_el1 += 4;

            return;
        }
    }

    default_exception_handler(e);
}
```

它检查错误地址是否等于`8 GiB`如果是，则将`ELR`的副本前进4，以便它指向引起异常的指令之后的下一条指令。
当处理程序返回时，我们之前介绍的汇编宏将继续执行。该宏只剩下一行: `b __exception_restore_context`，
它跳转到一个汇编函数，该函数在最终执行`eret`返回异常之前演示我们保存的上下文。

这将使我们回到`main.rs`。但我们也想看到`panic!`宏打印。

因此，第二次读取完成，这次是从地址`9 GiB`开始。处理程序无法捕获的情况，最终引发`panic!`从默认处理程序调用。

## 测试

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
[MP] ⏩ Pushing 64 KiB =========================================🦀 100% 0 KiB/s Time: 00:00:00
[ML] Loaded! Executing the payload now

[    0.798323] mingo version 0.11.0
[    0.798530] Booting on: Raspberry Pi 3
[    0.798985] MMU online. Special regions:
[    0.799462]       0x00080000 - 0x0008ffff |  64 KiB | C   RO PX  | Kernel code and RO data
[    0.800480]       0x3f000000 - 0x4000ffff |  17 MiB | Dev RW PXN | Device MMIO
[    0.801369] Current privilege level: EL1
[    0.801845] Exception handling state:
[    0.802290]       Debug:  Masked
[    0.802680]       SError: Masked
[    0.803069]       IRQ:    Masked
[    0.803459]       FIQ:    Masked
[    0.803849] Architectural timer resolution: 52 ns
[    0.804423] Drivers loaded:
[    0.804759]       1. BCM PL011 UART
[    0.805182]       2. BCM GPIO
[    0.805539] Timer test, spinning for 1 second
[    1.806070]
[    1.806074] Trying to read from address 8 GiB...
[    1.806624] ************************************************
[    1.807316] Whoa! We recovered from a synchronous exception!
[    1.808009] ************************************************
[    1.808703]
[    1.808876] Let's try again
[    1.809212] Trying to read from address 9 GiB...
[    1.809776] Kernel panic!

Panic location:
      File 'src/_arch/aarch64/exception.rs', line 58, column 5

CPU Exception!

ESR_EL1: 0x96000004
      Exception Class         (EC) : 0x25 - Data Abort, current EL
      Instr Specific Syndrome (ISS): 0x4
FAR_EL1: 0x0000000240000000
SPSR_EL1: 0x600003c5
      Flags:
            Negative (N): Not set
            Zero     (Z): Set
            Carry    (C): Set
            Overflow (V): Not set
      Exception handling state:
            Debug  (D): Masked
            SError (A): Masked
            IRQ    (I): Masked
            FIQ    (F): Masked
      Illegal Execution State (IL): Not set
ELR_EL1: 0x00000000000845f8

General purpose register:
      x0 : 0x0000000000000000         x1 : 0x0000000000086187
      x2 : 0x0000000000000027         x3 : 0x0000000000081280
      x4 : 0x0000000000000006         x5 : 0x1e27329c00000000
      x6 : 0x0000000000000000         x7 : 0xd3d18908028f0243
      x8 : 0x0000000240000000         x9 : 0x0000000000086187
      x10: 0x0000000000000443         x11: 0x000000003f201000
      x12: 0x0000000000000019         x13: 0x00000000ffffd8f0
      x14: 0x000000000000147b         x15: 0x00000000ffffff9c
      x16: 0x000000000007fd38         x17: 0x0000000005f5e0ff
      x18: 0x00000000000c58fc         x19: 0x0000000000090008
      x20: 0x0000000000085fc0         x21: 0x000000003b9aca00
      x22: 0x0000000000082238         x23: 0x00000000000813d4
      x24: 0x0000000010624dd3         x25: 0xffffffffc4653600
      x26: 0x0000000000086988         x27: 0x0000000000086080
      x28: 0x0000000000085f10         x29: 0x0000000000085c00
      lr : 0x00000000000845ec
```

## 相比之前的变化
请检查[英文版本](README.md#diff-to-previous)，这是最新的。
