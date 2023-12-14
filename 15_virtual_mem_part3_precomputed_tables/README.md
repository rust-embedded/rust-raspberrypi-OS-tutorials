# Tutorial 15 - Virtual Memory Part 3: Precomputed Translation Tables

## tl;dr

- We are making the next baby-steps towards mapping the kernel to the most significant area of the
  virtual memory space.
- Instead of dynamically computing the kernel's translation tables during runtime while booting, we
  are precomputing them in advance just after kernel compilation, and patch them into the kernel's
  binary ahead of time.
- For now, we are still `identity-mapping` the kernel binary.
  - However, after this tutorial, we have all the infrastructure in place to easily map it
    elsewhere.

## Table of Contents

- [Introduction](#introduction)
- [When Load Address != Link Address, Funny Things Can Happen](#when-load-address--link-address-funny-things-can-happen)
  * [Interim Conclusion](#interim-conclusion)
  * [Doing the Same Thing - Expecting Different Results](#doing-the-same-thing---expecting-different-results)
- [Position-Independent Code (PIC)](#position-independent-code-pic)
  * [Using PIC during kernel startup](#using-pic-during-kernel-startup)
- [Precomputed Translation Tables](#precomputed-translation-tables)
- [Implementation](#implementation)
  * [Preparing the Kernel Tables](#preparing-the-kernel-tables)
  * [Turning on the MMU Before Switching to EL1](#turning-on-the-mmu-before-switching-to-el1)
  * [The Translation Table Tool](#the-translation-table-tool)
  * [Other changes](#other-changes)
- [Discussion](#discussion)
- [Test it](#test-it)
- [Diff to previous](#diff-to-previous)

## Introduction

This tutorial is another preparatory step for our overall goal of mapping the kernel to the most
significant area of the virtual memory space.

The reasoning of why we want to do this was given in the previous tutorial's introduction. But lets
for a quick moment think about what it actually means in practice: Currently, the kernel's binary is
loaded by the Raspberry's firmware at address `0x8_0000`.

In decimal, this address is at `512 KiB`, and therefore well within the _least significant part_ of
the address space. Let's have a look at the picture from the [ARM Cortex-A Series Programmer’s Guide
for ARMv8-A] again to understand in which virtual address space region the kernel would ideally be
mapped to:

<p align="center">
    <img src="../doc/15_kernel_user_address_space_partitioning.png" height="500" align="center">
</p>

As we can see, the architecture proposes somewhere between addresses `0xffff_0000_0000_0000` and
`0xffff_ffff_ffff_ffff`. Once we succeed in mapping the kernel there, the whole lower range between
`0x0` and `0xffff_ffff_ffff` would be free for future applications to use.

[ARM Cortex-A Series Programmer’s Guide for ARMv8-A]: https://developer.arm.com/documentation/den0024/latest/

Now, how can we get there?

## When Load Address != Link Address, Funny Things Can Happen

Imagine that, using the linker script, we link the kernel so that its `_start()` function is located
at address `0xffff_0000_0000_0000`. What hasn't changed is that the Raspberry's firmware will still
load the kernel binary at address `0x8_0000`, and the kernel will still start executing from there
with the `MMU` disabled.

So one of the very first things the kernel must achieve during its boot to function correctly, is to
somehow enable the `MMU` together with `translation tables` that account for the address offset
(`0xffff_0000_0000_0000 -> 0x8_0000`). In previous tutorials, we already generated translation
tables during the kernel's boot, so lets quickly remember how we did that:

In `src/bsp/__board_name__/memory/mmu.rs` we have a static (or "global" in non-Rust speak) instance
of `struct KernelTranslationTable`:

```rust
static KERNEL_TABLES: InitStateLock<KernelTranslationTable> =
    InitStateLock::new(KernelTranslationTable::new());
```

In other parts of the kernel code, this instance would be referenced one way or the other, and its
member functions would be called, for example, when mapping a range of pages. At the end of the day,
after multiple layers of indirection, what happens at the most basic level is that a `piece of code`
manipulates some `global data`. So part of the job of the code is to retrieve the data's constant
address before it can manipulate it.

Let's simplify the address-retrieval to the most basic code example possible. The example will be
presented as `C` code. Don't ask yet why `C` is chosen. It will get clear as the tutorial develops.

```c
#include <stdint.h>

uint64_t global_data_word = 0x11223344;

uint64_t* get_address_of_global(void) {
     return &global_data_word;
}
```

Let's compile and link this using the following linker script:

```ld.s
SECTIONS
{
    . =  0x80000;

    .text : {
        QUAD(0); /* Intentional fill word */
        QUAD(0); /* Intentional fill word */
        KEEP(*(.text*))
    }
    .got    : ALIGN(8)   { *(.got) }
    .data   : ALIGN(64K) {
        QUAD(0); /* Intentional fill word */
        *(.data*)
    }
}
```

Here are the compilation steps and the corresponding `objdump` for `AArch64`:

```console
$ clang --target=aarch64-none-elf -Iinclude -Wall -c start.c -o start.o
$ ld.lld start.o -T kernel.ld -o example.elf
```

```c-objdump
Disassembly of section .text:

0000000000080010 get_address_of_global:
   80010: 80 00 00 90                  	adrp	x0, #0x10000
   80014: 00 20 00 91                  	add	x0, x0, #0x8
   80018: c0 03 5f d6                  	ret

Disassembly of section .data:

0000000000090008 global_data_word:
   90008: 44 33 22 11
   9000c: 00 00 00 00
```

As you can see, the address of function `get_address_of_global()` is `0x8_0010` and
`global_data_word` got address `0x9_0008`. In the function body, the compiler emitted an [`ADRP`]
and `ADD` instruction pair, which means that the global's address is calculated as a `PC-relative
offset`. `PC` means program counter, aka the current position of where the CPU core is currently
executing from.

Without going in too much detail, what the instruction basically does is: It retrieves the `4 KiB`
page address that belongs to the program counter's (PC) current position (PC is at `0x8_0010`, so
the page address is `0x8_0000`), and adds `0x1_0000`. So after the `ADRP` instruction, register `x0`
holds the value `0x9_0000`. To this value, `8` is added in the next instruction, resulting in the
overall address of `0x9_0008`, which is exactly where `global_data_word` is located. This works,
because after linking a `static executable binary` like we do since `tutorial 01`, relative
positions of code and data are fixed, and not supposed to change during runtime.

[`ADRP`]: https://developer.arm.com/documentation/dui0802/b/A64-General-Instructions/ADRP

If the Raspberry's firmware now loads this binary at address `0x8_0000`, as always, we can be sure
that our function returns the correct address of our global data word.

Now lets link this to the most significant area of memory:

```ld.s
SECTIONS
{
    . =  0xffff000000000000; /* <--- Only line changed in the linker script! */

    .text : {

    /* omitted for brevity */
}
```

And compile again:

```c-objdump
Disassembly of section .text:

ffff000000000010 get_address_of_global:
ffff000000000010: 80 00 00 90          	adrp	x0, #0x10000
ffff000000000014: 00 20 00 91          	add	x0, x0, #0x8
ffff000000000018: c0 03 5f d6          	ret

Disassembly of section .data:

ffff000000010008 global_data_word:
ffff000000010008: 44 33 22 11
ffff00000001000c: 00 00 00 00
```

And let the Raspberry's firmware load the binary at address `0x8_0000` again (we couldn't load it to
`0xffff_0000_0000_0000` even if we wanted to. That address is `15 Exbibyte`. A Raspberry Pi with
that much RAM won't exist for some time to come 😉).

Let's try to answer the same question again: Would `get_address_of_global()` return the value for
`global_data_word` that we expect to see (`0xffff_0000_0001_0008` as shown in the objdump)? This
time, the answer is **no**. It would again return `0x9_0008`.

Why is that? Don't let yourself be distracted by the addresses the `objdump` above is showing. When
the Raspberry's firmware loads this binary at `0x8_0000`, then the Program Counter value when
`get_address_of_global()` executes is again `0x8_0010`. So **the PC-relative calculation** will not
result in the expected value, which would be the **absolute** (alternatively: **link-time**) address
of `global_data_word`.

### Interim Conclusion

What have we learned so far? We wrote a little piece of code in a high-level language that retrieves
an address, and we naively expected to retrieve an **absolute** address.

But compiler and linker conspired against us, and machine code was emitted that uses a PC-relative
addressing scheme, so our expectation is not matched when **load address != link address**. If you
compile for `AArch64`, you'll see relative addressing schemes a lot, because it is natural to the
architecture.

If you now say: Wait a second, how is this a problem? It actually helps! After all, since the code
is loaded at address `0x8_0000`, this relative addressing scheme will ensure that the processor
accesses the global data word at the correct address!

Yes, in this particular, constrained demo case, it worked out for us. But have a look at the
following.

### Doing the Same Thing - Expecting Different Results

Let's take a quick detour and see what happens if we compile **the exactly same code** for the
`x86_64` processor architecture. First when linked to `0x8_0000`:

```c-objdump
Disassembly of section .text:

0000000000080070 get_address_of_global:
   80070: 55                            push    rbp
   80071: 48 89 e5                      mov     rbp, rsp
   80074: 48 b8 08 00 09 00 00 00 00 00 movabs  rax, 0x90008
   8007e: 5d                            pop     rbp
   8007f: c3                            ret

Disassembly of section .data:

ffff000000010008 global_data_word:
ffff000000010008: 44 33 22 11
ffff00000001000c: 00 00 00 00
```

And now linked to `0xffff_0000_0000_0000`:

```c-objdump
Disassembly of section .text:

ffff000000000070 get_address_of_global:
ffff000000000070: 55                            push    rbp
ffff000000000071: 48 89 e5                      mov     rbp, rsp
ffff000000000074: 48 b8 08 00 01 00 00 00 ff ff movabs  rax, 0xffff000000010008
ffff00000000007e: 5d                            pop     rbp
ffff00000000007f: c3                            ret

Disassembly of section .data:

ffff000000010008 global_data_word:
ffff000000010008: 44 33 22 11
ffff00000001000c: 00 00 00 00
```

Both times, the `movabs` instruction gets emitted. It means that the address is put into the target
register using hardcoded `immediate values`. PC-relative address calculation is not used here.
Hence, this code would return the `absolute` address in both cases. Which means in the second case,
even when the binary would be loaded at `0x8_0000`, the return value would be
`0xffff_0000_0001_0008`.

**In summary, we get a different result for the same piece of `C` code, depending on the target
processor architecture**. What do we learn from this little detour?

First, you cannot naively compile and run `Rust` or `C` statically linked binaries when there will
be a **load address != link address** situation. You'll run into undefined behavior very fast. It is
kinda expected and obvious, but hopefully it helped to see it fail in action.

Furthermore, it is important to understand that there are of course ways to load a symbol's absolute
address into `AArch64` registers using `immediate values` as well. Likewise, you can also do
PC-relative addressing in `x86`. We just looked at a tiny example. Maybe the next line of code would
be compiled into the opposite behavior on the two architectures, so that the `x86` code would do a
PC-relative calculation while the `AArch64` code goes for absolute.

At the end of the day, what is needed to solve our task at hand (bringup of virtual memory, while
being linked to one address and executing from another), is tight control over the machine
instructions that get emitted for **those pieces of code** that generate the `translation tables`
and enable the `MMU`.

What we need is called [position-independent code].

[position-independent code]: https://en.wikipedia.org/wiki/Position-independent_code

> Much low-level stuff in this tutorial, isn't it? This was a lot to digest already, but we're far
> from finished. So take a minute or two and clear your mind before we continue. 🧘

## Position-Independent Code (PIC)

As describend by Wikipedia, position-independent code

> is a body of machine code that, being placed somewhere in the primary memory, **executes
> properly** regardless of its absolute address.

Your safest bet is to write the pieces that need to be position-independent in `assembly`, because
this gives you full control over when relative or absolute addresses are being generated or used.
You will see this approach often in big projects like the Linux kernel, for example. The downside of
that approach is that the programmer needs good domain knowledge.

If you feel more adventurous and don't want to go completely without high-level code, you can try to
make use of suitable compiler flags such as `-fpic`, and only use `assembly` where absolutely
needed. Here is the [`-fpic` description for GCC]:

> -fpic
>
> Generate position-independent code (PIC) suitable for use in a shared library, if supported for
> the target machine. Such code accesses all constant addresses through a global offset table (GOT).
> The dynamic loader resolves the GOT entries when the program starts (the dynamic loader is not
> part of GCC; it is part of the operating system).

[`-fpic` description for GCC]: https://gcc.gnu.org/onlinedocs/gcc/Code-Gen-Options.html#Code-Gen-Options

However, it is very important to understand that this flag **is not** a ready-made solution for our
particular problem (and wasn't invented for that case either). There is a hint in the quoted text
above that gives it away: "_The dynamic loader resolves the GOT entries when the program starts (the
dynamic loader is not part of GCC; it is part of the operating system)_".

Well, we are a booting kernel, and not some (userspace) program running on top of an operating
system. Hence, there is no dynamic loader available. However, it is still possible to benefit from
`-fpic` even in our case. Lets have a look at what happens if we compile the earlier piece of `C`
code for `AArch64` using `-fpic`, still linking the output to the most signifcant part of the memory
space:

```console
$ clang --target=aarch64-none-elf -Iinclude -Wall -fpic -c start.c -o start.o
$ ld.lld start.o -T kernel.ld -o example.elf
```

```c-objdump
Disassembly of section .text:

ffff000000000010 get_address_of_global:
ffff000000000010: 00 00 00 90          	adrp	x0, #0x0
ffff000000000014: 00 28 40 f9          	ldr	x0, [x0, #0x50]
ffff000000000018: c0 03 5f d6          	ret

Disassembly of section .got:

ffff000000000050 .got:
ffff000000000050: 08 00 01 00
ffff000000000054: 00 00 ff ff

Disassembly of section .data:

ffff000000010008 global_data_word:
ffff000000010008: 44 33 22 11
ffff00000001000c: 00 00 00 00
```

What changed compared to earlier is that `get_address_of_global()` now indirects through the `Global
Offset Table`, as has been promised by the compiler's documentation. Specifically,
`get_address_of_global()` addresses the `GOT` using PC-relative addressing (distance from code to
`GOT` must always be fixed), and loads the first 64 bit word from the start of the `GOT`, which
happens to be `0xffff_0000_0001_0008`.

Okay okay okay... So when we use `-fpic`, we get the **absolute** address of `global_data_word` even
on `AArch64` now. How does this help when the code executes from `0x8_0000`?

Well, this is the part where the `dynamic loader` quoted above would come into picture if this was a
userspace program: "_The dynamic loader resolves the GOT entries when the program starts_". The
`-fpic` flag is normally used to compile _shared libraries_. Suppose we have a program that uses one
or more shared library. For various reasons, it happens that the shared library is loaded at a
different address than where the userspace program would initially expect it. In our example,
`global_data_word` could be supplied by such a shared library, and the userspace program is only
referencing it. The dynamic loader would know where the shared library was loaded into memory, and
therefore know the real address of `global_data_word`. So before the userspace program starts, the
loader would overwrite the `GOT` entry with the correct location. Et voilà, the compiled high-level
code would execute properly.

### Using PIC during kernel startup

If you think about it, our problem is a special case of what we just learned. We have a single
statically linked binary, where everything is dislocated by a fixed offset. In our case, it is
`0xffff_0000_0000_0000 - 0x8_0000 = 0x0fff_efff_ffff8_0000`. If we write some PIC-`assembly` code
which loops over the `GOT` and subtracts `0x0fff_efff_ffff8_0000` from every entry as the very first
thing when our kernel boots, any high-level code compiled with `-fpic` would work correctly
afterwards.

Moreover, this approach would be portable! Here's the output of our code compiled with `-fpic` for
`x86_64`:

```c-objdump
Disassembly of section .text:

ffff000000000070 get_address_of_global:
ffff000000000070: 55                    push    rbp
ffff000000000071: 48 89 e5              mov     rbp, rsp
ffff000000000074: 48 8b 05 2d 00 00 00  mov     rax, qword ptr [rip + 0x2d]
ffff00000000007b: 5d                    pop     rbp
ffff00000000007c: c3                    ret
ffff00000000007d: 0f 1f 00              nop     dword ptr [rax]

Disassembly of section .got:

ffff0000000000a8 .got:
ffff0000000000a8: 08 00 01 00
ffff0000000000ac: 00 00 ff ff

Disassembly of section .data:

ffff000000010008 global_data_word:
ffff000000010008: 44 33 22 11
ffff00000001000c: 00 00 00 00
```

As you can see, the `x86_64` code indirects through the `GOT` now same as the `AArch64` code.

Of course, indirecting through the `GOT` would be detrimental to performance, so you would restrict
`-fpic` compilation only to the code that is needed to enable the `MMU`. Everything else can be
compiled `non-relocatable` as always, because the translation tables naturally resolve the **load
address != link address** situation once they are live.

With `C/C++` compilers, this can be done rather easily. The compilers support compilation of
PIC-code on a per-[translation-unit] basis. Think of it as telling the compiler to compile this `.c`
file as `PIC`, but this other `.c` file not.

[translation-unit]: https://en.wikipedia.org/wiki/Translation_unit_(programming)

With `Rust`, unfortunately, the [relocation model] can only be set on a per-`crate` basis at the
moment (IINM), so that makes it difficult for us to put this approach to use.

[relocation model]: https://doc.rust-lang.org/rustc/codegen-options/index.html#relocation-model

## Precomputed Translation Tables

As we have just seen, going the `-fpic` way isn't a feasible solution at the time of writing this
text. On the other hand, writing the code to set up the initial page tables in `assembly` isn't that
attractive either, because writing larger pieces of assembly is an error-prone and delicate task.

Fortunately, there is a third way. We are writing an embedded kernel, and therefore the execution
environment is way more static and deterministic as compared to a general-purpose kernel that can be
deployed on a wide variety of targets. Specifically, for the Raspberrypi, we exactly know the **load
address** of the kernel in advance, and we know about the capabilities of the `MMU`. So there is
nothing stopping us from precomputing the kernel's translation tables ahead of time.

A disadvantage of this approach is an increased binary size, but this is not a deal breaker in our
case.

## Implementation

As stated in the initial `tl;dr`, we're not yet mapping the kernel to the most significant area of
virtual memory. This tutorial will keep the binary `identity-mapped`, and focuses only on the
infrastructure changes which enable the kernel to use `precomputed translation tables`. The actual
switch to high memory will happen in the next tutorial.

The changes needed are as follows:

1. Make preparations so that precomputed tables are supported by the kernel's memory subsystem code.
2. Change the boot code of the kernel so that the `MMU` is enabled with the precomputed tables as
   soon as possible.
3. Write a `translation table tool` that precomputes the translation tables from the generated
   `kernel.elf` file, and patches the tables back into the same.

### Preparing the Kernel Tables

The tables must be linked into the `.data` section now so that they become part of the final binary.
This is ensured using an attribute on the table's instance definition in
`bsp/__board_name__/memory/mmu.rs`:

```rust
#[link_section = ".data"]
#[no_mangle]
static KERNEL_TABLES: InitStateLock<KernelTranslationTable> =
    InitStateLock::new(KernelTranslationTable::new_for_precompute());
```

The `new_for_precompute()` is a new constructor in the the respective `_arch` code that ensures some
struct members that are not the translation table entries themselves are initialized properly for
the precompute use-case. The additional `#[no_mangle]` is added because we will need to parse the
symbol from the `translation table tool`, and this is easier with unmangled names.

In the `BSP` code, there is also a new file called `kernel_virt_addr_space_size.ld`, which contains
the kernel's virtual address space size. This file gets included in both, the `kernel.ld` linker
script and `mmu.rs`. We need this value both as a symbol in the kernel's ELF (for the `translation
table tool` to parse it later) and as a constant in the `Rust` code. This inclusion approach is just
a convenience hack that turned out working well.

One critical parameter that the kernel's boot code needs in order to enable the precomputed tables
is the `translation table base address` which must be programmed into the MMU's `TTBR` register. To
make it accessible easily, it is added to the `.text._start_arguments` section. The definition is
just below the definition of the kernel table instance in the `BSP` code:

```rust
/// This value is needed during early boot for MMU setup.
///
/// This will be patched to the correct value by the "translation table tool" after linking. This
/// given value here is just a dummy.
#[link_section = ".text._start_arguments"]
#[no_mangle]
static PHYS_KERNEL_TABLES_BASE_ADDR: u64 = 0xCCCCAAAAFFFFEEEE;
```

### Turning on the MMU Before Switching to EL1

Since the Raspberry Pi starts execution in the `EL2` privilege level, one of the first things we do
during boot since `tutorial 09` is to context-switch to the appropriate `EL1`. The `EL2` boot code
is a great place to set up virtual memory for `EL1`. It will allow execution in `EL1` to start with
virtual memory enabled since the very first instruction. The tweaks to `boot.s` are minimal:

```asm
// Load the base address of the kernel's translation tables.
ldr	x0, PHYS_KERNEL_TABLES_BASE_ADDR // provided by bsp/__board_name__/memory/mmu.rs

// Set the stack pointer. This ensures that any code in EL2 that needs the stack will work.
ADR_REL	x1, __boot_core_stack_end_exclusive
mov	sp, x1

// Jump to Rust code. x0 and x1 hold the function arguments provided to _start_rust().
b	_start_rust
```

In addition to the stack's address, we are now reading _the value_ of
`PHYS_KERNEL_TABLES_BASE_ADDR`. The `ldr` instruction addresses the value-to-be-read using a
PC-relative offset, so this is a `position-independent` operation and will therefore be future
proof. The retrieved value is supplied as an argument to function `_start_rust()`, which is defined
in `_arch/__arch_name__/cpu/boot.rs`:

```rust
#[no_mangle]
pub unsafe extern "C" fn _start_rust(
    phys_kernel_tables_base_addr: u64,
    phys_boot_core_stack_end_exclusive_addr: u64,
) -> ! {
    prepare_el2_to_el1_transition(phys_boot_core_stack_end_exclusive_addr);

    // Turn on the MMU for EL1.
    let addr = Address::new(phys_kernel_tables_base_addr as usize);
    memory::mmu::enable_mmu_and_caching(addr).unwrap();

    // Use `eret` to "return" to EL1. This results in execution of kernel_init() in EL1.
    asm::eret()
}
```

You can also see that we now turn on the `MMU` just before returning to `EL1`. That's basically it
already, the only missing piece that's left is the offline computation of the translation tables.

### The Translation Table Tool

The tool for translation table computation is located in the folder
`$ROOT/tools/translation_table_tool`. For ease of use, it is written in `Ruby` 💎. The code is
organized into `BSP` and `arch` parts just like the kernel's `Rust` code, and also has a class for
processing the kernel `ELF` file:

```console
$ tree tools/translation_table_tool

tools/translation_table_tool
├── arch.rb
├── bsp.rb
├── generic.rb
├── kernel_elf.rb
└── main.rb

0 directories, 5 files
```

Especially the `arch` part, which deals with compiling the translation table entries, will contain
some overlap with the `Rust` code present in `_arch/aarch64/memory/mmu/translation_table.rs`. It
might have been possible to write this tool in Rust as well, and borrow/share these pieces of code
with the kernel. But in the end, I found it not worth the effort for the few lines of code.

In the `Makefile`, the tool is invoked after compiling and linking the kernel, and before the
`stripped binary` is generated. It's command line arguments are the target `BSP` type and the path
to the kernel's `ELF` file:

```Makefile
TT_TOOL_PATH = tools/translation_table_tool

KERNEL_ELF_RAW      = target/$(TARGET)/release/kernel
# [...]

KERNEL_ELF_TTABLES      = target/$(TARGET)/release/kernel+ttables
# [...]

EXEC_TT_TOOL       = ruby $(TT_TOOL_PATH)/main.rb
# [...]

##------------------------------------------------------------------------------
## Compile the kernel ELF
##------------------------------------------------------------------------------
$(KERNEL_ELF_RAW): $(KERNEL_ELF_RAW_DEPS)
	$(call color_header, "Compiling kernel ELF - $(BSP)")
	@RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(RUSTC_CMD)

##------------------------------------------------------------------------------
## Precompute the kernel translation tables and patch them into the kernel ELF
##------------------------------------------------------------------------------
$(KERNEL_ELF_TTABLES): $(KERNEL_ELF_TTABLES_DEPS)
	$(call color_header, "Precomputing kernel translation tables and patching kernel ELF")
	@cp $(KERNEL_ELF_RAW) $(KERNEL_ELF_TTABLES)
	@$(DOCKER_TOOLS) $(EXEC_TT_TOOL) $(TARGET) $(BSP) $(KERNEL_ELF_TTABLES)

##------------------------------------------------------------------------------
## Generate the stripped kernel binary
##------------------------------------------------------------------------------
$(KERNEL_BIN): $(KERNEL_ELF_TTABLES)
	$(call color_header, "Generating stripped binary")
	@$(OBJCOPY_CMD) $(KERNEL_ELF_TTABLES) $(KERNEL_BIN)
```

In `main.rb`, the `KERNEL_ELF` instance for handling the `ELF` file is created first, followed by
`BSP` and `arch` parts:

```ruby
KERNEL_ELF = KernelELF.new(kernel_elf_path)

BSP = case BSP_TYPE
      when :rpi3, :rpi4
          RaspberryPi.new
      else
          raise
      end

TRANSLATION_TABLES = case KERNEL_ELF.machine
                     when :AArch64
                         Arch::ARMv8::TranslationTable.new
                     else
                         raise
                     end

kernel_map_binary
```

Finally, the function `kernel_map_binary` is called, which kicks of a sequence of interactions
between the `KERNEL_ELF`, `BSP` and `TRANSLATION_TABLES` instances:

```ruby
def kernel_map_binary
    mapping_descriptors = KERNEL_ELF.generate_mapping_descriptors

    # omitted

    mapping_descriptors.each do |i|
        print 'Generating'.rjust(12).green.bold
        print ' '
        puts i

        TRANSLATION_TABLES.map_at(i.virt_region, i.phys_region, i.attributes)
    end

    # omitted
end
```

The `generate_mapping_descriptors` method internally uses the
[rbelftools](https://github.com/david942j/rbelftools) gem to parse the kernel's `ELF`. It extracts
information about the various segments that comprise the kernel, as well as segment characteristics
like their `virtual` and `physical` addresses (aka the mapping; still identity-mapped in this case)
and the memory attributes.

Part of this information can be cross-checked using the `make readelf` target if you're curious:

```console
$ make readelf

Program Headers:
  Type           Offset             VirtAddr           PhysAddr
                 FileSiz            MemSiz              Flags  Align
  LOAD           0x0000000000010000 0x0000000000000000 0x0000000000000000
                 0x0000000000000000 0x0000000000080000  RW     0x10000
  LOAD           0x0000000000010000 0x0000000000080000 0x0000000000080000
                 0x000000000000cae8 0x000000000000cae8  R E    0x10000
  LOAD           0x0000000000020000 0x0000000000090000 0x0000000000090000
                 0x0000000000030dc0 0x0000000000030de0  RW     0x10000

 Section to Segment mapping:
  Segment Sections...
   00     .boot_core_stack
   01     .text .rodata
   02     .data .bss

```

The output of `generate_mapping_descriptors` is then fed into the `map_at()` method of the
`TRANSLATION_TABLE` instance. For it to work properly, `TRANSLATION_TABLE` needs knowledge about
**location** and **layout** of the kernel's table structure. Location will be queried from the `BSP`
code, which itself retrieves it by querying `KERNEL_ELF` for the `BSP`-specific `KERNEL_TABLES`
symbol. The layout, on the other hand, is hardcoded and as such must be kept in sync with the
structure definition in `translation_tables.rs`.

Finally, after the mappings have been created, it is time to _patch_ them back into the kernel ELF
file. This is initiated from `main.rb` again:

```ruby
kernel_patch_tables(kernel_elf_path)
kernel_patch_base_addr(kernel_elf_path)
```

The tool prints some information on the fly. Here's the console output of a successful run:

```console
$ make

Compiling kernel - rpi3
    Finished release [optimized] target(s) in 0.00s

Precomputing kernel translation tables and patching kernel ELF
             ------------------------------------------------------------------------------------
                 Sections          Virt Start Addr         Phys Start Addr       Size      Attr
             ------------------------------------------------------------------------------------
  Generating .boot_core_stack | 0x0000_0000_0000_0000 | 0x0000_0000_0000_0000 | 512 KiB | C RW XN
  Generating .text .rodata    | 0x0000_0000_0008_0000 | 0x0000_0000_0008_0000 |  64 KiB | C RO X
  Generating .data .bss       | 0x0000_0000_0009_0000 | 0x0000_0000_0009_0000 | 256 KiB | C RW XN
             ------------------------------------------------------------------------------------
    Patching Kernel table struct at ELF file offset 0x2_0000
    Patching Kernel tables physical base address start argument to value 0xb_0000 at ELF file offset 0x1_0068
    Finished in 0.16s

```

Please note how **only** the kernel binary is precomputed! Thanks to the changes made in the last
tutorial, anything else, like `MMIO-remapping`, can and will happen lazily during runtime.

### Other changes

Two more things that changed in this tutorial, but won't be explained in detail:

- Since virtual memory in `EL1` is now active from the start, any attempt to convert from a kernel
  `Address<Virtual>` to `Address<Physical>` is now done using the function
  `mmu::try_kernel_virt_addr_to_phys_addr()`, which internally uses a new function that has been
  added to the `TranslationTable` interface. If there is no valid virtual-to-physical mapping
  present in the tables, an `Err()` is returned.
- The precomputed translation table mappings won't automatically have entries in the kernel's
  `mapping info record`, which is used to print mapping info during boot. Mapping record entries are
  not computed offline in order to reduce complexity. For this reason, the `BSP` code, which in
  earlier tutorials would have called `kernel_map_at()` (which implicitly would have generated
  mapping record entries), now only calls `kernel_add_mapping_record()`, since the mappings are
  already in place.

## Discussion

It is understood that there is room for optimizations in the presented approach. For example, the
generated kernel binary contains the _complete_ array of translation tables for the whole kernel
virtual address space. However, most of the entries are empty initially, because the kernel binary
only occupies a small area in the tables. It would make sense to add some smarts so that only the
non-zero entries are packed into binary.

On the other hand, this would add complexity to the code. The increased size doesn't hurt too much
yet, so the reduced complexity in the code is a tradeoff made willingly to keep everything concise
and focused on the high-level concepts.

## Test it

```console
$ make chainboot
[...]

Precomputing kernel translation tables and patching kernel ELF
             ------------------------------------------------------------------------------------
                 Sections          Virt Start Addr         Phys Start Addr       Size      Attr
             ------------------------------------------------------------------------------------
  Generating .boot_core_stack | 0x0000_0000_0000_0000 | 0x0000_0000_0000_0000 | 512 KiB | C RW XN
  Generating .text .rodata    | 0x0000_0000_0008_0000 | 0x0000_0000_0008_0000 |  64 KiB | C RO X
  Generating .data .bss       | 0x0000_0000_0009_0000 | 0x0000_0000_0009_0000 | 256 KiB | C RW XN
             ------------------------------------------------------------------------------------
    Patching Kernel table struct at ELF file offset 0x2_0000
    Patching Kernel tables physical base address start argument to value 0xb_0000 at ELF file offset 0x1_0068
    Finished in 0.15s

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
[MP] ⏩ Pushing 257 KiB ======================================🦀 100% 128 KiB/s Time: 00:00:02
[ML] Loaded! Executing the payload now

[    2.866917] mingo version 0.15.0
[    2.867125] Booting on: Raspberry Pi 3
[    2.867580] MMU online:
[    2.867872]       -------------------------------------------------------------------------------------------------------------------------------------------
[    2.869616]                         Virtual                                   Physical               Size       Attr                    Entity
[    2.871360]       -------------------------------------------------------------------------------------------------------------------------------------------
[    2.873105]       0x0000_0000_0000_0000..0x0000_0000_0007_ffff --> 0x00_0000_0000..0x00_0007_ffff | 512 KiB | C   RW XN | Kernel boot-core stack
[    2.874709]       0x0000_0000_0008_0000..0x0000_0000_0008_ffff --> 0x00_0008_0000..0x00_0008_ffff |  64 KiB | C   RO X  | Kernel code and RO data
[    2.876322]       0x0000_0000_0009_0000..0x0000_0000_000c_ffff --> 0x00_0009_0000..0x00_000c_ffff | 256 KiB | C   RW XN | Kernel data and bss
[    2.877893]       0x0000_0000_000d_0000..0x0000_0000_000d_ffff --> 0x00_3f20_0000..0x00_3f20_ffff |  64 KiB | Dev RW XN | BCM PL011 UART
[    2.879410]                                                                                                             | BCM GPIO
[    2.880861]       0x0000_0000_000e_0000..0x0000_0000_000e_ffff --> 0x00_3f00_0000..0x00_3f00_ffff |  64 KiB | Dev RW XN | BCM Interrupt Controller
[    2.882487]       -------------------------------------------------------------------------------------------------------------------------------------------
```

## Diff to previous
```diff

diff -uNr 14_virtual_mem_part2_mmio_remap/kernel/Cargo.toml 15_virtual_mem_part3_precomputed_tables/kernel/Cargo.toml
--- 14_virtual_mem_part2_mmio_remap/kernel/Cargo.toml
+++ 15_virtual_mem_part3_precomputed_tables/kernel/Cargo.toml
@@ -1,6 +1,6 @@
 [package]
 name = "mingo"
-version = "0.14.0"
+version = "0.15.0"
 authors = ["Andre Richter <andre.o.richter@gmail.com>"]
 edition = "2021"


diff -uNr 14_virtual_mem_part2_mmio_remap/kernel/src/_arch/aarch64/cpu/boot.rs 15_virtual_mem_part3_precomputed_tables/kernel/src/_arch/aarch64/cpu/boot.rs
--- 14_virtual_mem_part2_mmio_remap/kernel/src/_arch/aarch64/cpu/boot.rs
+++ 15_virtual_mem_part3_precomputed_tables/kernel/src/_arch/aarch64/cpu/boot.rs
@@ -11,6 +11,7 @@
 //!
 //! crate::cpu::boot::arch_boot

+use crate::{memory, memory::Address};
 use aarch64_cpu::{asm, registers::*};
 use core::arch::global_asm;
 use tock_registers::interfaces::Writeable;
@@ -75,9 +76,16 @@
 ///
 /// - Exception return from EL2 must must continue execution in EL1 with `kernel_init()`.
 #[no_mangle]
-pub unsafe extern "C" fn _start_rust(phys_boot_core_stack_end_exclusive_addr: u64) -> ! {
+pub unsafe extern "C" fn _start_rust(
+    phys_kernel_tables_base_addr: u64,
+    phys_boot_core_stack_end_exclusive_addr: u64,
+) -> ! {
     prepare_el2_to_el1_transition(phys_boot_core_stack_end_exclusive_addr);

+    // Turn on the MMU for EL1.
+    let addr = Address::new(phys_kernel_tables_base_addr as usize);
+    memory::mmu::enable_mmu_and_caching(addr).unwrap();
+
     // Use `eret` to "return" to EL1. This results in execution of kernel_init() in EL1.
     asm::eret()
 }

diff -uNr 14_virtual_mem_part2_mmio_remap/kernel/src/_arch/aarch64/cpu/boot.s 15_virtual_mem_part3_precomputed_tables/kernel/src/_arch/aarch64/cpu/boot.s
--- 14_virtual_mem_part2_mmio_remap/kernel/src/_arch/aarch64/cpu/boot.s
+++ 15_virtual_mem_part3_precomputed_tables/kernel/src/_arch/aarch64/cpu/boot.s
@@ -53,19 +53,22 @@

 	// Prepare the jump to Rust code.
 .L_prepare_rust:
+	// Load the base address of the kernel's translation tables.
+	ldr	x0, PHYS_KERNEL_TABLES_BASE_ADDR // provided by bsp/__board_name__/memory/mmu.rs
+
 	// Set the stack pointer. This ensures that any code in EL2 that needs the stack will work.
-	ADR_REL	x0, __boot_core_stack_end_exclusive
-	mov	sp, x0
+	ADR_REL	x1, __boot_core_stack_end_exclusive
+	mov	sp, x1

 	// Read the CPU's timer counter frequency and store it in ARCH_TIMER_COUNTER_FREQUENCY.
 	// Abort if the frequency read back as 0.
-	ADR_REL	x1, ARCH_TIMER_COUNTER_FREQUENCY // provided by aarch64/time.rs
-	mrs	x2, CNTFRQ_EL0
-	cmp	x2, xzr
+	ADR_REL	x2, ARCH_TIMER_COUNTER_FREQUENCY // provided by aarch64/time.rs
+	mrs	x3, CNTFRQ_EL0
+	cmp	x3, xzr
 	b.eq	.L_parking_loop
-	str	w2, [x1]
+	str	w3, [x2]

-	// Jump to Rust code. x0 holds the function argument provided to _start_rust().
+	// Jump to Rust code. x0 and x1 hold the function arguments provided to _start_rust().
 	b	_start_rust

 	// Infinitely wait for events (aka "park the core").

diff -uNr 14_virtual_mem_part2_mmio_remap/kernel/src/_arch/aarch64/memory/mmu/translation_table.rs 15_virtual_mem_part3_precomputed_tables/kernel/src/_arch/aarch64/memory/mmu/translation_table.rs
--- 14_virtual_mem_part2_mmio_remap/kernel/src/_arch/aarch64/memory/mmu/translation_table.rs
+++ 15_virtual_mem_part3_precomputed_tables/kernel/src/_arch/aarch64/memory/mmu/translation_table.rs
@@ -125,7 +125,7 @@
 }

 trait StartAddr {
-    fn phys_start_addr(&self) -> Address<Physical>;
+    fn virt_start_addr(&self) -> Address<Virtual>;
 }

 //--------------------------------------------------------------------------------------------------
@@ -151,9 +151,8 @@
 // Private Code
 //--------------------------------------------------------------------------------------------------

-// The binary is still identity mapped, so we don't need to convert here.
 impl<T, const N: usize> StartAddr for [T; N] {
-    fn phys_start_addr(&self) -> Address<Physical> {
+    fn virt_start_addr(&self) -> Address<Virtual> {
         Address::new(self as *const _ as usize)
     }
 }
@@ -218,6 +217,35 @@
     }
 }

+/// Convert the HW-specific attributes of the MMU to kernel's generic memory attributes.
+impl convert::TryFrom<InMemoryRegister<u64, STAGE1_PAGE_DESCRIPTOR::Register>> for AttributeFields {
+    type Error = &'static str;
+
+    fn try_from(
+        desc: InMemoryRegister<u64, STAGE1_PAGE_DESCRIPTOR::Register>,
+    ) -> Result<AttributeFields, Self::Error> {
+        let mem_attributes = match desc.read(STAGE1_PAGE_DESCRIPTOR::AttrIndx) {
+            memory::mmu::arch_mmu::mair::NORMAL => MemAttributes::CacheableDRAM,
+            memory::mmu::arch_mmu::mair::DEVICE => MemAttributes::Device,
+            _ => return Err("Unexpected memory attribute"),
+        };
+
+        let acc_perms = match desc.read_as_enum(STAGE1_PAGE_DESCRIPTOR::AP) {
+            Some(STAGE1_PAGE_DESCRIPTOR::AP::Value::RO_EL1) => AccessPermissions::ReadOnly,
+            Some(STAGE1_PAGE_DESCRIPTOR::AP::Value::RW_EL1) => AccessPermissions::ReadWrite,
+            _ => return Err("Unexpected access permission"),
+        };
+
+        let execute_never = desc.read(STAGE1_PAGE_DESCRIPTOR::PXN) > 0;
+
+        Ok(AttributeFields {
+            mem_attributes,
+            acc_perms,
+            execute_never,
+        })
+    }
+}
+
 impl PageDescriptor {
     /// Create an instance.
     ///
@@ -250,6 +278,19 @@
         InMemoryRegister::<u64, STAGE1_PAGE_DESCRIPTOR::Register>::new(self.value)
             .is_set(STAGE1_PAGE_DESCRIPTOR::VALID)
     }
+
+    /// Returns the output page.
+    fn output_page_addr(&self) -> PageAddress<Physical> {
+        let shifted = InMemoryRegister::<u64, STAGE1_PAGE_DESCRIPTOR::Register>::new(self.value)
+            .read(STAGE1_PAGE_DESCRIPTOR::OUTPUT_ADDR_64KiB) as usize;
+
+        PageAddress::from(shifted << Granule64KiB::SHIFT)
+    }
+
+    /// Returns the attributes.
+    fn try_attributes(&self) -> Result<AttributeFields, &'static str> {
+        InMemoryRegister::<u64, STAGE1_PAGE_DESCRIPTOR::Register>::new(self.value).try_into()
+    }
 }

 //--------------------------------------------------------------------------------------------------
@@ -267,7 +308,7 @@
 impl<const NUM_TABLES: usize> FixedSizeTranslationTable<NUM_TABLES> {
     /// Create an instance.
     #[allow(clippy::assertions_on_constants)]
-    pub const fn new() -> Self {
+    const fn _new(for_precompute: bool) -> Self {
         assert!(bsp::memory::mmu::KernelGranule::SIZE == Granule64KiB::SIZE);

         // Can't have a zero-sized address space.
@@ -276,10 +317,19 @@
         Self {
             lvl3: [[PageDescriptor::new_zeroed(); 8192]; NUM_TABLES],
             lvl2: [TableDescriptor::new_zeroed(); NUM_TABLES],
-            initialized: false,
+            initialized: for_precompute,
         }
     }

+    pub const fn new_for_precompute() -> Self {
+        Self::_new(true)
+    }
+
+    #[cfg(test)]
+    pub fn new_for_runtime() -> Self {
+        Self::_new(false)
+    }
+
     /// Helper to calculate the lvl2 and lvl3 indices from an address.
     #[inline(always)]
     fn lvl2_lvl3_index_from_page_addr(
@@ -297,6 +347,18 @@
         Ok((lvl2_index, lvl3_index))
     }

+    /// Returns the PageDescriptor corresponding to the supplied page address.
+    #[inline(always)]
+    fn page_descriptor_from_page_addr(
+        &self,
+        virt_page_addr: PageAddress<Virtual>,
+    ) -> Result<&PageDescriptor, &'static str> {
+        let (lvl2_index, lvl3_index) = self.lvl2_lvl3_index_from_page_addr(virt_page_addr)?;
+        let desc = &self.lvl3[lvl2_index][lvl3_index];
+
+        Ok(desc)
+    }
+
     /// Sets the PageDescriptor corresponding to the supplied page address.
     ///
     /// Doesn't allow overriding an already valid page.
@@ -325,24 +387,23 @@
 impl<const NUM_TABLES: usize> memory::mmu::translation_table::interface::TranslationTable
     for FixedSizeTranslationTable<NUM_TABLES>
 {
-    fn init(&mut self) {
+    fn init(&mut self) -> Result<(), &'static str> {
         if self.initialized {
-            return;
+            return Ok(());
         }

         // Populate the l2 entries.
         for (lvl2_nr, lvl2_entry) in self.lvl2.iter_mut().enumerate() {
-            let phys_table_addr = self.lvl3[lvl2_nr].phys_start_addr();
+            let virt_table_addr = self.lvl3[lvl2_nr].virt_start_addr();
+            let phys_table_addr = memory::mmu::try_kernel_virt_addr_to_phys_addr(virt_table_addr)?;

             let new_desc = TableDescriptor::from_next_lvl_table_addr(phys_table_addr);
             *lvl2_entry = new_desc;
         }

         self.initialized = true;
-    }

-    fn phys_base_address(&self) -> Address<Physical> {
-        self.lvl2.phys_start_addr()
+        Ok(())
     }

     unsafe fn map_at(
@@ -372,6 +433,45 @@

         Ok(())
     }
+
+    fn try_virt_page_addr_to_phys_page_addr(
+        &self,
+        virt_page_addr: PageAddress<Virtual>,
+    ) -> Result<PageAddress<Physical>, &'static str> {
+        let page_desc = self.page_descriptor_from_page_addr(virt_page_addr)?;
+
+        if !page_desc.is_valid() {
+            return Err("Page marked invalid");
+        }
+
+        Ok(page_desc.output_page_addr())
+    }
+
+    fn try_page_attributes(
+        &self,
+        virt_page_addr: PageAddress<Virtual>,
+    ) -> Result<AttributeFields, &'static str> {
+        let page_desc = self.page_descriptor_from_page_addr(virt_page_addr)?;
+
+        if !page_desc.is_valid() {
+            return Err("Page marked invalid");
+        }
+
+        page_desc.try_attributes()
+    }
+
+    /// Try to translate a virtual address to a physical address.
+    ///
+    /// Will only succeed if there exists a valid mapping for the input address.
+    fn try_virt_addr_to_phys_addr(
+        &self,
+        virt_addr: Address<Virtual>,
+    ) -> Result<Address<Physical>, &'static str> {
+        let virt_page = PageAddress::from(virt_addr.align_down_page());
+        let phys_page = self.try_virt_page_addr_to_phys_page_addr(virt_page)?;
+
+        Ok(phys_page.into_inner() + virt_addr.offset_into_page())
+    }
 }

 //--------------------------------------------------------------------------------------------------

diff -uNr 14_virtual_mem_part2_mmio_remap/kernel/src/bsp/raspberrypi/kernel.ld 15_virtual_mem_part3_precomputed_tables/kernel/src/bsp/raspberrypi/kernel.ld
--- 14_virtual_mem_part2_mmio_remap/kernel/src/bsp/raspberrypi/kernel.ld
+++ 15_virtual_mem_part3_precomputed_tables/kernel/src/bsp/raspberrypi/kernel.ld
@@ -3,6 +3,8 @@
  * Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
  */

+INCLUDE kernel_virt_addr_space_size.ld;
+
 PAGE_SIZE = 64K;
 PAGE_MASK = PAGE_SIZE - 1;

@@ -89,7 +91,7 @@
     . += 8 * 1024 * 1024;
     __mmio_remap_end_exclusive = .;

-    ASSERT((. & PAGE_MASK) == 0, "MMIO remap reservation is not page aligned")
+    ASSERT((. & PAGE_MASK) == 0, "End of boot core stack is not page aligned")

     /***********************************************************************************************
     * Misc

diff -uNr 14_virtual_mem_part2_mmio_remap/kernel/src/bsp/raspberrypi/kernel_virt_addr_space_size.ld 15_virtual_mem_part3_precomputed_tables/kernel/src/bsp/raspberrypi/kernel_virt_addr_space_size.ld
--- 14_virtual_mem_part2_mmio_remap/kernel/src/bsp/raspberrypi/kernel_virt_addr_space_size.ld
+++ 15_virtual_mem_part3_precomputed_tables/kernel/src/bsp/raspberrypi/kernel_virt_addr_space_size.ld
@@ -0,0 +1 @@
+__kernel_virt_addr_space_size = 1024 * 1024 * 1024

diff -uNr 14_virtual_mem_part2_mmio_remap/kernel/src/bsp/raspberrypi/memory/mmu.rs 15_virtual_mem_part3_precomputed_tables/kernel/src/bsp/raspberrypi/memory/mmu.rs
--- 14_virtual_mem_part2_mmio_remap/kernel/src/bsp/raspberrypi/memory/mmu.rs
+++ 15_virtual_mem_part3_precomputed_tables/kernel/src/bsp/raspberrypi/memory/mmu.rs
@@ -7,8 +7,8 @@
 use crate::{
     memory::{
         mmu::{
-            self as generic_mmu, AccessPermissions, AddressSpace, AssociatedTranslationTable,
-            AttributeFields, MemAttributes, MemoryRegion, PageAddress, TranslationGranule,
+            self as generic_mmu, AddressSpace, AssociatedTranslationTable, AttributeFields,
+            MemoryRegion, PageAddress, TranslationGranule,
         },
         Physical, Virtual,
     },
@@ -31,7 +31,7 @@
 pub type KernelGranule = TranslationGranule<{ 64 * 1024 }>;

 /// The kernel's virtual address space defined by this BSP.
-pub type KernelVirtAddrSpace = AddressSpace<{ 1024 * 1024 * 1024 }>;
+pub type KernelVirtAddrSpace = AddressSpace<{ kernel_virt_addr_space_size() }>;

 //--------------------------------------------------------------------------------------------------
 // Global instances
@@ -43,13 +43,35 @@
 ///
 /// That is, `size_of(InitStateLock<KernelTranslationTable>) == size_of(KernelTranslationTable)`.
 /// There is a unit tests that checks this porperty.
+#[link_section = ".data"]
+#[no_mangle]
 static KERNEL_TABLES: InitStateLock<KernelTranslationTable> =
-    InitStateLock::new(KernelTranslationTable::new());
+    InitStateLock::new(KernelTranslationTable::new_for_precompute());
+
+/// This value is needed during early boot for MMU setup.
+///
+/// This will be patched to the correct value by the "translation table tool" after linking. This
+/// given value here is just a dummy.
+#[link_section = ".text._start_arguments"]
+#[no_mangle]
+static PHYS_KERNEL_TABLES_BASE_ADDR: u64 = 0xCCCCAAAAFFFFEEEE;

 //--------------------------------------------------------------------------------------------------
 // Private Code
 //--------------------------------------------------------------------------------------------------

+/// This is a hack for retrieving the value for the kernel's virtual address space size as a
+/// constant from a common place, since it is needed as a compile-time/link-time constant in both,
+/// the linker script and the Rust sources.
+#[allow(clippy::needless_late_init)]
+const fn kernel_virt_addr_space_size() -> usize {
+    let __kernel_virt_addr_space_size;
+
+    include!("../kernel_virt_addr_space_size.ld");
+
+    __kernel_virt_addr_space_size
+}
+
 /// Helper function for calculating the number of pages the given parameter spans.
 const fn size_to_num_pages(size: usize) -> usize {
     assert!(size > 0);
@@ -88,18 +110,22 @@
     MemoryRegion::new(start_page_addr, end_exclusive_page_addr)
 }

-// The binary is still identity mapped, so use this trivial conversion function for mapping below.
-
+// There is no reason to expect the following conversions to fail, since they were generated offline
+// by the `translation table tool`. If it doesn't work, a panic due to the unwraps is justified.
 fn kernel_virt_to_phys_region(virt_region: MemoryRegion<Virtual>) -> MemoryRegion<Physical> {
-    MemoryRegion::new(
-        PageAddress::from(virt_region.start_page_addr().into_inner().as_usize()),
-        PageAddress::from(
-            virt_region
-                .end_exclusive_page_addr()
-                .into_inner()
-                .as_usize(),
-        ),
-    )
+    let phys_start_page_addr =
+        generic_mmu::try_kernel_virt_page_addr_to_phys_page_addr(virt_region.start_page_addr())
+            .unwrap();
+
+    let phys_end_exclusive_page_addr = phys_start_page_addr
+        .checked_offset(virt_region.num_pages() as isize)
+        .unwrap();
+
+    MemoryRegion::new(phys_start_page_addr, phys_end_exclusive_page_addr)
+}
+
+fn kernel_page_attributes(virt_page_addr: PageAddress<Virtual>) -> AttributeFields {
+    generic_mmu::try_kernel_page_attributes(virt_page_addr).unwrap()
 }

 //--------------------------------------------------------------------------------------------------
@@ -121,109 +147,33 @@
     MemoryRegion::new(start_page_addr, end_exclusive_page_addr)
 }

-/// Map the kernel binary.
+/// Add mapping records for the kernel binary.
 ///
-/// # Safety
-///
-/// - Any miscalculation or attribute error will likely be fatal. Needs careful manual checking.
-pub unsafe fn kernel_map_binary() -> Result<(), &'static str> {
-    generic_mmu::kernel_map_at(
+/// The actual translation table entries for the kernel binary are generated using the offline
+/// `translation table tool` and patched into the kernel binary. This function just adds the mapping
+/// record entries.
+pub fn kernel_add_mapping_records_for_precomputed() {
+    let virt_boot_core_stack_region = virt_boot_core_stack_region();
+    generic_mmu::kernel_add_mapping_record(
         "Kernel boot-core stack",
-        &virt_boot_core_stack_region(),
-        &kernel_virt_to_phys_region(virt_boot_core_stack_region()),
-        &AttributeFields {
-            mem_attributes: MemAttributes::CacheableDRAM,
-            acc_perms: AccessPermissions::ReadWrite,
-            execute_never: true,
-        },
-    )?;
+        &virt_boot_core_stack_region,
+        &kernel_virt_to_phys_region(virt_boot_core_stack_region),
+        &kernel_page_attributes(virt_boot_core_stack_region.start_page_addr()),
+    );

-    generic_mmu::kernel_map_at(
+    let virt_code_region = virt_code_region();
+    generic_mmu::kernel_add_mapping_record(
         "Kernel code and RO data",
-        &virt_code_region(),
-        &kernel_virt_to_phys_region(virt_code_region()),
-        &AttributeFields {
-            mem_attributes: MemAttributes::CacheableDRAM,
-            acc_perms: AccessPermissions::ReadOnly,
-            execute_never: false,
-        },
-    )?;
+        &virt_code_region,
+        &kernel_virt_to_phys_region(virt_code_region),
+        &kernel_page_attributes(virt_code_region.start_page_addr()),
+    );

-    generic_mmu::kernel_map_at(
+    let virt_data_region = virt_data_region();
+    generic_mmu::kernel_add_mapping_record(
         "Kernel data and bss",
-        &virt_data_region(),
-        &kernel_virt_to_phys_region(virt_data_region()),
-        &AttributeFields {
-            mem_attributes: MemAttributes::CacheableDRAM,
-            acc_perms: AccessPermissions::ReadWrite,
-            execute_never: true,
-        },
-    )?;
-
-    Ok(())
-}
-
-//--------------------------------------------------------------------------------------------------
-// Testing
-//--------------------------------------------------------------------------------------------------
-
-#[cfg(test)]
-mod tests {
-    use super::*;
-    use core::{cell::UnsafeCell, ops::Range};
-    use test_macros::kernel_test;
-
-    /// Check alignment of the kernel's virtual memory layout sections.
-    #[kernel_test]
-    fn virt_mem_layout_sections_are_64KiB_aligned() {
-        for i in [
-            virt_boot_core_stack_region,
-            virt_code_region,
-            virt_data_region,
-        ]
-        .iter()
-        {
-            let start = i().start_page_addr().into_inner();
-            let end_exclusive = i().end_exclusive_page_addr().into_inner();
-
-            assert!(start.is_page_aligned());
-            assert!(end_exclusive.is_page_aligned());
-            assert!(end_exclusive >= start);
-        }
-    }
-
-    /// Ensure the kernel's virtual memory layout is free of overlaps.
-    #[kernel_test]
-    fn virt_mem_layout_has_no_overlaps() {
-        let layout = [
-            virt_boot_core_stack_region(),
-            virt_code_region(),
-            virt_data_region(),
-        ];
-
-        for (i, first_range) in layout.iter().enumerate() {
-            for second_range in layout.iter().skip(i + 1) {
-                assert!(!first_range.overlaps(second_range))
-            }
-        }
-    }
-
-    /// Check if KERNEL_TABLES is in .bss.
-    #[kernel_test]
-    fn kernel_tables_in_bss() {
-        extern "Rust" {
-            static __bss_start: UnsafeCell<u64>;
-            static __bss_end_exclusive: UnsafeCell<u64>;
-        }
-
-        let bss_range = unsafe {
-            Range {
-                start: __bss_start.get(),
-                end: __bss_end_exclusive.get(),
-            }
-        };
-        let kernel_tables_addr = &KERNEL_TABLES as *const _ as usize as *mut u64;
-
-        assert!(bss_range.contains(&kernel_tables_addr));
-    }
+        &virt_data_region,
+        &kernel_virt_to_phys_region(virt_data_region),
+        &kernel_page_attributes(virt_data_region.start_page_addr()),
+    );
 }

diff -uNr 14_virtual_mem_part2_mmio_remap/kernel/src/lib.rs 15_virtual_mem_part3_precomputed_tables/kernel/src/lib.rs
--- 14_virtual_mem_part2_mmio_remap/kernel/src/lib.rs
+++ 15_virtual_mem_part3_precomputed_tables/kernel/src/lib.rs
@@ -187,17 +187,7 @@
 #[no_mangle]
 unsafe fn kernel_init() -> ! {
     exception::handling_init();
-
-    let phys_kernel_tables_base_addr = match memory::mmu::kernel_map_binary() {
-        Err(string) => panic!("Error mapping kernel binary: {}", string),
-        Ok(addr) => addr,
-    };
-
-    if let Err(e) = memory::mmu::enable_mmu_and_caching(phys_kernel_tables_base_addr) {
-        panic!("Enabling MMU failed: {}", e);
-    }
-
-    memory::mmu::post_enable_init();
+    memory::init();
     bsp::driver::qemu_bring_up_console();

     test_main();

diff -uNr 14_virtual_mem_part2_mmio_remap/kernel/src/main.rs 15_virtual_mem_part3_precomputed_tables/kernel/src/main.rs
--- 14_virtual_mem_part2_mmio_remap/kernel/src/main.rs
+++ 15_virtual_mem_part3_precomputed_tables/kernel/src/main.rs
@@ -17,27 +17,16 @@

 /// Early init code.
 ///
+/// When this code runs, virtual memory is already enabled.
+///
 /// # Safety
 ///
 /// - Only a single core must be active and running this function.
-/// - The init calls in this function must appear in the correct order:
-///     - MMU + Data caching must be activated at the earliest. Without it, any atomic operations,
-///       e.g. the yet-to-be-introduced spinlocks in the device drivers (which currently employ
-///       IRQSafeNullLocks instead of spinlocks), will fail to work (properly) on the RPi SoCs.
+/// - Printing will not work until the respective driver's MMIO is remapped.
 #[no_mangle]
 unsafe fn kernel_init() -> ! {
     exception::handling_init();
-
-    let phys_kernel_tables_base_addr = match memory::mmu::kernel_map_binary() {
-        Err(string) => panic!("Error mapping kernel binary: {}", string),
-        Ok(addr) => addr,
-    };
-
-    if let Err(e) = memory::mmu::enable_mmu_and_caching(phys_kernel_tables_base_addr) {
-        panic!("Enabling MMU failed: {}", e);
-    }
-
-    memory::mmu::post_enable_init();
+    memory::init();

     // Initialize the BSP driver subsystem.
     if let Err(x) = bsp::driver::init() {
@@ -47,6 +36,8 @@
     // Initialize all device drivers.
     driver::driver_manager().init_drivers_and_irqs();

+    bsp::memory::mmu::kernel_add_mapping_records_for_precomputed();
+
     // Unmask interrupts on the boot CPU core.
     exception::asynchronous::local_irq_unmask();


diff -uNr 14_virtual_mem_part2_mmio_remap/kernel/src/memory/mmu/translation_table.rs 15_virtual_mem_part3_precomputed_tables/kernel/src/memory/mmu/translation_table.rs
--- 14_virtual_mem_part2_mmio_remap/kernel/src/memory/mmu/translation_table.rs
+++ 15_virtual_mem_part3_precomputed_tables/kernel/src/memory/mmu/translation_table.rs
@@ -23,6 +23,8 @@

 /// Translation table interfaces.
 pub mod interface {
+    use crate::memory::mmu::PageAddress;
+
     use super::*;

     /// Translation table operations.
@@ -33,10 +35,7 @@
         ///
         /// - Implementor must ensure that this function can run only once or is harmless if invoked
         ///   multiple times.
-        fn init(&mut self);
-
-        /// The translation table's base address to be used for programming the MMU.
-        fn phys_base_address(&self) -> Address<Physical>;
+        fn init(&mut self) -> Result<(), &'static str>;

         /// Map the given virtual memory region to the given physical memory region.
         ///
@@ -53,6 +52,30 @@
             phys_region: &MemoryRegion<Physical>,
             attr: &AttributeFields,
         ) -> Result<(), &'static str>;
+
+        /// Try to translate a virtual page address to a physical page address.
+        ///
+        /// Will only succeed if there exists a valid mapping for the input page.
+        fn try_virt_page_addr_to_phys_page_addr(
+            &self,
+            virt_page_addr: PageAddress<Virtual>,
+        ) -> Result<PageAddress<Physical>, &'static str>;
+
+        /// Try to get the attributes of a page.
+        ///
+        /// Will only succeed if there exists a valid mapping for the input page.
+        fn try_page_attributes(
+            &self,
+            virt_page_addr: PageAddress<Virtual>,
+        ) -> Result<AttributeFields, &'static str>;
+
+        /// Try to translate a virtual address to a physical address.
+        ///
+        /// Will only succeed if there exists a valid mapping for the input address.
+        fn try_virt_addr_to_phys_addr(
+            &self,
+            virt_addr: Address<Virtual>,
+        ) -> Result<Address<Physical>, &'static str>;
     }
 }

@@ -72,9 +95,9 @@
     #[kernel_test]
     fn translationtable_implementation_sanity() {
         // This will occupy a lot of space on the stack.
-        let mut tables = MinSizeTranslationTable::new();
+        let mut tables = MinSizeTranslationTable::new_for_runtime();

-        tables.init();
+        assert_eq!(tables.init(), Ok(()));

         let virt_start_page_addr: PageAddress<Virtual> = PageAddress::from(0);
         let virt_end_exclusive_page_addr: PageAddress<Virtual> =
@@ -94,5 +117,21 @@
         };

         unsafe { assert_eq!(tables.map_at(&virt_region, &phys_region, &attr), Ok(())) };
+
+        assert_eq!(
+            tables.try_virt_page_addr_to_phys_page_addr(virt_start_page_addr),
+            Ok(phys_start_page_addr)
+        );
+
+        assert_eq!(
+            tables.try_page_attributes(virt_start_page_addr.checked_offset(6).unwrap()),
+            Err("Page marked invalid")
+        );
+
+        assert_eq!(tables.try_page_attributes(virt_start_page_addr), Ok(attr));
+
+        let virt_addr = virt_start_page_addr.into_inner() + 0x100;
+        let phys_addr = phys_start_page_addr.into_inner() + 0x100;
+        assert_eq!(tables.try_virt_addr_to_phys_addr(virt_addr), Ok(phys_addr));
     }
 }

diff -uNr 14_virtual_mem_part2_mmio_remap/kernel/src/memory/mmu.rs 15_virtual_mem_part3_precomputed_tables/kernel/src/memory/mmu.rs
--- 14_virtual_mem_part2_mmio_remap/kernel/src/memory/mmu.rs
+++ 15_virtual_mem_part3_precomputed_tables/kernel/src/memory/mmu.rs
@@ -16,7 +16,8 @@
 use crate::{
     bsp,
     memory::{Address, Physical, Virtual},
-    synchronization, warn,
+    synchronization::{self, interface::Mutex},
+    warn,
 };
 use core::{fmt, num::NonZeroUsize};

@@ -73,17 +74,9 @@
 // Private Code
 //--------------------------------------------------------------------------------------------------
 use interface::MMU;
-use synchronization::interface::*;
+use synchronization::interface::ReadWriteEx;
 use translation_table::interface::TranslationTable;

-/// Query the BSP for the reserved virtual addresses for MMIO remapping and initialize the kernel's
-/// MMIO VA allocator with it.
-fn kernel_init_mmio_va_allocator() {
-    let region = bsp::memory::mmu::virt_mmio_remap_region();
-
-    page_alloc::kernel_mmio_va_allocator().lock(|allocator| allocator.init(region));
-}
-
 /// Map a region in the kernel's translation tables.
 ///
 /// No input checks done, input is passed through to the architectural implementation.
@@ -101,13 +94,21 @@
     bsp::memory::mmu::kernel_translation_tables()
         .write(|tables| tables.map_at(virt_region, phys_region, attr))?;

-    if let Err(x) = mapping_record::kernel_add(name, virt_region, phys_region, attr) {
-        warn!("{}", x);
-    }
+    kernel_add_mapping_record(name, virt_region, phys_region, attr);

     Ok(())
 }

+/// Try to translate a kernel virtual address to a physical address.
+///
+/// Will only succeed if there exists a valid mapping for the input address.
+fn try_kernel_virt_addr_to_phys_addr(
+    virt_addr: Address<Virtual>,
+) -> Result<Address<Physical>, &'static str> {
+    bsp::memory::mmu::kernel_translation_tables()
+        .read(|tables| tables.try_virt_addr_to_phys_addr(virt_addr))
+}
+
 //--------------------------------------------------------------------------------------------------
 // Public Code
 //--------------------------------------------------------------------------------------------------
@@ -155,27 +156,24 @@
     }
 }

-/// Raw mapping of a virtual to physical region in the kernel translation tables.
-///
-/// Prevents mapping into the MMIO range of the tables.
-///
-/// # Safety
-///
-/// - See `kernel_map_at_unchecked()`.
-/// - Does not prevent aliasing. Currently, the callers must be trusted.
-pub unsafe fn kernel_map_at(
+/// Query the BSP for the reserved virtual addresses for MMIO remapping and initialize the kernel's
+/// MMIO VA allocator with it.
+pub fn kernel_init_mmio_va_allocator() {
+    let region = bsp::memory::mmu::virt_mmio_remap_region();
+
+    page_alloc::kernel_mmio_va_allocator().lock(|allocator| allocator.init(region));
+}
+
+/// Add an entry to the mapping info record.
+pub fn kernel_add_mapping_record(
     name: &'static str,
     virt_region: &MemoryRegion<Virtual>,
     phys_region: &MemoryRegion<Physical>,
     attr: &AttributeFields,
-) -> Result<(), &'static str> {
-    if bsp::memory::mmu::virt_mmio_remap_region().overlaps(virt_region) {
-        return Err("Attempt to manually map into MMIO region");
+) {
+    if let Err(x) = mapping_record::kernel_add(name, virt_region, phys_region, attr) {
+        warn!("{}", x);
     }
-
-    kernel_map_at_unchecked(name, virt_region, phys_region, attr)?;
-
-    Ok(())
 }

 /// MMIO remapping in the kernel translation tables.
@@ -224,21 +222,29 @@
     Ok(virt_addr + offset_into_start_page)
 }

-/// Map the kernel's binary. Returns the translation table's base address.
-///
-/// # Safety
+/// Try to translate a kernel virtual page address to a physical page address.
 ///
-/// - See [`bsp::memory::mmu::kernel_map_binary()`].
-pub unsafe fn kernel_map_binary() -> Result<Address<Physical>, &'static str> {
-    let phys_kernel_tables_base_addr =
-        bsp::memory::mmu::kernel_translation_tables().write(|tables| {
-            tables.init();
-            tables.phys_base_address()
-        });
+/// Will only succeed if there exists a valid mapping for the input page.
+pub fn try_kernel_virt_page_addr_to_phys_page_addr(
+    virt_page_addr: PageAddress<Virtual>,
+) -> Result<PageAddress<Physical>, &'static str> {
+    bsp::memory::mmu::kernel_translation_tables()
+        .read(|tables| tables.try_virt_page_addr_to_phys_page_addr(virt_page_addr))
+}

-    bsp::memory::mmu::kernel_map_binary()?;
+/// Try to get the attributes of a kernel page.
+///
+/// Will only succeed if there exists a valid mapping for the input page.
+pub fn try_kernel_page_attributes(
+    virt_page_addr: PageAddress<Virtual>,
+) -> Result<AttributeFields, &'static str> {
+    bsp::memory::mmu::kernel_translation_tables()
+        .read(|tables| tables.try_page_attributes(virt_page_addr))
+}

-    Ok(phys_kernel_tables_base_addr)
+/// Human-readable print of all recorded kernel mappings.
+pub fn kernel_print_mappings() {
+    mapping_record::kernel_print()
 }

 /// Enable the MMU and data + instruction caching.
@@ -246,56 +252,9 @@
 /// # Safety
 ///
 /// - Crucial function during kernel init. Changes the the complete memory view of the processor.
+#[inline(always)]
 pub unsafe fn enable_mmu_and_caching(
     phys_tables_base_addr: Address<Physical>,
 ) -> Result<(), MMUEnableError> {
     arch_mmu::mmu().enable_mmu_and_caching(phys_tables_base_addr)
 }
-
-/// Finish initialization of the MMU subsystem.
-pub fn post_enable_init() {
-    kernel_init_mmio_va_allocator();
-}
-
-/// Human-readable print of all recorded kernel mappings.
-pub fn kernel_print_mappings() {
-    mapping_record::kernel_print()
-}
-
-//--------------------------------------------------------------------------------------------------
-// Testing
-//--------------------------------------------------------------------------------------------------
-
-#[cfg(test)]
-mod tests {
-    use super::*;
-    use crate::memory::mmu::{AccessPermissions, MemAttributes, PageAddress};
-    use test_macros::kernel_test;
-
-    /// Check that you cannot map into the MMIO VA range from kernel_map_at().
-    #[kernel_test]
-    fn no_manual_mmio_map() {
-        let phys_start_page_addr: PageAddress<Physical> = PageAddress::from(0);
-        let phys_end_exclusive_page_addr: PageAddress<Physical> =
-            phys_start_page_addr.checked_offset(5).unwrap();
-        let phys_region = MemoryRegion::new(phys_start_page_addr, phys_end_exclusive_page_addr);
-
-        let num_pages = NonZeroUsize::new(phys_region.num_pages()).unwrap();
-        let virt_region = page_alloc::kernel_mmio_va_allocator()
-            .lock(|allocator| allocator.alloc(num_pages))
-            .unwrap();
-
-        let attr = AttributeFields {
-            mem_attributes: MemAttributes::CacheableDRAM,
-            acc_perms: AccessPermissions::ReadWrite,
-            execute_never: true,
-        };
-
-        unsafe {
-            assert_eq!(
-                kernel_map_at("test", &virt_region, &phys_region, &attr),
-                Err("Attempt to manually map into MMIO region")
-            )
-        };
-    }
-}

diff -uNr 14_virtual_mem_part2_mmio_remap/kernel/src/memory.rs 15_virtual_mem_part3_precomputed_tables/kernel/src/memory.rs
--- 14_virtual_mem_part2_mmio_remap/kernel/src/memory.rs
+++ 15_virtual_mem_part3_precomputed_tables/kernel/src/memory.rs
@@ -136,6 +136,11 @@
     }
 }

+/// Initialize the memory subsystem.
+pub fn init() {
+    mmu::kernel_init_mmio_va_allocator();
+}
+
 //--------------------------------------------------------------------------------------------------
 // Testing
 //--------------------------------------------------------------------------------------------------

diff -uNr 14_virtual_mem_part2_mmio_remap/kernel/tests/00_console_sanity.rs 15_virtual_mem_part3_precomputed_tables/kernel/tests/00_console_sanity.rs
--- 14_virtual_mem_part2_mmio_remap/kernel/tests/00_console_sanity.rs
+++ 15_virtual_mem_part3_precomputed_tables/kernel/tests/00_console_sanity.rs
@@ -18,17 +18,7 @@
     use console::console;

     exception::handling_init();
-
-    let phys_kernel_tables_base_addr = match memory::mmu::kernel_map_binary() {
-        Err(string) => panic!("Error mapping kernel binary: {}", string),
-        Ok(addr) => addr,
-    };
-
-    if let Err(e) = memory::mmu::enable_mmu_and_caching(phys_kernel_tables_base_addr) {
-        panic!("Enabling MMU failed: {}", e);
-    }
-
-    memory::mmu::post_enable_init();
+    memory::init();
     bsp::driver::qemu_bring_up_console();

     // Handshake

diff -uNr 14_virtual_mem_part2_mmio_remap/kernel/tests/01_timer_sanity.rs 15_virtual_mem_part3_precomputed_tables/kernel/tests/01_timer_sanity.rs
--- 14_virtual_mem_part2_mmio_remap/kernel/tests/01_timer_sanity.rs
+++ 15_virtual_mem_part3_precomputed_tables/kernel/tests/01_timer_sanity.rs
@@ -17,17 +17,7 @@
 #[no_mangle]
 unsafe fn kernel_init() -> ! {
     exception::handling_init();
-
-    let phys_kernel_tables_base_addr = match memory::mmu::kernel_map_binary() {
-        Err(string) => panic!("Error mapping kernel binary: {}", string),
-        Ok(addr) => addr,
-    };
-
-    if let Err(e) = memory::mmu::enable_mmu_and_caching(phys_kernel_tables_base_addr) {
-        panic!("Enabling MMU failed: {}", e);
-    }
-
-    memory::mmu::post_enable_init();
+    memory::init();
     bsp::driver::qemu_bring_up_console();

     // Depending on CPU arch, some timer bring-up code could go here. Not needed for the RPi.

diff -uNr 14_virtual_mem_part2_mmio_remap/kernel/tests/02_exception_sync_page_fault.rs 15_virtual_mem_part3_precomputed_tables/kernel/tests/02_exception_sync_page_fault.rs
--- 14_virtual_mem_part2_mmio_remap/kernel/tests/02_exception_sync_page_fault.rs
+++ 15_virtual_mem_part3_precomputed_tables/kernel/tests/02_exception_sync_page_fault.rs
@@ -22,26 +22,12 @@
 #[no_mangle]
 unsafe fn kernel_init() -> ! {
     exception::handling_init();
+    memory::init();
+    bsp::driver::qemu_bring_up_console();

     // This line will be printed as the test header.
     println!("Testing synchronous exception handling by causing a page fault");

-    let phys_kernel_tables_base_addr = match memory::mmu::kernel_map_binary() {
-        Err(string) => {
-            info!("Error mapping kernel binary: {}", string);
-            cpu::qemu_exit_failure()
-        }
-        Ok(addr) => addr,
-    };
-
-    if let Err(e) = memory::mmu::enable_mmu_and_caching(phys_kernel_tables_base_addr) {
-        info!("Enabling MMU failed: {}", e);
-        cpu::qemu_exit_failure()
-    }
-
-    memory::mmu::post_enable_init();
-    bsp::driver::qemu_bring_up_console();
-
     info!("Writing beyond mapped area to address 9 GiB...");
     let big_addr: u64 = 9 * 1024 * 1024 * 1024;
     core::ptr::read_volatile(big_addr as *mut u64);

diff -uNr 14_virtual_mem_part2_mmio_remap/kernel/tests/03_exception_restore_sanity.rs 15_virtual_mem_part3_precomputed_tables/kernel/tests/03_exception_restore_sanity.rs
--- 14_virtual_mem_part2_mmio_remap/kernel/tests/03_exception_restore_sanity.rs
+++ 15_virtual_mem_part3_precomputed_tables/kernel/tests/03_exception_restore_sanity.rs
@@ -31,26 +31,12 @@
 #[no_mangle]
 unsafe fn kernel_init() -> ! {
     exception::handling_init();
+    memory::init();
+    bsp::driver::qemu_bring_up_console();

     // This line will be printed as the test header.
     println!("Testing exception restore");

-    let phys_kernel_tables_base_addr = match memory::mmu::kernel_map_binary() {
-        Err(string) => {
-            info!("Error mapping kernel binary: {}", string);
-            cpu::qemu_exit_failure()
-        }
-        Ok(addr) => addr,
-    };
-
-    if let Err(e) = memory::mmu::enable_mmu_and_caching(phys_kernel_tables_base_addr) {
-        info!("Enabling MMU failed: {}", e);
-        cpu::qemu_exit_failure()
-    }
-
-    memory::mmu::post_enable_init();
-    bsp::driver::qemu_bring_up_console();
-
     info!("Making a dummy system call");

     // Calling this inside a function indirectly tests if the link register is restored properly.

diff -uNr 14_virtual_mem_part2_mmio_remap/kernel/tests/04_exception_irq_sanity.rs 15_virtual_mem_part3_precomputed_tables/kernel/tests/04_exception_irq_sanity.rs
--- 14_virtual_mem_part2_mmio_remap/kernel/tests/04_exception_irq_sanity.rs
+++ 15_virtual_mem_part3_precomputed_tables/kernel/tests/04_exception_irq_sanity.rs
@@ -15,20 +15,10 @@

 #[no_mangle]
 unsafe fn kernel_init() -> ! {
-    exception::handling_init();
-
-    let phys_kernel_tables_base_addr = match memory::mmu::kernel_map_binary() {
-        Err(string) => panic!("Error mapping kernel binary: {}", string),
-        Ok(addr) => addr,
-    };
-
-    if let Err(e) = memory::mmu::enable_mmu_and_caching(phys_kernel_tables_base_addr) {
-        panic!("Enabling MMU failed: {}", e);
-    }
-
-    memory::mmu::post_enable_init();
+    memory::init();
     bsp::driver::qemu_bring_up_console();

+    exception::handling_init();
     exception::asynchronous::local_irq_unmask();

     test_main();

diff -uNr 14_virtual_mem_part2_mmio_remap/Makefile 15_virtual_mem_part3_precomputed_tables/Makefile
--- 14_virtual_mem_part2_mmio_remap/Makefile
+++ 15_virtual_mem_part3_precomputed_tables/Makefile
@@ -72,10 +72,20 @@
 KERNEL_LINKER_SCRIPT = kernel.ld
 LAST_BUILD_CONFIG    = target/$(BSP).build_config

-KERNEL_ELF      = target/$(TARGET)/release/kernel
+KERNEL_ELF_RAW      = target/$(TARGET)/release/kernel
 # This parses cargo's dep-info file.
 # https://doc.rust-lang.org/cargo/guide/build-cache.html#dep-info-files
-KERNEL_ELF_DEPS = $(filter-out modulo: ,$(file < $(KERNEL_ELF).d)) $(KERNEL_MANIFEST) $(LAST_BUILD_CONFIG)
+KERNEL_ELF_RAW_DEPS = $(filter-out modulo: ,$(file < $(KERNEL_ELF_RAW).d)) $(KERNEL_MANIFEST) $(LAST_BUILD_CONFIG)
+
+##------------------------------------------------------------------------------
+## Translation tables
+##------------------------------------------------------------------------------
+TT_TOOL_PATH = tools/translation_table_tool
+
+KERNEL_ELF_TTABLES      = target/$(TARGET)/release/kernel+ttables
+KERNEL_ELF_TTABLES_DEPS = $(KERNEL_ELF_RAW) $(wildcard $(TT_TOOL_PATH)/*)
+
+KERNEL_ELF = $(KERNEL_ELF_TTABLES)



@@ -104,6 +114,7 @@
     -O binary

 EXEC_QEMU          = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)
+EXEC_TT_TOOL       = ruby $(TT_TOOL_PATH)/main.rb
 EXEC_TEST_DISPATCH = ruby ../common/tests/dispatch.rb
 EXEC_MINIPUSH      = ruby ../common/serial/minipush.rb

@@ -154,16 +165,24 @@
 ##------------------------------------------------------------------------------
 ## Compile the kernel ELF
 ##------------------------------------------------------------------------------
-$(KERNEL_ELF): $(KERNEL_ELF_DEPS)
+$(KERNEL_ELF_RAW): $(KERNEL_ELF_RAW_DEPS)
 	$(call color_header, "Compiling kernel ELF - $(BSP)")
 	@RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(RUSTC_CMD)

 ##------------------------------------------------------------------------------
+## Precompute the kernel translation tables and patch them into the kernel ELF
+##------------------------------------------------------------------------------
+$(KERNEL_ELF_TTABLES): $(KERNEL_ELF_TTABLES_DEPS)
+	$(call color_header, "Precomputing kernel translation tables and patching kernel ELF")
+	@cp $(KERNEL_ELF_RAW) $(KERNEL_ELF_TTABLES)
+	@$(DOCKER_TOOLS) $(EXEC_TT_TOOL) $(BSP) $(KERNEL_ELF_TTABLES)
+
+##------------------------------------------------------------------------------
 ## Generate the stripped kernel binary
 ##------------------------------------------------------------------------------
-$(KERNEL_BIN): $(KERNEL_ELF)
+$(KERNEL_BIN): $(KERNEL_ELF_TTABLES)
 	$(call color_header, "Generating stripped binary")
-	@$(OBJCOPY_CMD) $(KERNEL_ELF) $(KERNEL_BIN)
+	@$(OBJCOPY_CMD) $(KERNEL_ELF_TTABLES) $(KERNEL_BIN)
 	$(call color_progress_prefix, "Name")
 	@echo $(KERNEL_BIN)
 	$(call color_progress_prefix, "Size")
@@ -301,6 +320,7 @@
     TEST_ELF=$$(echo $$1 | sed -e 's/.*target/target/g')
     TEST_BINARY=$$(echo $$1.img | sed -e 's/.*target/target/g')

+    $(DOCKER_TOOLS) $(EXEC_TT_TOOL) $(BSP) $$TEST_ELF > /dev/null
     $(OBJCOPY_CMD) $$TEST_ELF $$TEST_BINARY
     $(DOCKER_TEST) $(EXEC_TEST_DISPATCH) $(EXEC_QEMU) $(QEMU_TEST_ARGS) -kernel $$TEST_BINARY
 endef

diff -uNr 14_virtual_mem_part2_mmio_remap/tools/translation_table_tool/arch.rb 15_virtual_mem_part3_precomputed_tables/tools/translation_table_tool/arch.rb
--- 14_virtual_mem_part2_mmio_remap/tools/translation_table_tool/arch.rb
+++ 15_virtual_mem_part3_precomputed_tables/tools/translation_table_tool/arch.rb
@@ -0,0 +1,312 @@
+# frozen_string_literal: true
+
+# SPDX-License-Identifier: MIT OR Apache-2.0
+#
+# Copyright (c) 2021-2023 Andre Richter <andre.o.richter@gmail.com>
+
+# Bitfield manipulation.
+class BitField
+    def initialize
+        @value = 0
+    end
+
+    def self.attr_bitfield(name, offset, num_bits)
+        define_method("#{name}=") do |bits|
+            mask = (2**num_bits) - 1
+
+            raise "Input out of range: #{name} = 0x#{bits.to_s(16)}" if (bits & ~mask).positive?
+
+            # Clear bitfield
+            @value &= ~(mask << offset)
+
+            # Set it
+            @value |= (bits << offset)
+        end
+    end
+
+    def to_i
+        @value
+    end
+
+    def size_in_byte
+        8
+    end
+end
+
+# An array class that knows its memory location.
+class CArray < Array
+    attr_reader :phys_start_addr
+
+    def initialize(phys_start_addr, size, &block)
+        @phys_start_addr = phys_start_addr
+
+        super(size, &block)
+    end
+
+    def size_in_byte
+        inject(0) { |sum, n| sum + n.size_in_byte }
+    end
+end
+
+#---------------------------------------------------------------------------------------------------
+# Arch::
+#---------------------------------------------------------------------------------------------------
+module Arch
+#---------------------------------------------------------------------------------------------------
+# Arch::ARMv8
+#---------------------------------------------------------------------------------------------------
+module ARMv8
+# ARMv8 Table Descriptor.
+class Stage1TableDescriptor < BitField
+    module NextLevelTableAddr
+        OFFSET = 16
+        NUMBITS = 32
+    end
+
+    module Type
+        OFFSET = 1
+        NUMBITS = 1
+
+        BLOCK = 0
+        TABLE = 1
+    end
+
+    module Valid
+        OFFSET = 0
+        NUMBITS = 1
+
+        FALSE = 0
+        TRUE = 1
+    end
+
+    attr_bitfield(:__next_level_table_addr, NextLevelTableAddr::OFFSET, NextLevelTableAddr::NUMBITS)
+    attr_bitfield(:type, Type::OFFSET, Type::NUMBITS)
+    attr_bitfield(:valid, Valid::OFFSET, Valid::NUMBITS)
+
+    def next_level_table_addr=(addr)
+        addr >>= Granule64KiB::SHIFT
+
+        self.__next_level_table_addr = addr
+    end
+
+    private :__next_level_table_addr=
+end
+
+# ARMv8 level 3 page descriptor.
+class Stage1PageDescriptor < BitField
+    module UXN
+        OFFSET = 54
+        NUMBITS = 1
+
+        FALSE = 0
+        TRUE = 1
+    end
+
+    module PXN
+        OFFSET = 53
+        NUMBITS = 1
+
+        FALSE = 0
+        TRUE = 1
+    end
+
+    module OutputAddr
+        OFFSET = 16
+        NUMBITS = 32
+    end
+
+    module AF
+        OFFSET = 10
+        NUMBITS = 1
+
+        FALSE = 0
+        TRUE = 1
+    end
+
+    module SH
+        OFFSET = 8
+        NUMBITS = 2
+
+        INNER_SHAREABLE = 0b11
+    end
+
+    module AP
+        OFFSET = 6
+        NUMBITS = 2
+
+        RW_EL1 = 0b00
+        RO_EL1 = 0b10
+    end
+
+    module AttrIndx
+        OFFSET = 2
+        NUMBITS = 3
+    end
+
+    module Type
+        OFFSET = 1
+        NUMBITS = 1
+
+        RESERVED_INVALID = 0
+        PAGE = 1
+    end
+
+    module Valid
+        OFFSET = 0
+        NUMBITS = 1
+
+        FALSE = 0
+        TRUE = 1
+    end
+
+    attr_bitfield(:uxn, UXN::OFFSET, UXN::NUMBITS)
+    attr_bitfield(:pxn, PXN::OFFSET, PXN::NUMBITS)
+    attr_bitfield(:__output_addr, OutputAddr::OFFSET, OutputAddr::NUMBITS)
+    attr_bitfield(:af, AF::OFFSET, AF::NUMBITS)
+    attr_bitfield(:sh, SH::OFFSET, SH::NUMBITS)
+    attr_bitfield(:ap, AP::OFFSET, AP::NUMBITS)
+    attr_bitfield(:attr_indx, AttrIndx::OFFSET, AttrIndx::NUMBITS)
+    attr_bitfield(:type, Type::OFFSET, Type::NUMBITS)
+    attr_bitfield(:valid, Valid::OFFSET, Valid::NUMBITS)
+
+    def output_addr=(addr)
+        addr >>= Granule64KiB::SHIFT
+
+        self.__output_addr = addr
+    end
+
+    private :__output_addr=
+end
+
+# Translation table representing the structure defined in translation_table.rs.
+class TranslationTable
+    module MAIR
+        NORMAL = 1
+    end
+
+    def initialize
+        do_sanity_checks
+
+        num_lvl2_tables = BSP.kernel_virt_addr_space_size >> Granule512MiB::SHIFT
+
+        @lvl3 = new_lvl3(num_lvl2_tables, BSP.phys_addr_of_kernel_tables)
+
+        @lvl2_phys_start_addr = @lvl3.phys_start_addr + @lvl3.size_in_byte
+        @lvl2 = new_lvl2(num_lvl2_tables, @lvl2_phys_start_addr)
+
+        populate_lvl2_entries
+    end
+
+    def map_at(virt_region, phys_region, attributes)
+        return if virt_region.empty?
+
+        raise if virt_region.size != phys_region.size
+        raise if phys_region.last > BSP.phys_addr_space_end_page
+
+        virt_region.zip(phys_region).each do |virt_page, phys_page|
+            desc = page_descriptor_from(virt_page)
+            set_lvl3_entry(desc, phys_page, attributes)
+        end
+    end
+
+    def to_binary
+        data = @lvl3.flatten.map(&:to_i) + @lvl2.map(&:to_i)
+        data.pack('Q<*') # "Q" == uint64_t, "<" == little endian
+    end
+
+    def phys_tables_base_addr_binary
+        [@lvl2_phys_start_addr].pack('Q<*') # "Q" == uint64_t, "<" == little endian
+    end
+
+    def phys_tables_base_addr
+        @lvl2_phys_start_addr
+    end
+
+    private
+
+    def do_sanity_checks
+        raise unless BSP.kernel_granule::SIZE == Granule64KiB::SIZE
+        raise unless (BSP.kernel_virt_addr_space_size modulo Granule512MiB::SIZE).zero?
+    end
+
+    def new_lvl3(num_lvl2_tables, start_addr)
+        CArray.new(start_addr, num_lvl2_tables) do
+            temp = CArray.new(start_addr, 8192) do
+                Stage1PageDescriptor.new
+            end
+            start_addr += temp.size_in_byte
+
+            temp
+        end
+    end
+
+    def new_lvl2(num_lvl2_tables, start_addr)
+        CArray.new(start_addr, num_lvl2_tables) do
+            Stage1TableDescriptor.new
+        end
+    end
+
+    def populate_lvl2_entries
+        @lvl2.each_with_index do |descriptor, i|
+            descriptor.next_level_table_addr = @lvl3[i].phys_start_addr
+            descriptor.type = Stage1TableDescriptor::Type::TABLE
+            descriptor.valid = Stage1TableDescriptor::Valid::TRUE
+        end
+    end
+
+    def lvl2_lvl3_index_from(addr)
+        lvl2_index = addr >> Granule512MiB::SHIFT
+        lvl3_index = (addr & Granule512MiB::MASK) >> Granule64KiB::SHIFT
+
+        raise unless lvl2_index < @lvl2.size
+
+        [lvl2_index, lvl3_index]
+    end
+
+    def page_descriptor_from(virt_addr)
+        lvl2_index, lvl3_index = lvl2_lvl3_index_from(virt_addr)
+
+        @lvl3[lvl2_index][lvl3_index]
+    end
+
+    # rubocop:disable Metrics/MethodLength
+    def set_attributes(desc, attributes)
+        case attributes.mem_attributes
+        when :CacheableDRAM
+            desc.sh = Stage1PageDescriptor::SH::INNER_SHAREABLE
+            desc.attr_indx = MAIR::NORMAL
+        else
+            raise 'Invalid input'
+        end
+
+        desc.ap = case attributes.acc_perms
+                  when :ReadOnly
+                      Stage1PageDescriptor::AP::RO_EL1
+                  when :ReadWrite
+                      Stage1PageDescriptor::AP::RW_EL1
+                  else
+                      raise 'Invalid input'
+
+                  end
+
+        desc.pxn = if attributes.execute_never
+                       Stage1PageDescriptor::PXN::TRUE
+                   else
+                       Stage1PageDescriptor::PXN::FALSE
+                   end
+
+        desc.uxn = Stage1PageDescriptor::UXN::TRUE
+    end
+    # rubocop:enable Metrics/MethodLength
+
+    def set_lvl3_entry(desc, output_addr, attributes)
+        desc.output_addr = output_addr
+        desc.af = Stage1PageDescriptor::AF::TRUE
+        desc.type = Stage1PageDescriptor::Type::PAGE
+        desc.valid = Stage1PageDescriptor::Valid::TRUE
+
+        set_attributes(desc, attributes)
+    end
+end
+end
+end

diff -uNr 14_virtual_mem_part2_mmio_remap/tools/translation_table_tool/bsp.rb 15_virtual_mem_part3_precomputed_tables/tools/translation_table_tool/bsp.rb
--- 14_virtual_mem_part2_mmio_remap/tools/translation_table_tool/bsp.rb
+++ 15_virtual_mem_part3_precomputed_tables/tools/translation_table_tool/bsp.rb
@@ -0,0 +1,49 @@
+# frozen_string_literal: true
+
+# SPDX-License-Identifier: MIT OR Apache-2.0
+#
+# Copyright (c) 2021-2023 Andre Richter <andre.o.richter@gmail.com>
+
+# Raspberry Pi 3 + 4
+class RaspberryPi
+    attr_reader :kernel_granule, :kernel_virt_addr_space_size
+
+    MEMORY_SRC = File.read('kernel/src/bsp/raspberrypi/memory.rs').split("\n")
+
+    def initialize
+        @kernel_granule = Granule64KiB
+
+        @kernel_virt_addr_space_size = KERNEL_ELF.symbol_value('__kernel_virt_addr_space_size')
+
+        @virt_addr_of_kernel_tables = KERNEL_ELF.symbol_value('KERNEL_TABLES')
+        @virt_addr_of_phys_kernel_tables_base_addr = KERNEL_ELF.symbol_value(
+            'PHYS_KERNEL_TABLES_BASE_ADDR'
+        )
+    end
+
+    def phys_addr_of_kernel_tables
+        KERNEL_ELF.virt_to_phys(@virt_addr_of_kernel_tables)
+    end
+
+    def kernel_tables_offset_in_file
+        KERNEL_ELF.virt_addr_to_file_offset(@virt_addr_of_kernel_tables)
+    end
+
+    def phys_kernel_tables_base_addr_offset_in_file
+        KERNEL_ELF.virt_addr_to_file_offset(@virt_addr_of_phys_kernel_tables_base_addr)
+    end
+
+    def phys_addr_space_end_page
+        x = MEMORY_SRC.grep(/pub const END/)
+        x = case BSP_TYPE
+            when :rpi3
+                x[0]
+            when :rpi4
+                x[1]
+            else
+                raise
+            end
+
+        x.scan(/\d+/).join.to_i(16)
+    end
+end

diff -uNr 14_virtual_mem_part2_mmio_remap/tools/translation_table_tool/generic.rb 15_virtual_mem_part3_precomputed_tables/tools/translation_table_tool/generic.rb
--- 14_virtual_mem_part2_mmio_remap/tools/translation_table_tool/generic.rb
+++ 15_virtual_mem_part3_precomputed_tables/tools/translation_table_tool/generic.rb
@@ -0,0 +1,179 @@
+# frozen_string_literal: true
+
+# SPDX-License-Identifier: MIT OR Apache-2.0
+#
+# Copyright (c) 2021-2023 Andre Richter <andre.o.richter@gmail.com>
+
+module Granule64KiB
+    SIZE = 64 * 1024
+    SHIFT = Math.log2(SIZE).to_i
+end
+
+module Granule512MiB
+    SIZE = 512 * 1024 * 1024
+    SHIFT = Math.log2(SIZE).to_i
+    MASK = SIZE - 1
+end
+
+# Monkey-patch Integer with some helper functions.
+class Integer
+    def power_of_two?
+        self[0].zero?
+    end
+
+    def aligned?(alignment)
+        raise unless alignment.power_of_two?
+
+        (self & (alignment - 1)).zero?
+    end
+
+    def align_up(alignment)
+        raise unless alignment.power_of_two?
+
+        (self + alignment - 1) & ~(alignment - 1)
+    end
+
+    def to_hex_underscore(with_leading_zeros: false)
+        fmt = with_leading_zeros ? 'modulo016x' : 'modulox'
+        value = format(fmt, self).to_s.reverse.scan(/.{4}|.+/).join('_').reverse
+
+        format('0xmodulos', value)
+    end
+end
+
+# An array where each value is the start address of a Page.
+class MemoryRegion < Array
+    def initialize(start_addr, size, granule_size)
+        raise unless start_addr.aligned?(granule_size)
+        raise unless size.positive?
+        raise unless (size modulo granule_size).zero?
+
+        num_pages = size / granule_size
+        super(num_pages) do |i|
+            (i * granule_size) + start_addr
+        end
+    end
+end
+
+# Collection of memory attributes.
+class AttributeFields
+    attr_reader :mem_attributes, :acc_perms, :execute_never
+
+    def initialize(mem_attributes, acc_perms, execute_never)
+        @mem_attributes = mem_attributes
+        @acc_perms = acc_perms
+        @execute_never = execute_never
+    end
+
+    def to_s
+        x = case @mem_attributes
+            when :CacheableDRAM
+                'C'
+            else
+                '?'
+            end
+
+        y = case @acc_perms
+            when :ReadWrite
+                'RW'
+            when :ReadOnly
+                'RO'
+            else
+                '??'
+            end
+
+        z = @execute_never ? 'XN' : 'X '
+
+        "#{x} #{y} #{z}"
+    end
+end
+
+# A container that describes a virt-to-phys region mapping.
+class MappingDescriptor
+    @max_section_name_length = 'Sections'.length
+
+    class << self
+        attr_accessor :max_section_name_length
+
+        def update_max_section_name_length(length)
+            @max_section_name_length = [@max_section_name_length, length].max
+        end
+    end
+
+    attr_reader :name, :virt_region, :phys_region, :attributes
+
+    def initialize(name, virt_region, phys_region, attributes)
+        @name = name
+        @virt_region = virt_region
+        @phys_region = phys_region
+        @attributes = attributes
+    end
+
+    def to_s
+        name = @name.ljust(self.class.max_section_name_length)
+        virt_start = @virt_region.first.to_hex_underscore(with_leading_zeros: true)
+        phys_start = @phys_region.first.to_hex_underscore(with_leading_zeros: true)
+        size = ((@virt_region.size * 65_536) / 1024).to_s.rjust(3)
+
+        "#{name} | #{virt_start} | #{phys_start} | #{size} KiB | #{@attributes}"
+    end
+
+    def self.print_divider
+        print '             '
+        print '-' * max_section_name_length
+        puts '--------------------------------------------------------------------'
+    end
+
+    def self.print_header
+        print_divider
+        print '             '
+        print 'Sections'.center(max_section_name_length)
+        print '   '
+        print 'Virt Start Addr'.center(21)
+        print '   '
+        print 'Phys Start Addr'.center(21)
+        print '   '
+        print 'Size'.center(7)
+        print '   '
+        print 'Attr'.center(7)
+        puts
+        print_divider
+    end
+end
+
+def kernel_map_binary
+    mapping_descriptors = KERNEL_ELF.generate_mapping_descriptors
+
+    # Generate_mapping_descriptors updates the header being printed with this call. So it must come
+    # afterwards.
+    MappingDescriptor.print_header
+
+    mapping_descriptors.each do |i|
+        print 'Generating'.rjust(12).green.bold
+        print ' '
+        puts i
+
+        TRANSLATION_TABLES.map_at(i.virt_region, i.phys_region, i.attributes)
+    end
+
+    MappingDescriptor.print_divider
+end
+
+def kernel_patch_tables(kernel_elf_path)
+    print 'Patching'.rjust(12).green.bold
+    print ' Kernel table struct at ELF file offset '
+    puts BSP.kernel_tables_offset_in_file.to_hex_underscore
+
+    File.binwrite(kernel_elf_path, TRANSLATION_TABLES.to_binary, BSP.kernel_tables_offset_in_file)
+end
+
+def kernel_patch_base_addr(kernel_elf_path)
+    print 'Patching'.rjust(12).green.bold
+    print ' Kernel tables physical base address start argument to value '
+    print TRANSLATION_TABLES.phys_tables_base_addr.to_hex_underscore
+    print ' at ELF file offset '
+    puts BSP.phys_kernel_tables_base_addr_offset_in_file.to_hex_underscore
+
+    File.binwrite(kernel_elf_path, TRANSLATION_TABLES.phys_tables_base_addr_binary,
+                  BSP.phys_kernel_tables_base_addr_offset_in_file)
+end

diff -uNr 14_virtual_mem_part2_mmio_remap/tools/translation_table_tool/kernel_elf.rb 15_virtual_mem_part3_precomputed_tables/tools/translation_table_tool/kernel_elf.rb
--- 14_virtual_mem_part2_mmio_remap/tools/translation_table_tool/kernel_elf.rb
+++ 15_virtual_mem_part3_precomputed_tables/tools/translation_table_tool/kernel_elf.rb
@@ -0,0 +1,96 @@
+# frozen_string_literal: true
+
+# SPDX-License-Identifier: MIT OR Apache-2.0
+#
+# Copyright (c) 2021-2023 Andre Richter <andre.o.richter@gmail.com>
+
+# KernelELF
+class KernelELF
+    SECTION_FLAG_ALLOC = 2
+
+    def initialize(kernel_elf_path)
+        @elf = ELFTools::ELFFile.new(File.open(kernel_elf_path))
+        @symtab_section = @elf.section_by_name('.symtab')
+    end
+
+    def machine
+        @elf.machine.to_sym
+    end
+
+    def symbol_value(symbol_name)
+        @symtab_section.symbol_by_name(symbol_name).header.st_value
+    end
+
+    def segment_containing_virt_addr(virt_addr)
+        @elf.each_segments do |segment|
+            return segment if segment.vma_in?(virt_addr)
+        end
+    end
+
+    def virt_to_phys(virt_addr)
+        segment = segment_containing_virt_addr(virt_addr)
+        translation_offset = segment.header.p_vaddr - segment.header.p_paddr
+
+        virt_addr - translation_offset
+    end
+
+    def virt_addr_to_file_offset(virt_addr)
+        segment = segment_containing_virt_addr(virt_addr)
+        segment.vma_to_offset(virt_addr)
+    end
+
+    def sections_in_segment(segment)
+        head = segment.mem_head
+        tail = segment.mem_tail
+
+        sections = @elf.each_sections.select do |section|
+            file_offset = section.header.sh_addr
+            flags = section.header.sh_flags
+
+            file_offset >= head && file_offset < tail && (flags & SECTION_FLAG_ALLOC != 0)
+        end
+
+        sections.map(&:name).join(' ')
+    end
+
+    def select_load_segments
+        @elf.each_segments.select do |segment|
+            segment.instance_of?(ELFTools::Segments::LoadSegment)
+        end
+    end
+
+    def segment_get_acc_perms(segment)
+        if segment.readable? && segment.writable?
+            :ReadWrite
+        elsif segment.readable?
+            :ReadOnly
+        else
+            :Invalid
+        end
+    end
+
+    def update_max_section_name_length(descriptors)
+        MappingDescriptor.update_max_section_name_length(descriptors.map { |i| i.name.size }.max)
+    end
+
+    def generate_mapping_descriptors
+        descriptors = select_load_segments.map do |segment|
+            # Assume each segment is page aligned.
+            size = segment.mem_size.align_up(BSP.kernel_granule::SIZE)
+            virt_start_addr = segment.header.p_vaddr
+            phys_start_addr = segment.header.p_paddr
+            acc_perms = segment_get_acc_perms(segment)
+            execute_never = !segment.executable?
+            section_names = sections_in_segment(segment)
+
+            virt_region = MemoryRegion.new(virt_start_addr, size, BSP.kernel_granule::SIZE)
+            phys_region = MemoryRegion.new(phys_start_addr, size, BSP.kernel_granule::SIZE)
+            attributes = AttributeFields.new(:CacheableDRAM, acc_perms, execute_never)
+
+            MappingDescriptor.new(section_names, virt_region, phys_region, attributes)
+        end
+
+        update_max_section_name_length(descriptors)
+        descriptors
+    end
+end

diff -uNr 14_virtual_mem_part2_mmio_remap/tools/translation_table_tool/main.rb 15_virtual_mem_part3_precomputed_tables/tools/translation_table_tool/main.rb
--- 14_virtual_mem_part2_mmio_remap/tools/translation_table_tool/main.rb
+++ 15_virtual_mem_part3_precomputed_tables/tools/translation_table_tool/main.rb
@@ -0,0 +1,46 @@
+#!/usr/bin/env ruby
+# frozen_string_literal: true
+
+# SPDX-License-Identifier: MIT OR Apache-2.0
+#
+# Copyright (c) 2021-2023 Andre Richter <andre.o.richter@gmail.com>
+
+require 'rubygems'
+require 'bundler/setup'
+require 'colorize'
+require 'elftools'
+
+require_relative 'generic'
+require_relative 'kernel_elf'
+require_relative 'bsp'
+require_relative 'arch'
+
+BSP_TYPE = ARGV[0].to_sym
+kernel_elf_path = ARGV[1]
+
+start = Time.now
+
+KERNEL_ELF = KernelELF.new(kernel_elf_path)
+
+BSP = case BSP_TYPE
+      when :rpi3, :rpi4
+          RaspberryPi.new
+      else
+          raise
+      end
+
+TRANSLATION_TABLES = case KERNEL_ELF.machine
+                     when :AArch64
+                         Arch::ARMv8::TranslationTable.new
+                     else
+                         raise
+                     end
+
+kernel_map_binary
+kernel_patch_tables(kernel_elf_path)
+kernel_patch_base_addr(kernel_elf_path)
+
+elapsed = Time.now - start
+
+print 'Finished'.rjust(12).green.bold
+puts " in #{elapsed.round(2)}s"

```
