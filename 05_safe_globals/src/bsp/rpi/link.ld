/* SPDX-License-Identifier: MIT OR Apache-2.0
 *
 * Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>
 */

SECTIONS
{
    /* Set current address to the value from which the RPi starts execution */
    . = 0x80000;

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

    /DISCARD/ : { *(.comment*) }
}
