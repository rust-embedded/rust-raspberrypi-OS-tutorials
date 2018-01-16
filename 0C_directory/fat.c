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

#include "sd.h"
#include "uart.h"

// get the end of bss segment from linker
extern unsigned char _end;
unsigned int partitionlba;

// the BIOS Parameter Block (in Volume Boot Record)
typedef struct {
    char            jmp[3];
    char            oem[8];
    unsigned short  bps;
    unsigned char   spc;
    unsigned short  rsc;
    unsigned char   nf;
    unsigned short  nr;
    unsigned short  ts16;
    unsigned char   media;
    unsigned short  spf16;
    unsigned short  spt;
    unsigned short  nh;
    unsigned int    hs;
    unsigned int    ts32;
    unsigned int    spf32;
    unsigned int    flg;
    unsigned int    rc;
    char            vol[6];
    char            fst[8];
    char            dmy[20];
    char            fst2[8];
} __attribute__((packed)) bpb_t;

// directory entry structure
typedef struct {
    char            name[8];
    char            ext[3];
    char            attr[9];
    unsigned short  ch;
    unsigned int    attr2;
    unsigned short  cl;
    unsigned int    size;
} __attribute__((packed)) fatdir_t;

/**
 * Get the starting LBA address of the first partition
 * so that we know where our FAT file system starts, and
 * read that volume's BIOS Parameter Block
 */
int fat_getpartition()
{
    unsigned char *mbr=&_end;
    bpb_t *bpb=(bpb_t*)&_end;
    // read the partitioning table
    if(sd_readblock(0,&_end,1)) {
        // check magic
        if(mbr[510]!=0x55 || mbr[511]!=0xAA) {
            uart_puts("ERROR: Bad magic in MBR\n");
            return 0;
        }
        // check partition type
        if(mbr[0x1C2]!=0xE/*FAT16 LBA*/ && mbr[0x1C2]!=0xC/*FAT32 LBA*/) {
            uart_puts("ERROR: Wrong partition type\n");
            return 0;
        }
        uart_puts("MBR disk identifier: ");
        uart_hex(*((unsigned int*)((unsigned long)&_end+0x1B8)));
        uart_puts("\nFAT partition starts at: ");
        partitionlba=*((unsigned int*)((unsigned long)&_end+0x1C6));
        uart_hex(partitionlba);
        uart_puts("\n");
        // read the boot record
        if(!sd_readblock(partitionlba,&_end,1)) {
            uart_puts("ERROR: Unable to read boot record\n");
            return 0;
        }
        // check file system type. We don't use cluster numbers for that, but magic bytes
        if( !(bpb->fst[0]=='F' && bpb->fst[1]=='A' && bpb->fst[2]=='T') &&
            !(bpb->fst2[0]=='F' && bpb->fst2[1]=='A' && bpb->fst2[2]=='T')) {
            uart_puts("ERROR: Unknown file system type\n");
            return 0;
        }
        uart_puts("FAT type: ");
        // if 16 bit sector per fat is zero, then it's a FAT32
        uart_puts(bpb->spf16>0?"FAT16":"FAT32");
        uart_puts("\n");
        return 1;
    }
    return 0;
}

/**
 * List root directory entries in a FAT file system
 */
void fat_listdirectory()
{
    bpb_t *bpb=(bpb_t*)&_end;
    fatdir_t *dir=(fatdir_t*)&_end;
    unsigned int root_sec, s;
    // find the root directory's LBA
    root_sec=((bpb->spf16?bpb->spf16:bpb->spf32)*bpb->nf)+bpb->rsc;
    //WARNING gcc generates bad code for bpb->nr, causing unaligned exception
    s=*((unsigned int*)&bpb->nf);
    s>>=8;
    s&=0xFFFF;
    uart_puts("FAT number of root diretory entries: ");
    uart_hex(s);
    s<<=5;
    // now s=bpb->nr*sizeof(fatdir_t));
    if(bpb->spf16==0) {
        // adjust for FAT32
        root_sec+=(bpb->rc-2)*bpb->spc;
    }
    // add partition LBA
    root_sec+=partitionlba;
    uart_puts("\nFAT root directory LBA: ");
    uart_hex(root_sec);
    uart_puts("\n");
    // load the root directory
    if(sd_readblock(root_sec,(unsigned char*)&_end,s/512+1)) {
        uart_puts("\nAttrib Cluster  Size     Name\n");
        // iterate on each entry and print out
        for(;dir->name[0]!=0;dir++) {
            // is it a valid entry?
            if(dir->name[0]==0xE5 || dir->attr[0]==0xF) continue;
            // decode attributes
            uart_send(dir->attr[0]& 1?'R':'.');  // read-only
            uart_send(dir->attr[0]& 2?'H':'.');  // hidden
            uart_send(dir->attr[0]& 4?'S':'.');  // system
            uart_send(dir->attr[0]& 8?'L':'.');  // volume label
            uart_send(dir->attr[0]&16?'D':'.');  // directory
            uart_send(dir->attr[0]&32?'A':'.');  // archive
            uart_send(' ');
            // staring cluster
            uart_hex(((unsigned int)dir->ch)<<16|dir->cl);
            uart_send(' ');
            // size
            uart_hex(dir->size);
            uart_send(' ');
            // filename
            dir->attr[0]=0;
            uart_puts(dir->name);
            uart_puts("\n");
        }
    } else {
        uart_puts("ERROR: Unable to load root directory\n");
    }
}
