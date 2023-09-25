# 教程10 - 虚拟内存第一部分：将所有内容进行身份映射！

## tl;dr

- 打开`MMU`。
- 使用简单的方案：静态的`64 KiB`转换表。
- 为了教学目的，我们将数据写入重新映射的`UART`，并对其他所有内容进行`identity map`。

## 目录

- [介绍](#introduction)
- [MMU和分页理论](#MMU和分页理论)
- [方法](#方法)
  * [通用内核代码：`memory/mmu.rs`](#通用内核代码：`memory/mmu.rs`)
  * [BSP：`bsp/raspberrypi/memory/mmu.rs`](#bsp-bspraspberrypimemorymmurs)
  * [AArch64：`_arch/aarch64/memory/*`](#aarch64-_archaarch64memory)
  * [`kernel.ld`](#kernelld)
- [地址转换示例](#地址转换示例)
  * [使用64 KiB页描述符进行地址转换](#使用64KiB页描述符进行地址转换)
- [零成本抽象](#零成本抽象)
- [测试](#测试)
- [相比之前的变化（diff）](#相比之前的变化（diff）)

## 介绍

虚拟内存是一个非常复杂但重要且强大的主题。在本教程中，我们从简单易懂的方式开始，
通过打开`MMU`，使用静态转换表和一次性进行`identity-map`
（除了为教育目的而重新映射的`UART`之外；在下一个教程中，这将被取消）。

## MMU和分页理论

在这一点上，我们不会重新发明轮子并详细描述现代应用级处理器中分页的工作原理。
互联网上有很多关于这个主题的优秀资源，我们鼓励您阅读其中一些以获得对该主题的高层理解。

继续阅读本`AArch64`特定的教程，我强烈建议您在此处停下来，首先阅读[ARM Cortex-A Series Programmer's Guide for ARMv8-A]的`第12章`，
以便在继续之前获得所有所需的`AArch64`特定知识。

已经阅读完`第12章`了吗？做得好 :+1:!

[ARM Cortex-A Series Programmer's Guide for ARMv8-A]: http://infocenter.arm.com/help/topic/com.arm.doc.den0024a/DEN0024A_v8_architecture_PG.pdf

## 方法

1. 通用的`kernel`部分：`src/memory/mmu.rs`及其子模块提供了与体系结构无关的描述符类型，
   用于组合一个高级数据结构，描述内核的虚拟内存布局：`memory::mmu::KernelVirtualLayout`。
2. `BSP`部分：`src/bsp/raspberrypi/memory/mmu.rs`包含一个`KernelVirtualLayout`的静态实例，并通过函数
   `bsp::memory::mmu::virt_mem_layout()`使其可访问。
3. `aarch64`部分：`src/_arch/aarch64/memory/mmu.rs`及其子模块包含实际的`MMU`驱动程序。它使用`64 KiB`粒度获取
   `BSP`的高级`KernelVirtualLayout`并进行映射。

### 通用内核代码：`memory/mmu.rs`

在这个文件中提供的描述符类型是构建块，用于描述不同内存区域的属性。
例如，`R/W`（读/写）、`no-execute`（不执行）、`cached/uncached`（缓存/非缓存）等等。

这些描述符与硬件`MMU`的实际描述符无关。不同的`BSP`可以使用这些类型来生成内核虚拟内存布局的高级描述。
真实硬件的实际`MMU`驱动程序将使用这些类型作为输入。

通过这种方式，我们在`BSP`和`_arch`代码之间实现了清晰的抽象，这样可以在不需要调整另一个的情况下进行交换。

### BSP: `bsp/raspberrypi/memory/mmu.rs`

这个文件包含了一个`KernelVirtualLayout`的实例，用于存储先前提到的描述符。
将其放在`BSP`中是正确的位置，因为它具有目标板的内存映射知识。

策略是只描述**不是**普通的、可缓存的DRAM的区域。然而，如果您希望，也可以定义这些区域。
这里是一个设备MMIO区域的示例：

```rust
TranslationDescriptor {
    name: "Device MMIO",
    virtual_range: mmio_range_inclusive,
    physical_range_translation: Translation::Identity,
    attribute_fields: AttributeFields {
        mem_attributes: MemAttributes::Device,
        acc_perms: AccessPermissions::ReadWrite,
        execute_never: true,
    },
},
```

`KernelVirtualLayout`本身实现了以下方法：

```rust
pub fn virt_addr_properties(
    &self,
    virt_addr: usize,
) -> Result<(usize, AttributeFields), &'static str>
```

它将被`_arch/aarch64`的`MMU`代码使用，用于请求虚拟地址和转换的属性，该转换提供物理输出地址
（返回元组中的`usize`）。该函数扫描包含查询地址的描述符，并返回第一个匹配的条目的相应结果。
如果找不到条目，则返回普通可缓存DRAM的默认属性和输入地址，从而告诉`MMU`代码请求的地址应该是`identity mapped`。

由于这种默认行为，不需要定义普通可缓存DRAM区域。

### AArch64: `_arch/aarch64/memory/*`

这些模块包含了`AArch64`的`MMU`驱动程序。粒度在这里被硬编码为（`64 KiB`页描述符）。

在`translation_table.rs`中，有一个实际的转换表结构的定义，它对`LVL2`表的数量进行了泛化。
后者取决于目标板的内存大小。自然地，`BSP`了解目标板的这些细节，并通过常量
`bsp::memory::mmu::KernelAddrSpace::SIZE`提供大小信息。

`translation_table.rs`使用这些信息来计算所需的`LVL2`表的数量。由于在`64 KiB`配置中，
一个`LVL2`表可以覆盖`512 MiB`，所以只需要将`KernelAddrSpace::SIZE`除以`512 MiB`
（有几个编译时检查确保`KernelAddrSpace::SIZE`是`512 MiB`的倍数）。

最终的表类型被导出为`KernelTranslationTable`。以下是来自`translation_table.rs`的相关代码：

```rust
/// A table descriptor for 64 KiB aperture.
///
/// The output points to the next table.
#[derive(Copy, Clone)]
#[repr(C)]
struct TableDescriptor {
    value: u64,
}

/// A page descriptor with 64 KiB aperture.
///
/// The output points to physical memory.
#[derive(Copy, Clone)]
#[repr(C)]
struct PageDescriptor {
    value: u64,
}

const NUM_LVL2_TABLES: usize = bsp::memory::mmu::KernelAddrSpace::SIZE >> Granule512MiB::SHIFT;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Big monolithic struct for storing the translation tables. Individual levels must be 64 KiB
/// aligned, hence the "reverse" order of appearance.
#[repr(C)]
#[repr(align(65536))]
pub struct FixedSizeTranslationTable<const NUM_TABLES: usize> {
    /// Page descriptors, covering 64 KiB windows per entry.
    lvl3: [[PageDescriptor; 8192]; NUM_TABLES],

    /// Table descriptors, covering 512 MiB windows.
    lvl2: [TableDescriptor; NUM_TABLES],
}

/// A translation table type for the kernel space.
pub type KernelTranslationTable = FixedSizeTranslationTable<NUM_LVL2_TABLES>;
```

在`mmu.rs`中，`KernelTranslationTable`用于创建内核表的最终实例：

```rust
//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

/// The kernel translation tables.
static mut KERNEL_TABLES: KernelTranslationTable = KernelTranslationTable::new();
```

它们在`MMU::init()`期间通过调用`KERNEL_TABLES.populate_tt_entries()`进行填充，
该函数利用`bsp::memory::mmu::virt_mem_layout().virt_addr_properties()`和一系列实用函数，将内核通用描述符转换为
`AArch64 MMU`硬件所需的实际`64 bit`整数条目，用于填充转换表数组。

一个值得注意的事情是，每个页描述符都有一个索引（`AttrIndex`），它索引到[MAIR_EL1]寄存器，
该寄存器保存了有关相应页面的缓存属性的信息。我们目前定义了普通可缓存内存和设备内存（不被缓存）。

[MAIR_EL1]: http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0500d/CIHDHJBB.html

```rust
impl MemoryManagementUnit {
    /// Setup function for the MAIR_EL1 register.
    fn set_up_mair(&self) {
        // Define the memory types being mapped.
        MAIR_EL1.write(
            // Attribute 1 - Cacheable normal DRAM.
            MAIR_EL1::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc +
        MAIR_EL1::Attr1_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc +

        // Attribute 0 - Device.
        MAIR_EL1::Attr0_Device::nonGathering_nonReordering_EarlyWriteAck,
        );
    }
```

然后，[Translation Table Base Register 0 - EL1]使用`lvl2`表的基地址进行设置，同时配置[Translation Control Register - EL1]：

```rust
// Set the "Translation Table Base Register".
TTBR0_EL1.set_baddr(KERNEL_TABLES.phys_base_address());

self.configure_translation_control();
```

最后，通过[System Control Register - EL1]打开`MMU`。最后一步还启用了数据和指令的缓存。

[Translation Table Base Register 0 - EL1]: https://docs.rs/aarch64-cpu/9.0.0/src/aarch64_cpu/registers/ttbr0_el1.rs.html
[Translation Control Register - EL1]: https://docs.rs/aarch64-cpu/9.0.0/src/aarch64_cpu/registers/tcr_el1.rs.html
[System Control Register - EL1]: https://docs.rs/aarch64-cpu/9.0.0/src/aarch64_cpu/registers/sctlr_el1.rs.html

### `kernel.ld`

我们需要将`code`段对齐到`64 KiB`，这样它就不会与下一个需要读/写属性而不是读/执行属性的部分重叠。

```ld.s
. = ALIGN(PAGE_SIZE);
__code_end_exclusive = .;
```

这会增加二进制文件的大小，但考虑到与传统的`4 KiB`粒度相比，它显著减少了静态分页条目的数量，这是一个小小的代价。

## 地址转换示例

出于教育目的，定义了一个布局，允许通过两个不同的虚拟地址访问`UART`
- 由于我们对整个`Device MMIO`区域进行了身份映射，所以在`MMU`打开后，可以通过断言其物理基地址
  （`0x3F20_1000`或`0xFA20_1000`，取决于使用的是哪个RPi版本）来访问它。
- 此外，它还映射到第一个`512 MiB`中的最后一个`64 KiB`槽位，使其可以通过基地址`0x1FFF_1000`访问。

以下块图可视化了第二个映射的底层转换。

### 使用64KiB页描述符进行地址转换

<img src="../doc/11_page_tables_64KiB.png" alt="Page Tables 64KiB" width="90%">

## 零成本抽象

初始化代码再次是展示Rust零成本抽象在嵌入式编程中巨大潜力的一个很好的例子[[1]][[2]]。

让我们再次看一下使用[aarch64-cpu]crate设置`MAIR_EL1`寄存器的代码片段：

[1]: https://blog.rust-lang.org/2015/05/11/traits.html
[2]: https://ruudvanasseldonk.com/2016/11/30/zero-cost-abstractions
[aarch64-cpu]: https://crates.io/crates/aarch64-cpu

```rust
/// Setup function for the MAIR_EL1 register.
fn set_up_mair(&self) {
    // Define the memory types being mapped.
    MAIR_EL1.write(
        // Attribute 1 - Cacheable normal DRAM.
        MAIR_EL1::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc +
    MAIR_EL1::Attr1_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc +

    // Attribute 0 - Device.
    MAIR_EL1::Attr0_Device::nonGathering_nonReordering_EarlyWriteAck,
    );
}
```

这段代码具有超强的表达能力，它利用`traits`，不同的`types`和`constants`来提供类型安全的寄存器操作。

最后，此代码根据数据表将寄存器的前四个字节设置为特定值。查看生成的代码，
我们可以看到，尽管有所有的类型安全和抽象，但它可以归结为两条汇编指令：

```text
   800a8:       529fe089        mov     w9, #0xff04                     // #65284
   800ac:       d518a209        msr     mair_el1, x9
```

## 测试

打开虚拟内存现在是我们在内核初始化过程中要做的第一件事：

```rust
unsafe fn kernel_init() -> ! {
    use memory::mmu::interface::MMU;

    if let Err(string) = memory::mmu::mmu().enable_mmu_and_caching() {
        panic!("MMU: {}", string);
    }
```

稍后在引导过程中，可以观察到有关映射的打印：

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

[    0.811167] mingo version 0.10.0
[    0.811374] Booting on: Raspberry Pi 3
[    0.811829] MMU online. Special regions:
[    0.812306]       0x00080000 - 0x0008ffff |  64 KiB | C   RO PX  | Kernel code and RO data
[    0.813324]       0x1fff0000 - 0x1fffffff |  64 KiB | Dev RW PXN | Remapped Device MMIO
[    0.814310]       0x3f000000 - 0x4000ffff |  17 MiB | Dev RW PXN | Device MMIO
[    0.815198] Current privilege level: EL1
[    0.815675] Exception handling state:
[    0.816119]       Debug:  Masked
[    0.816509]       SError: Masked
[    0.816899]       IRQ:    Masked
[    0.817289]       FIQ:    Masked
[    0.817679] Architectural timer resolution: 52 ns
[    0.818253] Drivers loaded:
[    0.818589]       1. BCM PL011 UART
[    0.819011]       2. BCM GPIO
[    0.819369] Timer test, spinning for 1 second
[     !!!    ] Writing through the remapped UART at 0x1FFF_1000
[    1.820409] Echoing input now
```

## 相比之前的变化（diff）
请检查[英文版本](README.md#diff-to-previous)，这是最新的。
