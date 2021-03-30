/* SPDX-License-Identifier: MIT OR Apache-2.0
 *
 * Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
 */

/* This file provides __kernel_virt_addr_space_size */
INCLUDE src/bsp/raspberrypi/kernel_virt_addr_space_size.ld;

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
        *(.text._start_arguments) /* Constants (or statics in Rust speak) read by _start(). */
        *(.text._start_rust)      /* The Rust entry point */
        *(.text*)                 /* Everything else */
    } :segment_rx

    .rodata : ALIGN(8) { *(.rodata*) } :segment_rx
    .got    : ALIGN(8) { *(.got)     } :segment_rx

    . = ALIGN(64K); /* Align to page boundary */
    __rx_end_exclusive = .;

    /***********************************************************************************************
    * Data + BSS
    ***********************************************************************************************/
    __rw_start = .;
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

    . = ALIGN(64K); /* Align to page boundary */
    __rw_end_exclusive = .;

    /***********************************************************************************************
    * Guard Page between boot core stack and data
    ***********************************************************************************************/
    __boot_core_stack_guard_page_start = .;
    . += 64K;
    __boot_core_stack_guard_page_end_exclusive = .;

    /***********************************************************************************************
    * Boot Core Stack
    ***********************************************************************************************/
    __boot_core_stack_start = .;         /*   ^             */
                                         /*   | stack       */
    . += 512K;                           /*   | growth      */
                                         /*   | direction   */
    __boot_core_stack_end_exclusive = .; /*   |             */
}
