/* SPDX-License-Identifier: MIT OR Apache-2.0
 *
 * Copyright (c) 2020 Ken Kawamoto <kentaro.kawamoto@gmail.com>
 */

OUTPUT_ARCH("riscv")

ENTRY(_start)

SECTIONS
{
    /* Set current address to the value from which the HiFive1 starts execution */
    . = 0x20400000;

    .text :
    {
        *(.text._start) *(.text*)
    }

    /DISCARD/ : { *(.comment*) }
}
