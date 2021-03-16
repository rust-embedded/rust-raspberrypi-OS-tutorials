/* SPDX-License-Identifier: MIT OR Apache-2.0
 *
 * Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
 */

/* The address at which the the kernel binary will be loaded by the Raspberry's firmware */
__rpi_load_addr = 0x80000;

ENTRY(__rpi_load_addr)

PHDRS
{
    segment_rx PT_LOAD FLAGS(5); /* 5 == RX */
}

SECTIONS
{
    . =  __rpi_load_addr;

    /***********************************************************************************************
    * Code
    ***********************************************************************************************/
    .text :
    {
        KEEP(*(.text._start))
        *(.text*)
    } :segment_rx
}
