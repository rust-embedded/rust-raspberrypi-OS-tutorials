/*
 * Copyright (C) 2018 bzt (bztsrc@github)
 * Copyright (c) 2018 Andre Richter <andre.o.richter@gmail.com>
 *
 * Permission is hereby granted, free of charge, to any person
 * obtaining a copy of this software and associated documentation
 * files (the "Software"), to deal in the Software without
 * restriction, including without limitation the rights to use, copy,
 * modify, merge, publish, distribute, sublicense, and/or sell copies
 * of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be
 * included in all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
 * EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
 * MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
 * NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT
 * HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
 * WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
 * DEALINGS IN THE SOFTWARE.
 *
 */

.section ".text.boot"

.global _boot_cores

_boot_cores:
    // Save arguments in registers (we will need them later for the new kernel)
    // We choose x25-x28 because instructions generated from Rust by rustc do not
    // touch them. You can check that with
    // "./dockcross-linux-aarch64 aarch64-linux-gnu-objdump -d kernel8"
    mov     x25, x0
    mov     x26, x1
    mov     x27, x2
    mov     x28, x3

    // read cpu id, stop slave cores
    mrs     x1, mpidr_el1
    and     x1, x1, #3
    cbz     x1, 2f
    // cpu id > 0, stop
1:  wfe
    b       1b
2:  // cpu id == 0

    // relocate our code from load address to link address
    ldr     x1, =0x80000
    ldr     x2, =_boot_cores
    ldr     w3, =__loader_size
3:  ldr     x4, [x1], #8
    str     x4, [x2], #8
    sub     w3, w3, #1
    cbnz    w3, 3b

    // set stack before our code
    ldr     x1, =_boot_cores
    mov     sp, x1

    // jump to relocated Rust code, should not return
    bl      reset-2048
    // for failsafe, halt this core too
    b       1b
