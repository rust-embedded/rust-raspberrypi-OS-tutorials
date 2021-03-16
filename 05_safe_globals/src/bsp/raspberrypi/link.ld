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
    segment_rw PT_LOAD FLAGS(6); /* 6 == RW */
}

SECTIONS
{
    . =  __rpi_load_addr;

    /***********************************************************************************************
    * Code + RO Data + Global Offset Table
    ***********************************************************************************************/
    __rx_start = .;
    .text :
    {
        KEEP(*(.text._start))
        *(.text*)
    } :segment_rx

    .rodata : ALIGN(8) { *(.rodata*) } :segment_rx
    .got    : ALIGN(8) { *(.got)     } :segment_rx

    /***********************************************************************************************
    * Data + BSS
    ***********************************************************************************************/
    .data : { *(.data*) } :segment_rw

    /* Section is zeroed in u64 chunks, align start and end to 8 bytes */
    .bss : ALIGN(8)
    {
        __bss_start = .;
        *(.bss*);
        . = ALIGN(8);

        . += 8; /* Fill for the bss == 0 case, so that __bss_start <= __bss_end_inclusive holds */
        __bss_end_inclusive = . - 8;
    } :NONE
}
