# Tutorial 0E - Global `println!`

Coming soon!

This lesson will teach about:
- Restructuring the current codebase.
- Realizing global println! and print! macros by reusing macros from the Rust
  standard library.
- The NullLock, a wrapper that allows using global static variables without
  explicit need for `unsafe {}` code. It is a teaching concept that is only
  valid in single-threaded IRQ-disabled environments. However, it already lays
  the groundwork for the introduction of proper locking mechanisms, e.g.  real
  Spinlocks.

```console
ferris@box:~$ make raspboot

[0] UART is live!
[1] Press a key to continue booting... Greetings fellow Rustacean!
[2] MMU online.
[i] Kernel memory layout:
      0x00000000 - 0x0007FFFF | 512 KiB | C   RW PXN | Kernel stack
      0x00080000 - 0x00082FFF |  12 KiB | C   RO PX  | Kernel code and RO data
      0x00083000 - 0x0008500F |   8 KiB | C   RW PXN | Kernel data and BSS
      0x3F000000 - 0x3FFFFFFF |  16 MiB | Dev RW PXN | Device MMIO

$>
```
