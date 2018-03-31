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

/* POSIX ustar header format */
typedef struct {                /* byte offset */
  char name[100];               /*   0 */
  char mode[8];                 /* 100 */
  char uid[8];                  /* 108 */
  char gid[8];                  /* 116 */
  char size[12];                /* 124 */
  char mtime[12];               /* 136 */
  char chksum[8];               /* 148 */
  char typeflag;                /* 156 */
  char linkname[100];           /* 157 */
  char magic[6];                /* 257 */
  char version[2];              /* 263 */
  char uname[32];               /* 265 */
  char gname[32];               /* 297 */
  char devmajor[8];             /* 329 */
  char devminor[8];             /* 337 */
  char prefix[167];             /* 345 */
} __attribute__((packed)) tar_t;

/* cpio hpodc format */
typedef struct {
    char magic[6];      /* Magic header '070707'. */
    char dev[6];        /* device number. */
    char ino[6];        /* "i-node" number. */
    char mode[6];       /* Permisions. */
    char uid[6];        /* User ID. */
    char gid[6];        /* Group ID. */
    char nlink[6];      /* Number of hard links. */
    char rdev[6];       /* device major/minor number. */
    char mtime[11];     /* Modification time. */
    char namesize[6];   /* Length of filename in bytes. */
    char filesize[11];  /* File size. */
} __attribute__((packed)) cpio_t;

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
 * List the contents of an archive
 */
void initrd_list(char *buf)
{
    char *types[]={"regular", "link  ", "symlnk", "chrdev", "blkdev", "dircty", "fifo  ", "???   "};

    uart_puts("Type     Offset   Size     Access rights\tFilename\n");

    // iterate on archive's contents
    while(!__builtin_memcmp(buf+257,"ustar",5)) {
        // if it's an ustar archive
        tar_t *header=(tar_t*)buf;
        int fs=oct2bin(header->size,11);
        // print out meta information
        uart_puts(types[header->typeflag-'0']);
        uart_send(' ');
        uart_send(' ');
        uart_hex((unsigned int)((unsigned long)buf)+sizeof(tar_t));
        uart_send(' ');
        uart_hex(fs);               // file size in hex
        uart_send(' ');
        uart_puts(header->mode);    // access bits in octal
        uart_send(' ');
        uart_puts(header->uname);   // owner
        uart_send('.');
        uart_puts(header->gname);   // group
        uart_send('\t');
        uart_puts(buf);             // filename
        if(header->typeflag=='2') {
            uart_puts(" -> ");      // symlink target
            uart_puts(header->linkname);
        }
        uart_puts("\n");
        // jump to the next file
        buf+=(((fs+511)/512)+1)*512;
    }
    // if it's a cpio archive. Cpio also has a trailer entry
    while(!__builtin_memcmp(buf,"070707",6) && __builtin_memcmp(buf+sizeof(cpio_t),"TRAILER!!",9)) {
        cpio_t *header = (cpio_t*)buf;
        int ns=oct2bin(header->namesize,6);
        int fs=oct2bin(header->filesize,11);
        // print out meta information
        uart_hex(oct2bin(header->mode,6));  // mode (access rights + type)
        uart_send(' ');
        uart_hex((unsigned int)((unsigned long)buf)+sizeof(cpio_t)+ns);
        uart_send(' ');
        uart_hex(fs);                       // file size in hex
        uart_send(' ');
        uart_hex(oct2bin(header->uid,6));   // user id in hex
        uart_send('.');
        uart_hex(oct2bin(header->gid,6));   // group id in hex
        uart_send('\t');
        uart_puts(buf+sizeof(cpio_t));      // filename
        uart_puts("\n");
        // jump to the next file
        buf+=(sizeof(cpio_t)+ns+fs);
    }
}
