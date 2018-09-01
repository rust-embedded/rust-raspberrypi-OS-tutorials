# Tutorial 0B - Exception Levels

**This is a stub**

This tutorial covers a very exciting and central feature of the Raspberry Pi's
Cortex-A53 processor: `Exception levels`.

TODO: Write rest of tutorial.

```text
raspi3_boot::setup_and_enter_el1_from_el2::h0641a5a5302db706:
   80954:       e8 03 1f aa     mov     x8, xzr
   80958:       e9 07 00 32     orr     w9, wzr, #0x3
   8095c:       4a 00 80 52     mov     w10, #0x2
   80960:       0a 00 b0 72     movk    w10, #0x8000, lsl #16
   80964:       09 e1 1c d5     msr     CNTHCTL_EL2, x9
   80968:       68 e0 1c d5     msr     CNTVOFF_EL2, x8
   8096c:       08 00 00 90     adrp    x8, #0x0
   80970:       8b 78 80 52     mov     w11, #0x3c4
   80974:       0a 11 1c d5     msr     HCR_EL2, x10
   80978:       ec 03 0d 32     orr     w12, wzr, #0x80000
   8097c:       08 81 24 91     add     x8, x8, #0x920
   80980:       0b 40 1c d5     msr     SPSR_EL2, x11
   80984:       28 40 1c d5     msr     ELR_EL2, x8
   80988:       0c 41 18 d5     msr     SP_EL0, x12
   8098c:       e0 03 9f d6     eret
```
