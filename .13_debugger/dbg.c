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

#define DISASSEMBLER 1

// array to store register values (see dbg_saveregs in start.S)
unsigned long dbg_regs[37];
// command line
char cmd[256], dbg_running=0;

#if DISASSEMBLER
/**
 * things needed by the disassembler
 */
#include "sprintf.h"
#define NULL ((void*)0)
typedef unsigned long   uint64_t;
typedef unsigned int    uint32_t;
typedef unsigned short  uint16_t;
typedef unsigned char   uint8_t;
// include the Universal Disassembler Library
#include "disasm.h"
#endif

/**
 * Decode exception cause
 */
void dbg_decodeexc(unsigned long type)
{
    unsigned char cause=dbg_regs[33]>>26;

    // print out interruption type
    switch(type) {
        case 0: printf("Synchronous"); break;
        case 1: printf("IRQ"); break;
        case 2: printf("FIQ"); break;
        case 3: printf("SError"); break;
    }
    printf(": ");
    // decode exception type (some, not all. See ARM DDI0487B_b chapter D10.2.28)
    switch(cause) {
        case 0b000000: printf("Unknown"); break;
        case 0b000001: printf("Trapped WFI/WFE"); break;
        case 0b001110: printf("Illegal execution"); break;
        case 0b010101: printf("System call"); break;
        case 0b100000: printf("Instruction abort, lower EL"); break;
        case 0b100001: printf("Instruction abort, same EL"); break;
        case 0b100010: printf("Instruction alignment fault"); break;
        case 0b100100: printf("Data abort, lower EL"); break;
        case 0b100101: printf("Data abort, same EL"); break;
        case 0b100110: printf("Stack alignment fault"); break;
        case 0b101100: printf("Floating point"); break;
        case 0b110000: printf("Breakpoint, lower EL"); break;
        case 0b110001: printf("Breakpoint, same EL"); break;
        case 0b111100: printf("Breakpoint instruction"); break;
        default: printf("Unknown %x", cause); break;
    }
    // decode data abort cause
    if(cause==0b100100 || cause==0b100101) {
        printf(", ");
        switch((dbg_regs[33]>>2)&0x3) {
            case 0: printf("Address size fault"); break;
            case 1: printf("Translation fault"); break;
            case 2: printf("Access flag fault"); break;
            case 3: printf("Permission fault"); break;
        }
        switch(dbg_regs[33]&0x3) {
            case 0: printf(" at level 0"); break;
            case 1: printf(" at level 1"); break;
            case 2: printf(" at level 2"); break;
            case 3: printf(" at level 3"); break;
        }
    }
    printf("\n");
    // if the exception happened in the debugger, we stop to avoid infinite loop
    if(dbg_running) {
        printf("Exception in debugger!\n"
            "  elr_el1: %x  spsr_el1: %x\n  esr_el1: %x  far_el1: %x\nsctlr_el1: %x  tcr_el1: %x\n",
            dbg_regs[31],dbg_regs[32],dbg_regs[33],dbg_regs[34],dbg_regs[35],dbg_regs[36]);
        while(1);
    }
}

/**
 * helper to read a line from user. We redefine some control caracters to handle CSI
 * \e[3~ = 1, delete
 * \e[D  = 2, cursor left
 * \e[C  = 3, cursor right
 */
void dbg_getline()
{
    int i,cmdidx=0,cmdlast=0;
    char c;
    cmd[0]=0;
    // prompt
    printf("\r> ");
    // read until Enter pressed
    while((c=uart_getc())!='\n') {
        // decode CSI key sequences (some, not all)
        if(c==27) {
            c=uart_getc();
            if(c=='[') {
                c=uart_getc();
                if(c=='C') c=3; else    // left
                if(c=='D') c=2; else    // right
                if(c=='3') {
                    c=uart_getc();
                    if(c=='~') c=1;     // delete
                }
            }
        }
        // Backspace
        if(c==8 || c==127) {
            if(cmdidx>0) {
                cmdidx--;
                for(i=cmdidx;i<cmdlast;i++) cmd[i]=cmd[i+1];
                cmdlast--;
            }
        } else
        // Delete
        if(c==1) {
            if(cmdidx<cmdlast) {
                for(i=cmdidx;i<cmdlast;i++) cmd[i]=cmd[i+1];
                cmdlast--;
            }
        } else
        // cursor left
        if(c==2) {
            if(cmdidx>0) cmdidx--;
        } else
        // cursor right
        if(c==3) {
            if(cmdidx<cmdlast) cmdidx++;
        } else {
            // is there a valid character and space to store it?
            if(c<' ' || cmdlast>=sizeof(cmd)-1) {
                continue;
            }
            // if we're not appending, move bytes after cursor
            if(cmdidx<cmdlast) {
                for(i=cmdlast;i>cmdidx;i--)
                    cmd[i]=cmd[i-1];
            }
            cmdlast++;
            cmd[cmdidx++]=c;
        }
        cmd[cmdlast]=0;
        // display prompt and command line, place cursor with CSI code
        printf("\r> %s \r\e[%dC",cmd,cmdidx+2);
    }
    printf("\n");
}

/**
 * helper function to parse the command line for arguments
 */
