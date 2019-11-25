/* SPDX-License-Identifier: MIT OR Apache-2.0
 *
 * Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>
 */

SECTIONS
{
    /* Set the link address to the top-most 40 KiB of DRAM (assuming 1GiB) */
    . = 0x3F000000 - 0x10000;

    __binary_start = .;
    .text :
    {
        *(.text._start) *(.text*)
    }

    .rodata :
    {
        *(.rodata*)
    }

    .data :
    {
        *(.data*)
    }

    /* Align to 8 byte boundary */
    .bss ALIGN(8):
    {
        __bss_start = .;
        *(.bss*);
        __bss_end = .;
    }

    .got :
    {
        *(.got*)
    }

    /* Fill up to 8 byte, b/c relocating the binary is done in u64 chunks */
    . = ALIGN(8);
    __binary_end = .;

    /DISCARD/ : { *(.comment*) }
}
