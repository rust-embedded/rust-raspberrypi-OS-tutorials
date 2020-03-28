/* SPDX-License-Identifier: MIT OR Apache-2.0
 *
 * Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
 */

SECTIONS
{
    /* Set current address to the value from which the RPi starts execution */
    . = 0x80000;

    __ro_start = .;
    .text :
    {
        *(.text._start) *(.text*)
    }

    .rodata :
    {
        *(.rodata*)
    }
    . = ALIGN(65536); /* Fill up to 64 KiB */
    __ro_end = .;

    .data :
    {
        *(.data*)
    }

    /* Section is zeroed in u64 chunks, align start and end to 8 bytes */
    .bss ALIGN(8):
    {
        __bss_start = .;
        *(.bss*);
        . = ALIGN(8);
        __bss_end = .;
    }

    /DISCARD/ : { *(.comment*) }
}
