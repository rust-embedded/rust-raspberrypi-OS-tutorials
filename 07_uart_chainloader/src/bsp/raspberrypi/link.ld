/* SPDX-License-Identifier: MIT OR Apache-2.0
 *
 * Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>
 */

SECTIONS
{
    /* Set the link address to 32 MiB */
    . = 0x2000000;

    __binary_start = .;
    .text :
    {
        *(.text._start)
        KEEP(*(.text.runtime_init))
        *(.text*);
    }

    .rodata :
    {
        *(.rodata*)
    }

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

        /* Fill for the bss == 0 case, so that __bss_start <= __bss_end_inclusive holds */
        . += 8;
        __bss_end_inclusive = . - 8;
    }

    .got :
    {
        *(.got*)
    }

    /* Fill up to 8 byte, b/c relocating the binary is done in u64 chunks */
    . = ALIGN(8);
    __binary_end_inclusive = . - 8;

    __runtime_init_reloc = runtime_init;

    /DISCARD/ : { *(.comment*) }
}
