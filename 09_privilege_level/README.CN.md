# 教程 09 - 特权级别

## tl;dr

- 在早期引导代码中，我们从`Hypervisor`特权级别（AArch64中的`EL2`）过渡到`Kernel` （`EL1`）特权级别。

## 目录

- [介绍](#介绍)
- [本教程的范围](#本教程的范围)
- [在入口点检查EL2](#在入口点检查EL2)
- [过渡准备](#过渡准备)
- [从未发生的异常中返回](#从未发生的异常中返回)
- [测试](#测试)
- [相比之前的变化（diff）](#相比之前的变化（diff）)

## 介绍

应用级别的CPU具有所谓的`privilege levels`，它们具有不同的目的：

| Typically used for | AArch64 | RISC-V | x86 |
| ------------- | ------------- | ------------- | ------------- |
| Userspace applications | EL0 | U/VU | Ring 3 |
| OS Kernel | EL1 | S/VS | Ring 0 |
| Hypervisor | EL2 | HS | Ring -1 |
| Low-Level Firmware | EL3 | M | |

在AArch64中，`EL`代表`Exception Level`（异常级别）。如果您想获取有关其他体系结构的更多信息，请查看以下链接：
- [x86 privilege rings](https://en.wikipedia.org/wiki/Protection_ring).
- [RISC-V privilege modes](https://content.riscv.org/wp-content/uploads/2017/12/Tue0942-riscv-hypervisor-waterman.pdf).

在继续之前，我强烈建议您先浏览一下[Programmer’s Guide for ARMv8-A]`的第3章`。它提供了关于该主题的简明概述。

[Programmer’s Guide for ARMv8-A]: http://infocenter.arm.com/help/topic/com.arm.doc.den0024a/DEN0024A_v8_architecture_PG.pdf

## 本教程的范围

默认情况下，树莓派将始终在`EL2`中开始执行。由于我们正在编写一个传统的`Kernel`，我们需要过渡到更合适的`EL1`。

## 在入口点检查EL2

首先，我们需要确保我们实际上是在`EL2`中执行，然后才能调用相应的代码过渡到`EL1`。
因此，我们在`boot.s`的顶部添加了一个新的检查，如果CPU核心不在`EL2`中，则将其停止。

```
// Only proceed if the core executes in EL2. Park it otherwise.
mrs	x0, CurrentEL
cmp	x0, {CONST_CURRENTEL_EL2}
b.ne	.L_parking_loop
```

接下来，在`boot.rs`中继续准备从`EL2`到`EL1`的过渡，通过调用`prepare_el2_to_el1_transition()`函数。

```rust
#[no_mangle]
pub unsafe extern "C" fn _start_rust(phys_boot_core_stack_end_exclusive_addr: u64) -> ! {
    prepare_el2_to_el1_transition(phys_boot_core_stack_end_exclusive_addr);

    // Use `eret` to "return" to EL1. This results in execution of kernel_init() in EL1.
    asm::eret()
}
```

## 过渡准备

由于`EL2`比`EL1`更具特权，它可以控制各种处理器功能，并允许或禁止`EL1`代码使用它们。
其中一个例子是访问计时器和计数器寄存器。我们已经在[tutorial 07](../07_timestamps/)中使用了它们，所以当然我们希望保留它们。
因此，我们在[Counter-timer Hypervisor Control register]中设置相应的标志，并将虚拟偏移量设置为零，以获取真实的物理值。

[Counter-timer Hypervisor Control register]:  https://docs.rs/aarch64-cpu/9.0.0/src/aarch64_cpu/registers/cnthctl_el2.rs.html

```rust
// Enable timer counter registers for EL1.
CNTHCTL_EL2.write(CNTHCTL_EL2::EL1PCEN::SET + CNTHCTL_EL2::EL1PCTEN::SET);

// No offset for reading the counters.
CNTVOFF_EL2.set(0);
```

接下来，我们配置[Hypervisor Configuration Register]，使`EL1`在`AArch64`模式下运行，而不是在`AArch32`模式下运行，这也是可能的。

[Hypervisor Configuration Register]: https://docs.rs/aarch64-cpu/9.0.0/src/aarch64_cpu/registers/hcr_el2.rs.html

```rust
// Set EL1 execution state to AArch64.
HCR_EL2.write(HCR_EL2::RW::EL1IsAarch64);
```

## 从未发生的异常中返回

实际上，从较高的EL过渡到较低的EL只有一种方式，即通过执行[ERET]指令。

[ERET]: https://docs.rs/aarch64-cpu/9.0.0/src/aarch64_cpu/asm.rs.html#92-101

在这个指令中，它将会将[Saved Program Status Register - EL2]的内容复制到`Current Program Status Register - EL1`，并跳转到存储在[Exception Link Register - EL2]。

这基本上是在发生异常时所发生的相反过程。您将在即将发布的教程中了解更多相关内容。

[Saved Program Status Register - EL2]: https://docs.rs/aarch64-cpu/9.0.0/src/aarch64_cpu/registers/spsr_el2.rs.html
[Exception Link Register - EL2]: https://docs.rs/aarch64-cpu/9.0.0/src/aarch64_cpu/registers/elr_el2.rs.html

```rust
// Set up a simulated exception return.
//
// First, fake a saved program status where all interrupts were masked and SP_EL1 was used as a
// stack pointer.
SPSR_EL2.write(
    SPSR_EL2::D::Masked
        + SPSR_EL2::A::Masked
        + SPSR_EL2::I::Masked
        + SPSR_EL2::F::Masked
        + SPSR_EL2::M::EL1h,
);

// Second, let the link register point to kernel_init().
ELR_EL2.set(crate::kernel_init as *const () as u64);

// Set up SP_EL1 (stack pointer), which will be used by EL1 once we "return" to it. Since there
// are no plans to ever return to EL2, just re-use the same stack.
SP_EL1.set(phys_boot_core_stack_end_exclusive_addr);
```

正如您所看到的，我们将`ELR_EL2`的值设置为之前直接从入口点调用的`kernel_init()`函数的地址。最后，我们设置了`SP_EL1`的堆栈指针。

您可能已经注意到，堆栈的地址作为函数参数进行了传递。正如您可能记得的，在`boot.s`的`_start()`函数中，
我们已经为`EL2`设置了堆栈。由于没有计划返回到`EL2`，我们可以直接重用相同的堆栈作为`EL1`的堆栈，
因此使用函数参数将其地址传递。

最后，在`_start_rust()`函数中调用了`ERET`指令。

```rust
#[no_mangle]
pub unsafe extern "C" fn _start_rust(phys_boot_core_stack_end_exclusive_addr: u64) -> ! {
    prepare_el2_to_el1_transition(phys_boot_core_stack_end_exclusive_addr);

    // Use `eret` to "return" to EL1. This results in execution of kernel_init() in EL1.
    asm::eret()
}
```

## 测试

在`main.rs`中，我们打印`current privilege level`，并额外检查`SPSR_EL2`中的掩码位是否传递到了`EL1`：

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
[MP] ⏩ Pushing 14 KiB =========================================🦀 100% 0 KiB/s Time: 00:00:00
[ML] Loaded! Executing the payload now

[    0.162546] mingo version 0.9.0
[    0.162745] Booting on: Raspberry Pi 3
[    0.163201] Current privilege level: EL1
[    0.163677] Exception handling state:
[    0.164122]       Debug:  Masked
[    0.164511]       SError: Masked
[    0.164901]       IRQ:    Masked
[    0.165291]       FIQ:    Masked
[    0.165681] Architectural timer resolution: 52 ns
[    0.166255] Drivers loaded:
[    0.166592]       1. BCM PL011 UART
[    0.167014]       2. BCM GPIO
[    0.167371] Timer test, spinning for 1 second
[    1.167904] Echoing input now
```

## 相比之前的变化（diff）
请检查[英文版本](README.md#diff-to-previous)，这是最新的。
