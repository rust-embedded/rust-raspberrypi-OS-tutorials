# Tutorial 0B - Exception Levels

**This is a stub**

This tutorial covers a very exciting and central feature of the Raspberry Pi's
Cortex-A53 processor: `Exception levels`.

TODO: Write rest of tutorial.

```text
raspi3_boot::setup_and_enter_el1_from_el2::h568f1410ae7cc9b8:
   808c0:	e8 03 1f aa 	mov	x8, xzr
   808c4:	e9 07 00 32 	orr	w9, wzr, #0x3
   808c8:	09 e1 1c d5 	msr	CNTHCTL_EL2, x9
   808cc:	4a 00 80 52 	mov	w10, #0x2
   808d0:	68 e0 1c d5 	msr	CNTVOFF_EL2, x8
   808d4:	08 00 00 90 	adrp	x8, #0x0
   808d8:	0a 00 b0 72 	movk	w10, #0x8000, lsl #16
   808dc:	0a 11 1c d5 	msr	HCR_EL2, x10
   808e0:	ab 78 80 52 	mov	w11, #0x3c5
   808e4:	0b 40 1c d5 	msr	SPSR_EL2, x11
   808e8:	ec 03 0d 32 	orr	w12, wzr, #0x80000
   808ec:	08 31 22 91 	add	x8, x8, #0x88c
   808f0:	28 40 1c d5 	msr	ELR_EL2, x8
   808f4:	0c 41 1c d5 	msr	SP_EL1, x12
   808f8:	e0 03 9f d6 	eret
```
