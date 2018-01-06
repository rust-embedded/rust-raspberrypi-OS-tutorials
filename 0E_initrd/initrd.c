/*
 * Copyright (C) 2018 bzt (bztsrc@github)
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

#include "uart.h"

/**
 * Helper function to convert ASCII octal number into binary
 * s string
 * n number of digits
 */
int oct2bin(char *s, int n)
{
    int r=0;
    while(n-->0) {
        r<<=3;
        r+=*s++-'0';
    }
    return r;
}

/**
 * List the contents of a tar archive
 */
void initrd_list(char *buf)
{
    char *types[]={"regular", "link  ", "symlnk", "chrdev", "blkdev", "dircty", "fifo  ", "???   "};

    uart_puts("Type   Offset   Size     Access rights\tFilename\n");
    // iterate on archive's contents
    while(!__builtin_memcmp(buf+257,"ustar",5)){
        int fs=oct2bin(buf+0x7c,11);
        // print out meta information
        uart_puts(types[buf[0x9c]-'0']);
        uart_send(' ');
        uart_hex((unsigned int)((unsigned long)buf)+512);
        uart_send(' ');
        uart_hex(fs);           // file size in hex
        uart_send(' ');
        uart_puts(buf+0x64);    // access bits in octal
        uart_send(' ');
        uart_puts(buf+0x109);   // owner
        uart_send('.');
        uart_puts(buf+0x129);   // group
        uart_send('\t');
        uart_puts(buf);         // filename
        if(buf[0x9c]=='2') {
            uart_puts(" -> ");  // symlink target
            uart_puts(buf+0x9d);
        }
        uart_puts("\n");
        // jump to the next file
        buf+=(((fs+511)/512)+1)*512;
    }
}
