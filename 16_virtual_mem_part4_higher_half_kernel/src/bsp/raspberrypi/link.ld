/* SPDX-License-Identifier: MIT OR Apache-2.0
 *
 * Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
 */

/* This file provides __kernel_virt_addr_space_size */
INCLUDE src/bsp/raspberrypi/kernel_virt_addr_space_size.ld;

/* The kernel's virtual address range will be:
 *
 * [END_ADDRESS_INCLUSIVE, START_ADDRESS]
 * [u64::MAX             , (u64::MAX - __kernel_virt_addr_space_size) + 1]
 *
 * Since the start address is needed to set the linker address below, calculate it now.
 */
__kernel_virt_start_addr = ((0xffffffffffffffff - __kernel_virt_addr_space_size) + 1);

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
    /* Add the load address as an offset. Makes virt-to-phys translation easier for the human eye */
    . =  __kernel_virt_start_addr + __rpi_load_addr;

    /***********************************************************************************************
    * Code + RO Data + Global Offset Table
    ***********************************************************************************************/
    __rx_start = .;
    .text : AT(__rpi_load_addr)
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

    /* Section is zeroed in pairs of u64. Align start and end to 16 bytes */
    .bss : ALIGN(16)
    {
        __bss_start = .;
        *(.bss*);
        . = ALIGN(16);
        __bss_end_exclusive = .;
    } :NONE

    . = ALIGN(64K); /* Align to page boundary */
    __rw_end_exclusive = .;

    /***********************************************************************************************
    * Guard Page between boot core stack and data
    ***********************************************************************************************/
    . += 64K;

    /***********************************************************************************************
    * Boot Core Stack
    ***********************************************************************************************/
    __boot_core_stack_start = .;         /*   ^             */
                                         /*   | stack       */
    . += 512K;                           /*   | growth      */
                                         /*   | direction   */
    __boot_core_stack_end_exclusive = .; /*   |             */
}
