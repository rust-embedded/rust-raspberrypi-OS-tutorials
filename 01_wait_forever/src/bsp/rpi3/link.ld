/* SPDX-License-Identifier: MIT
 *
 * Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>
 */

SECTIONS
{
    /* Set current address to the value from which the RPi3 starts execution */
    . = 0x80000;

    .text :
    {
        *(.text._start) *(.text*)
    }

    /DISCARD/ : { *(.comment*) }
}