unsigned long dbg_getoffs(int i)
{
    unsigned long base=0,ret=0;
    int j=0,sign=0;
    // if starts with a register
    if(cmd[i]=='x' || cmd[i]=='r') {
        i++; if(cmd[i]>='0' && cmd[i]<='9') { j=cmd[i]-'0'; }
        i++; if(cmd[i]>='0' && cmd[i]<='9') { j*=10; j+=cmd[i]-'0'; }
        if(j>=0 && j<37) base=dbg_regs[j];
        i++;
        if(cmd[i]=='-') { i++; sign++; }
        if(cmd[i]=='+') i++;
    }
    // offset part
    if(cmd[i]=='0' && cmd[i+1]=='x') {
        i+=2;
        // hex value
        while((cmd[i]>='0'&&cmd[i]<='9')||(cmd[i]>='a'&&cmd[i]<='f')||(cmd[i]>='A'&&cmd[i]<='F')) {
            ret <<= 4;
            if(cmd[i]>='0' && cmd[i]<='9') ret += cmd[i]-'0';
            else if(cmd[i] >= 'a' && cmd[i] <= 'f') ret += cmd[i]-'a'+10;
            else if(cmd[i] >= 'A' && cmd[i] <= 'F') ret += cmd[i]-'A'+10;
            i++;
        }
    } else {
        // decimal value
        while(cmd[i]>='0'&&cmd[i]<='9'){
            ret *= 10;
            ret += cmd[i++]-'0';
        }
    }
    // return base + offset
    return sign? base-ret : base+ret;
}

/**
 * main loop, get and parse commands
 */
void dbg_main()
{
    unsigned long os=0, oe=0, a;
    char c;
#if DISASSEMBLER
    char str[64];
#endif
    int i;

    dbg_running++;

    // main debugger loop
    while(1) {
        // get command from user
        dbg_getline();
        // parse commands
        if(cmd[0]==0 || cmd[0]=='?' || cmd[0]=='h') {
            // print help
            printf("Mini debugger commands:\n"
                "  ?/h\t\tthis help\n"
                "  r\t\tdump registers\n"
                "  x [os [oe]]\texamine memory from offset start (os) to offset end (oe)\n"
                "  i [os [oe]]\tdisassemble instruction from offset start to offset end\n"
                "  c\t\tcontinue execution\n");
            continue;
        } else
        // continue execution
        if(cmd[0]=='c') {
            // move instruction pointer, skip over 'brk'
            asm volatile ("msr elr_el1, %0" : : "r" (dbg_regs[31]+4));
            break;
        } else
        // dump registers
        if(cmd[0]=='r') {
            // general purpose registers x0-x30
            for(i=0;i<31;i++) {
                if(i && i%3==0) printf("\n");
                if(i<10) printf(" ");
                printf("x%d: %16x  ",i,dbg_regs[i]);
            }
            // some system registers
            printf("elr_el1: %x  spsr_el1: %x\n  esr_el1: %x  far_el1: %x\nsctlr_el1: %x  tcr_el1: %x\n",
                dbg_regs[31],dbg_regs[32],dbg_regs[33],dbg_regs[34],dbg_regs[35],dbg_regs[36]);
            continue;
        } else
        // examine or disassemble, commands with arguments
        if(cmd[0]=='x' || cmd[0]=='i') {
            i=1;
            // get first argument
            while(cmd[i]!=0 && cmd[i]!=' ') i++;    // skip command
            while(cmd[i]!=0 && cmd[i]==' ') i++;    // skip separators
            if(cmd[i]!=0) {
                os=oe=dbg_getoffs(i);
                // get second argument
                while(cmd[i]!=0 && cmd[i]!=' ') i++;    // skip 1st arg
                while(cmd[i]!=0 && cmd[i]==' ') i++;    // skip separators
                if(cmd[i]!=0) {
                    oe=dbg_getoffs(i);
                }
            } else {
                // no arguments, use defaults
                if(cmd[0]=='i') {
                    // elr or lr (x30)
                    os=oe=dbg_regs[31]?dbg_regs[31]:dbg_regs[30];
                } else {
                    // sp (x29)
                    os=oe=dbg_regs[29];
                }
            }
            // do the thing
            if(cmd[0]=='i') {
                // must be multiple of 4
                os=os&~3L;
                oe=(oe+3)&~3L;
                if(oe<=os) oe=os+4;
                // disassemble AArch64 bytecode
                while(os<oe) {
                    // print out address and instruction bytecode
                    printf("%8x: %8x",os,*((unsigned int*)os));
#if DISASSEMBLER
                    // disassemble and print out instruction mnemonic
                    os=disasm(os,str);
                    printf("\t%s\n",str);
#else
                    os+=4;
                    printf("\n");
#endif
                }
            } else {
                // dump memory
                if(oe<=os) oe=os+16;
                // for each 16 bytes, do
                for(a=os;a<oe;a+=16) {
                    // print out address
                    printf("%8x: ", a);
                    // hex representation
                    for(i=0;i<16;i++) {
                        printf("%2x%s ",*((unsigned char*)(a+i)),i%4==3?" ":"");
                    }
                    // character representation
                    for(i=0;i<16;i++) {
                        c=*((unsigned char*)(a+i));
                        printf("%c",c<32||c>=127?'.':c);
                    }
                    printf("\n");
                }
            }
            continue;
        } else {
            printf("ERROR: unknown command.\n");
        }
    }
    dbg_running--;
}
