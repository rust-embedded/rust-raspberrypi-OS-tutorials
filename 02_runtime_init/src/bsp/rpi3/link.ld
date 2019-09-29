/* SPDX-License-Identifier: MIT
 *
 * Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>
 */

ENTRY(start)

SECTIONS
{
    /* Set current address to the value from which the RPi3 starts execution */
    . = 0x80000;

    .text :
    {
        *(.text.start) *(.text*)
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

    /DISCARD/ : { *(.comment*) }
}
