# Tutorial 0F - DMA Memory

Coming soon!

This lesson will teach about:
- Simple bump memory allocators and non-cacheable memory.
- Using MiniUart for early boot messages and dynamically switching to the PL011
  Uart later (which now needs the memory allocator that theoretically could fail
  - which the MiniUart could then print).


## Output

```console
ferris@box:~$ make raspboot

[0] MiniUart online.
[1] Press a key to continue booting... Greetings fellow Rustacean!
[2] MMU online.
[i] Kernel memory layout:
      0x00000000 - 0x0007FFFF | 512 KiB | C   RW PXN | Kernel stack
      0x00080000 - 0x00083FFF |  16 KiB | C   RO PX  | Kernel code and RO data
      0x00084000 - 0x0008700F |  12 KiB | C   RW PXN | Kernel data and BSS
      0x00200000 - 0x005FFFFF |   4 MiB | NC  RW PXN | DMA heap pool
      0x3F000000 - 0x3FFFFFFF |  16 MiB | Dev RW PXN | Device MMIO
[i] Global DMA Allocator:
      Allocated Addr 0x00200000 Size 0x90
[3] Videocore Mailbox set up (DMA mem heap allocation successful).
[4] PL011 UART online. Output switched to it.

$>
```
