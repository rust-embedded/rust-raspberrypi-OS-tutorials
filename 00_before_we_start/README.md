# Before we start

The following text is a 1:1 copy of the documentation that can be found at the top of the kernel's
main source code file in each tutorial. It describes the general structure of the source code, and
tries to convey the philosophy behind the respective approach. Please read it to make yourself
familiar with what you will encounter during the tutorials. It will help you to navigate the code
better and understand the differences and additions between the separate tutorials.

Please also note that the following text will reference source code files (e.g. `**/memory.rs`) or
functions that won't exist yet in the first bunch of the tutorials. They will be added gradually as
the tutorials advance.

Have fun!

# Code organization and architecture

The code is divided into different *modules*, each representing a typical **subsystem** of the
`kernel`. Top-level module files of subsystems reside directly in the `src` folder. For example,
`src/memory.rs` contains code that is concerned with all things memory management.

## Visibility of processor architecture code

Some of the `kernel`'s subsystems depend on low-level code that is specific to the target processor
architecture. For each supported processor architecture, there exists a subfolder in `src/_arch`,
for example, `src/_arch/aarch64`.

The architecture folders mirror the subsystem modules laid out in `src`. For example, architectural
code that belongs to the `kernel`'s MMU subsystem (`src/memory/mmu.rs`) would go into
`src/_arch/aarch64/memory/mmu.rs`. The latter file is loaded as a module in `src/memory/mmu.rs`
using the `path attribute`. Usually, the chosen module name is the generic module's name prefixed
with `arch_`.

For example, this is the top of `src/memory/mmu.rs`:

```
#[cfg(target_arch = "aarch64")]
#[path = "../_arch/aarch64/memory/mmu.rs"]
mod arch_mmu;
```

Often times, items from the `arch_ module` will be publicly reexported by the parent module. This
way, each architecture specific module can provide its implementation of an item, while the caller
must not be concerned which architecture has been conditionally compiled.

## BSP code

`BSP` stands for Board Support Package. `BSP` code is organized under `src/bsp.rs` and contains
target board specific definitions and functions. These are things such as the board's memory map or
instances of drivers for devices that are featured on the respective board.

Just like processor architecture code, the `BSP` code's module structure tries to mirror the
`kernel`'s subsystem modules, but there is no reexporting this time. That means whatever is provided
must be called starting from the `bsp` namespace, e.g. `bsp::driver::driver_manager()`.

## Kernel interfaces

Both `arch` and `bsp` contain code that is conditionally compiled depending on the actual target and
board for which the kernel is compiled. For example, the `interrupt controller` hardware of the
`Raspberry Pi 3` and the `Raspberry Pi 4` is different, but we want the rest of the `kernel` code to
play nicely with any of the two without much hassle.

In order to provide a clean abstraction between `arch`, `bsp` and `generic kernel code`, `interface`
traits are provided *whenever possible* and *where it makes sense*. They are defined in the
respective subsystem module and help to enforce the idiom of *program to an interface, not an
implementation*. For example, there will be a common IRQ handling interface which the two different
interrupt controller `drivers` of both Raspberrys will implement, and only export the interface to
the rest of the `kernel`.

```
        +-------------------+
        | Interface (Trait) |
        |                   |
        +--+-------------+--+
           ^             ^
           |             |
           |             |
+----------+--+       +--+----------+
| kernel code |       |  bsp code   |
|             |       |  arch code  |
+-------------+       +-------------+
```

# Summary

For a logical `kernel` subsystem, corresponding code can be distributed over several physical
locations. Here is an example for the **memory** subsystem:

- `src/memory.rs` and `src/memory/**/*`
  - Common code that is agnostic of target processor architecture and `BSP` characteristics.
    - Example: A function to zero a chunk of memory.
  - Interfaces for the memory subsystem that are implemented by `arch` or `BSP` code.
    - Example: An `MMU` interface that defines `MMU` function prototypes.
- `src/bsp/__board_name__/memory.rs` and `src/bsp/__board_name__/memory/**/*`
  - `BSP` specific code.
  - Example: The board's memory map (physical addresses of DRAM and MMIO devices).
- `src/_arch/__arch_name__/memory.rs` and `src/_arch/__arch_name__/memory/**/*`
  - Processor architecture specific code.
  - Example: Implementation of the `MMU` interface for the `__arch_name__` processor
    architecture.

From a namespace perspective, **memory** subsystem code lives in:

- `crate::memory::*`
- `crate::bsp::memory::*`

# Boot flow

1. The kernel's entry point is the function `cpu::boot::arch_boot::_start()`.
    - It is implemented in `src/_arch/__arch_name__/cpu/boot.s`.


