/* SPDX-License-Identifier: MIT OR Apache-2.0
 *
 * Copyright (c) 2022 Andre Richter <andre.o.richter@gmail.com>
 */

SECTIONS
{
    .rodata : {
        ASSERT(. > 0xffffffff00000000, "Expected higher half address")

        KEEP(*(.rodata.symbol_desc*))
        . = ALIGN(8);
        *(.rodata*)
    }
}
