/* Universal Disassembler Function Library
 * https://github.com/bztsrc/udisasm
 *
 *            ----- GENERATED FILE, DO NOT EDIT! -----
 *
 * Copyright (C) 2017 bzt (bztsrc@github)
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
 * @brief Disassembler source generated from aarch64.txt
 */

enum { disasm_arg_NONE,disasm_arg_ofs,disasm_arg_ofe, disasm_arg_Xt, disasm_arg_labelij1, disasm_arg_RtS, disasm_arg_RnS, disasm_arg_i, disasm_arg_j12_opt, disasm_arg_Rn, disasm_arg_Rt, disasm_arg_j16_opt, disasm_arg_j, disasm_arg_Rm, disasm_arg_c, disasm_arg_labeli4, disasm_arg_i_opt, disasm_arg_pstate, disasm_arg_sh, disasm_arg_a0, disasm_arg_a1, disasm_arg_a2, disasm_arg_dc0, disasm_arg_dc1, disasm_arg_ZVA, disasm_arg_dc2, disasm_arg_ic, disasm_arg_Xt_opt, disasm_arg_tl0, disasm_arg_tl1, disasm_arg_tl2, disasm_arg_sysreg, disasm_arg_Cn, disasm_arg_Cm, disasm_arg_Xn, disasm_arg_b, disasm_arg_VtT, disasm_arg_Vt2T, disasm_arg_Vt3T, disasm_arg_Vt4T, disasm_arg_offs, disasm_arg_XnS, disasm_arg_offe, disasm_arg_Qi, disasm_arg_Xm, disasm_arg_Qi3, disasm_arg_Qi2, disasm_arg_Qi1, disasm_arg_VtB, disasm_arg_VtH, disasm_arg_VtS, disasm_arg_VtD, disasm_arg_i1, disasm_arg_i2, disasm_arg_i4, disasm_arg_i8, disasm_arg_Vt3B, disasm_arg_Vt3H, disasm_arg_Vt3S, disasm_arg_Vt3D, disasm_arg_i3, disasm_arg_i6, disasm_arg_i12, disasm_arg_i24, disasm_arg_Vt2B, disasm_arg_Vt2H, disasm_arg_Vt2S, disasm_arg_Vt2D, disasm_arg_i16, disasm_arg_Vt4B, disasm_arg_Vt4H, disasm_arg_Vt4S, disasm_arg_Vt4D, disasm_arg_i32, disasm_arg_z, disasm_arg_z3, disasm_arg_z2, disasm_arg_z4, disasm_arg_Rd, disasm_arg_Rd1, disasm_arg_Rt1, disasm_arg_Wd, disasm_arg_Wt, disasm_arg_FPt, disasm_arg_prf_op, disasm_arg_is4_opt, disasm_arg_FPm, disasm_arg_iz4_opt, disasm_arg_im4_opt, disasm_arg_nRt, disasm_arg_FPst, disasm_arg_j_opt, disasm_arg_Rom, disasm_arg_amountj, disasm_arg_amountz, disasm_arg_amountjs, disasm_arg_amountj2, disasm_arg_amountj3, disasm_arg_shiftj_opt, disasm_arg_Rsom, disasm_arg_exts, disasm_arg_Wn, disasm_arg_Wm, disasm_arg_Xd, disasm_arg_Vt16b, disasm_arg_Vn16b, disasm_arg_Qt, disasm_arg_Sn, disasm_arg_Vm4s, disasm_arg_Vt4s, disasm_arg_Vn4s, disasm_arg_Qn, disasm_arg_St, disasm_arg_FPjt, disasm_arg_Vnj, disasm_arg_FPidx, disasm_arg_Vtjq, disasm_arg_Ht, disasm_arg_Hn, disasm_arg_Hm, disasm_arg_FPn, disasm_arg_VtH1, disasm_arg_VnH1, disasm_arg_VmH1, disasm_arg_Vtzq, disasm_arg_Vnzq, disasm_arg_Vmzq, disasm_arg_simd0, disasm_arg_FPz2t, disasm_arg_FPz2n, disasm_arg_FPz2m, disasm_arg_VnT, disasm_arg_VmT, disasm_arg_FPz3t, disasm_arg_FPz3n, disasm_arg_FPz4n, disasm_arg_VnT3, disasm_arg_Vn2d, disasm_arg_Vn2h, disasm_arg_Vnz, disasm_arg_FPz4t, disasm_arg_Vtz, disasm_arg_FPz3m, disasm_arg_Dt, disasm_arg_Dn, disasm_arg_shrshift, disasm_arg_Vtj2, disasm_arg_Vnj2, disasm_arg_shlshift, disasm_arg_FPnj, disasm_arg_VnTa, disasm_arg_FPjt2, disasm_arg_FPjn2, disasm_arg_Vtz3, disasm_arg_VmTs, disasm_arg_VmHs, disasm_arg_VmTs2, disasm_arg_Vn116b, disasm_arg_Vn216b, disasm_arg_Vn316b, disasm_arg_Vn416b, disasm_arg_Vtj, disasm_arg_R2n, disasm_arg_FPidxk, disasm_arg_Vtzq2, disasm_arg_VnT2, disasm_arg_Vnz3, disasm_arg_Vnzq2, disasm_arg_shift8, disasm_arg_VtT3, disasm_arg_VmT3, disasm_arg_VtT4, disasm_arg_imm8, disasm_arg_amountk_opt, disasm_arg_amountk2_opt, disasm_arg_imm64, disasm_arg_Vt2d, disasm_arg_F16, disasm_arg_F32, disasm_arg_F64, disasm_arg_VmTs4b, disasm_arg_Vm2d, disasm_arg_Vm16b, disasm_arg_Vd16b, disasm_arg_Vd4s, disasm_arg_FPz5t, disasm_arg_fbits, disasm_arg_FPz5n, disasm_arg_Vn1d, disasm_arg_Vt1d, disasm_arg_FPk5t, disasm_arg_FPz5m, disasm_arg_jz, disasm_arg_FPz5d };

/*** private functions ***/
char *disasm_str(char*s,int n) {if(!s)return "?";while(n){s++;if(!*s){s++;n--;}}return *s?s:"?";};
char *disasm_sysreg(uint8_t p,uint8_t k,uint8_t n,uint8_t m,uint8_t j) {char *t=NULL;switch(p){case 2: switch(k) {case 0: switch(n) {case 0: switch(m) {case 0: t="?\0?\0OSDTRRX_EL1\0"; break;case 2: t="MDCCINT_EL1\0?\0MDSCR_EL1\0"; break;case 3: t="?\0?\0OSDTRTX_EL1\0"; break;case 6: t="?\0?\0OSECCR_EL1\0"; break;default: { n=j; j=m; switch(n) {case 4: t="DBGBVR0_EL1\0DBGBVR1_EL1\0DBGBVR2_EL1\0DBGBVR3_EL1\0DBGBVR4_EL1\0DBGBVR5_EL1\0DBGBVR6_EL1\0DBGBVR7_EL1\0"; break;case 5: t="DBGBCR0_EL1\0DBGBCR1_EL1\0DBGBCR2_EL1\0DBGBCR3_EL1\0DBGBCR4_EL1\0DBGBCR5_EL1\0DBGBCR6_EL1\0DBGBCR7_EL1\0"; break;case 6: t="DBGWVR0_EL1\0DBGWVR1_EL1\0DBGWVR2_EL1\0DBGWVR3_EL1\0DBGWVR4_EL1\0DBGWVR5_EL1\0DBGWVR6_EL1\0DBGWVR7_EL1\0"; break;case 7: t="DBGWCR0_EL1\0DBGWCR1_EL1\0DBGWCR2_EL1\0DBGWCR3_EL1\0DBGWCR4_EL1\0DBGWCR5_EL1\0DBGWCR6_EL1\0DBGWCR7_EL1\0"; break;} break; }} break;case 1: if(m==0) t="MDRAR_EL1\0?\0?\0?\0OSLAR_EL1\0"; else if(j==4) { j=m; t="OSLSR_EL1\0?\0OSDLR_EL1\0DBGPRCR_EL1\0"; }break;case 7: if(j==6) { j=m; t="?\0?\0?\0?\0?\0?\0?\0?\0DBGCLAIMSET_EL1\0DBGCLAIMCLR_EL1\0?\0?\0?\0?\0DBGAUTHSTATUS_EL1\0"; }break;} break;case 3: if(n==0&&j==0) { j=m; t="?\0MDCCSR_EL0\0?\0?\0DBGDTR_EL0\0DBGDTRRX_EL0\0"; } break;case 4: if(n==0&&m==7) t="DBGVCR32_EL2\0"; break;} break;case 3: switch(k) {case 0: switch(n) {case 0: if(m==0) t="MIDR_EL1\0?\0?\0?\0?\0MPIDR_EL1\0REVIDR_EL1\0?\0ID_PFR0_EL1\0ID_PFR1_EL1\0ID_DFR0_EL1\0ID_AFR0_EL1\0ID_MMFR0_EL1\0ID_MMFR1_EL1\0ID_MMFR2_EL1\0ID_MMFR3_EL1\0ID_ISAR0_EL1\0ID_ISAR1_EL1\0ID_ISAR2_EL1\0ID_ISAR2_EL1\0ID_ISAR3_EL1\0ID_ISAR4_EL1\0ID_ISAR5_EL1\0ID_MMFR4_EL1\0?\0MVFR0_EL1\0MVFR1_EL1\0MVFR2_EL1\0?\0?\0?\0?\0?\0ID_AA64PFR0_EL1\0ID_AA64PFR1_EL1\0?\0?\0ID_AA64ZFR0_EL1\0?\0?\0?\0ID_AA64DFR0_EL1\0ID_AA64DFR1_EL1\0?\0?\0ID_AA64AFR0_EL1\0ID_AA64AFR1_EL1\0?\0?\0ID_AA64ISAR0_EL1\0ID_AA64ISAR1_EL1\0?\0?\0?\0?\0?\0?\0ID_AA64MMFR0_EL1\0ID_AA64MMFR1_EL1\0ID_AA64MMFR2_EL1\0"; break;case 1: switch(m) {case 0: t="SCTLR_EL1\0ACTLR_EL1\0CPACR_EL1\0"; break;case 2: t="ZCR_EL1\0"; break;} break;case 2: if(m==0) t="TTBR0_EL1\0TTBR1_EL1\0TCR_EL1\0"; break;case 4: switch(m) {case 0: t="SPSR_EL1\0ELR_EL1\0"; break;case 1: t="SP_EL0\0"; break;case 2: t="SPSel\0?\0CurrentEL\0PAN\0UAO\0"; break;case 6: t="ICC_PMR_EL1\0"; break;} break;case 5: switch(m) {case 1: t="AFSR0_EL1\0AFSR1_EL1\0"; break;case 2: t="ESR_EL1"; break;case 3: t="ERRIDR_EL1\0ERRSELR_EL1\0"; break;case 4: t="ERXFR_EL1\0ERXCTLR_EL1\0ERXSTATUS_EL1\0ERXADDR_EL1\0"; break;case 5: t="ERXMISC0_EL1\0ERXMISC1_EL1\0"; break;} break;case 6: if(m==0) t="FAR_EL1\0"; break;case 7: if(m==4) t="PAR_EL1\0"; break;case 9: switch(m) {case 9: t="PMSCR_EL1\0?\0PMSICR_EL1\0PMSIRR_EL1\0PMSFCR_EL1\0PMSEVFR_EL1\0PMSLATFR_EL1\0PMSIDR_EL1\0PMSIDR_EL1\0"; break;case 10: t="PMBLIMITR_EL1\0PMBPTR_EL1\0?\0PMBSR_EL1\0?\0?\0?\0PMBIDR_EL1\0"; break;case 14: t="?\0PMINTENSET_EL1\0PMINTENCLR_EL1\0"; break;} break;case 10: if(m==4) t="LORSA_EL1\0LOREA_EL1\0LORN_EL1\0LORC_EL1\0?\0?\0?\0LORID_EL1\0"; else if(m!=4&&j==0) { j=m; t="?\0?\0MAIR_EL1\0AMAIR_EL1\0"; }break;case 12: switch(m) {case 0: t="VBAR_EL1\0RVBAR_EL1\0RMR_EL1\0"; break;case 1: t="ISR_EL1\0DISR_EL1\0"; break;case 8: t="ICC_IAR0_EL1\0ICC_EOIR0_EL1\0ICC_HPPIR0_EL1\0ICC_BPR0_EL1\0ICC_AP0R0_EL1\0ICC_AP0R1_EL1\0ICC_AP0R2_EL1\0ICC_AP0R3_EL1\0"; break;case 9: t="ICC_AP1R0_EL1\0ICC_AP1R1_EL1\0ICC_AP1R2_EL1\0ICC_AP1R3_EL1\0"; break;case 11: t="?\0ICC_DIR_EL1\0?\0ICC_RPR_EL1\0?\0ICC_SGI1R_EL1\0ICC_ASGI1R_EL1\0ICC_SGI0R_EL1\0"; break;case 12: t="ICC_IAR1_EL1\0ICC_EOIR1_EL1\0ICC_HPPIR1_EL1\0ICC_BPR1_EL1\0ICC_CTLR_EL1\0ICC_SRE_EL1\0ICC_IGRPEN0_EL1\0ICC_IGRPEN1_EL1\0"; break;} break;case 13: if(m==0) t="?\0CONTEXTIDR_EL1\0?\0?\0TPIDR_EL1\0"; break;case 14: if(m==1) t="CNTKCTL_EL1\0"; break;} break;case 1: if(n==0&&m==0) t="CCSIDR_EL1\0CLIDR_EL1\0?\0?\0?\0?\0?\0AIDR_EL1\0"; break;case 2: if(n==0&&m==0) t="CSSELR_EL1\0"; break;case 3: switch(n) {case 0: if(m==0) t="?\0CTR_EL0\0?\0?\0?\0?\0?\0DCZID_EL0\0"; break;case 4: switch(m) {case 2: t="NZCV\0DAIF\0"; break;case 4: t="FPCR\0FPSR\0"; break;case 5: t="DSPSR_EL0\0DLR_EL0\0"; break;} break;case 9: switch(m) {case 12: t="PMCR_EL0\0PMCNTENSET_EL0\0PMCNTENCLR_EL0\0PMOVSCLR_EL0\0PMSWINC_EL0\0PMSELR_EL0\0PMCEID0_EL0\0PMCEID1_EL0\0"; break;case 13: t="PMCCNTR_EL0\0PMXEVTYPER_EL0\0PMXEVCNTR_EL0\0"; break;case 14: t="PMUSERENR_EL0\0?\0?\0PMOVSSET_EL0\0"; break;} break;
case 13: if(m==0) t="?\0?\0TPIDR_EL0\0TPIDRRO_EL0\0"; break;case 14: switch(m) {case 0: t="CNTFRQ_EL0\0CNTPCT_EL0\0CNTVCT_EL0\0"; break;case 2: t="CNTP_TVAL_EL0\0CNTP_CTL_EL0\0CNTP_CVAL_EL0\0"; break;case 3: t="CNTV_TVAL_EL0\0CNTV_CTL_EL0\0CNTV_CVAL_EL0\0"; break;} break;} break;case 4: switch(n) {case 0: if(m==0) t="VPIDR_EL2\0?\0?\0?\0?\0VMPIDR_EL2\0"; break;case 1: switch(m) {case 0: t="SCTLR_EL2\0ACTLR_EL2\0"; break;case 1: t="HCR_EL2\0MDCR_EL2\0CPTR_EL2\0HSTR_EL2\0?\0?\0?\0HACR_EL2\0"; break;case 2: t="ZCR_EL2\0"; break;} break;case 2: switch(m) {case 0: t="TTBR0_EL2\0?\0TCR_EL2\0"; break;case 1: t="VTTBR0_EL2\0?\0VTCR_EL2\0"; break;} break;case 3: if(m==0) t="DACR32_EL2\0"; break;case 4: switch(m) {case 0: t="SPSR_EL2\0ELR_EL2\0"; break;case 1: t="SP_EL1\0"; break;case 3: t="SPSR_irq\0SPSR_abt\0SPSR_und\0SPSR_fiq\0"; break;} break;case 5: switch(m) {case 0: t="?\0IFSR32_EL2\0"; break;case 1: t="AFSR0_EL2\0AFSR1_EL2\0"; break;case 2: t="ESR_EL2\0?\0?\0VSESR_EL2\0"; break;case 3: t="FPEXC32_EL2\0"; break;} break;case 6: if(m==0) t="FAR_EL2\0?\0?\0?\0HPFAR_EL2\0"; break;case 9: if(m==9) t="PMSCR_EL2\0"; break;case 10: switch(m) {case 2: t="MAIR_EL2\0"; break;case 3: t="AMAIR_EL2\0"; break;} break;case 12: switch(m) {case 0: t="VBAR_EL2\0RVBAR_EL2\0RMR_EL2\0"; break;case 1: t="?\0VDISR_EL2\0"; break;case 8: t="ICH_AP0R0_EL2\0ICH_AP0R1_EL2\0ICH_AP0R2_EL2\0ICH_AP0R3_EL2\0"; break;case 9: t="ICH_AP1R0_EL2\0ICH_AP1R1_EL2\0ICH_AP1R2_EL2\0ICH_AP1R3_EL2\0ICC_SRE_EL2\0"; break;case 11: t="ICH_HCR_EL2\0ICH_VTR_EL2\0ICH_MISR_EL2\0ICH_EISR_EL2\0?\0ICH_ELRSR_EL2\0?\0ICH_VMCR_EL2\0"; break;case 12: t="ICH_LR0_EL2\0ICH_LR1_EL2\0ICH_LR2_EL2\0ICH_LR3_EL2\0ICH_LR4_EL2\0ICH_LR5_EL2\0ICH_LR6_EL2\0ICH_LR7_EL2\0"; break;case 13: t="ICH_LR8_EL2\0ICH_LR9_EL2\0ICH_LR10_EL2\0ICH_LR11_EL2\0ICH_LR12_EL2\0ICH_LR13_EL2\0ICH_LR14_EL2\0ICH_LR15_EL2\0"; break;} break;case 13: if(m==0) t="?\0CONTEXTIDR_EL2\0TPIDR_EL2\0"; break;case 14: switch(m) {case 0: t="?\0?\0?\0CNTVOFF_EL2\0"; break;case 1: t="CNTHCTL_EL2\0"; break;case 2: t="CNTHP_TVAL_EL2\0CNTHP_CTL_EL2\0CNTHP_CVAL_EL2\0"; break;case 3: t="CNTHV_TVAL_EL2\0CNTHV_CTL_EL2\0CNTHV_CVAL_EL2\0"; break;} break;} break;case 5: if(n==4&&m==0) t="SPSR_EL12\0ELR_EL12\0"; break;case 6: if(n==4&&m==1) t="SP_EL2\0"; break;case 7: if(n==14&&m==2) t="CNTPS_TVAL_EL1\0CNTPS_CTL_EL1\0CNTPS_CVAL_EL1\0"; break;} break;}return t?disasm_str(t,j):NULL;}

/*** public API ***/
uint64_t disasm(uint64_t addr, char *str)
{
    uint32_t i=0;
    uint16_t op=0, om=0, j=0;
    uint8_t t=0, s=0, n=0, m=0, c=0, p=0, a=0, d=0, k=0, b=0, q=0, z=0, o=0;
    uint32_t ic32;
    uint64_t iaddr=addr;
    char *names=NULL,*olds=str;
    char *pstate="?\0?\0?\0uao\0pan\0spsel\0daifs\0daifc\0";
    char *conds="eq\0ne\0cs\0cc\0mi\0pl\0vs\0vc\0hi\0ls\0ge\0lt\0gt\0le\0al\0nv\0";
    char *share="?\0oshld\0oshst\0osh\0?\0nshld\0nshst\0nsh\0?\0ishld\0ishst\0ish\0?\0ld\0st\0sy\0";
    char *at_op0="s1e1r\0s1e1w\0s1e0r\0s1e0w\0";
    char *at_op1="s1e1rp\0s1e1wp\0";
    char *at_op2="s1e2r\0s1e2w\0?\0?\0s12e1r\0s12e1w\0s12e0r\0s12e0w\0s1e3r\0s1e3w\0";
    char *dc_op0="?\0ivac\0isw\0";
    char *dc_op1="csw\0cisw\0";
    char *dc_op2="cvac\0cvau\0civac\0";
    char *ic_op="ialluis\0iallu\0?\0ivau\0";
    char *tlbi_op0="vmalle1is\0vae1is\0aside1is\0vaae1is\0?\0vale1is\0?\0vaale1is\0vmalle1\0vae1\0aside1\0vaae1\0?\0vale1\0?\0vaale1\0alle2is\0vae2is\0?\0?\0alle1is\0vale2is\0vmalls12e1is\0alle2\0vae2\0?\0?\0alle1\0vale2\0vmalls12e1\0";
    char *tlbi_op1="ipas2e1is\0ipas2le1is\0ipas2e1\0ipas2el1\0";
    char *tlbi_op2="alle3is\0vae3is\0?\0vale3is\0alle3\0vae3\0?\0vale3\0";
    char *quantum="8b\016b\04h\08h\02s\04s\01d\02d\01q\02q\0";
    char *prf_typ="pld\0pli\0pst\0";
    char *prf_pol="keep\0strm\0";
    char *extend32="uxtb\0uxth\0lsl\0uxtx\0sxtb\0sxth\0sxtw\0sxtx\0";
    char *extend64="uxtb\0uxth\0uxtw\0lsl\0sxtb\0sxth\0sxtw\0sxtx\0";
    char *shift="lsl\0lsr\0asr\0ror\0";
    uint8_t args[9]={0,0,0,0,0,0,0,0,0};

    ic32=*((uint32_t*)addr);

    //handle multiple NOPs at once
    if(ic32==0b11010101000000110010000000011111) {
        while(*((uint32_t*)addr)==ic32) { op++; addr+=4; }
        if(str!=NULL) str+=sprintf(str,"  %d x nop",op);
        *str=0;
        return addr;
    }

    //decode instruction
    if(((ic32>>8)&0b111111110000000001111100)==0b000010000000000001111100) {
        names="stxrb\0stlxrb\0?\0?\0?\0?\0?\0?\0?\0?\0casb\0caslb\0?\0?\0casab\0casalb\0";
        op=((ic32>>20)&0xe)|((ic32>>15)&0x1); d=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Wd; args[1]=disasm_arg_Wt; args[2]=disasm_arg_offs; args[3]=disasm_arg_XnS; args[4]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b111111111011111111111100)==0b000011100010000101101000) {
        names="fcvtn\0";
        z=((ic32>>22)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        q=0;
        args[0]=disasm_arg_Vtzq2; args[1]=disasm_arg_Vnz3; 
    } else
    if(((ic32>>8)&0b111111111011111111111100)==0b000011100010000111101000) {
        names="fcvtl\0";
        z=((ic32>>22)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        q=0;
        args[0]=disasm_arg_Vtz3; args[1]=disasm_arg_Vnzq2; 
    } else
    if(((ic32>>8)&0b111111110011111111111100)==0b000011100010000100101000) {
        names="xtn\0";
        z=((ic32>>22)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        q=0;
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_VnT2; 
    } else
    if(((ic32>>8)&0b111111110011111111111100)==0b000011100010000100111000) {
        names="shll\0";
        z=((ic32>>22)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        q=0;
        args[0]=disasm_arg_Vtz; args[1]=disasm_arg_VnT; args[2]=disasm_arg_shift8; 
    } else
    if(((ic32>>8)&0b111111110010000010011100)==0b000011100010000010010000) {
        names="sqdmlal\0sqdmlsl\0sqdmull\0";
        op=((ic32>>13)&0x3); z=((ic32>>22)&0x3); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        q=0;
        args[0]=disasm_arg_Vtz; args[1]=disasm_arg_VnT; args[2]=disasm_arg_VmT; 
    } else
    if(((ic32>>8)&0b111111111100000010110100)==0b000011110100000000100000) {
        names="smlal\0smlsl\0";
        op=((ic32>>14)&0x1); j=((ic32>>9)&0x4)|((ic32>>20)&0x3); m=((ic32>>16)&0xf); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=1;q=0;
        args[0]=disasm_arg_Vtz3; args[1]=disasm_arg_VnT; args[2]=disasm_arg_VmTs; 
    } else
    if(((ic32>>8)&0b111111111100000011110100)==0b000011110100000010100000) {
        names="smull\0";
        j=((ic32>>9)&0x4)|((ic32>>20)&0x3); m=((ic32>>16)&0xf); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=1;q=0;
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_VnT; args[2]=disasm_arg_VmTs; 
    } else
    if(((ic32>>8)&0b111111111100000010110100)==0b000011111000000000100000) {
        names="smlal\0smlsl\0";
        op=((ic32>>14)&0x1); j=((ic32>>10)&0x2)|((ic32>>21)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=2;q=0;
        args[0]=disasm_arg_Vtz3; args[1]=disasm_arg_VnT; args[2]=disasm_arg_VmTs; 
    } else
    if(((ic32>>8)&0b111111111100000011110100)==0b000011111000000010100000) {
        names="smull\0";
        j=((ic32>>10)&0x2)|((ic32>>21)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=2;q=0;
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_VnT; args[2]=disasm_arg_VmTs; 
    } else
    if(((ic32>>8)&0b111111111111111111111100)==0b000111100110001001000000) {
        names="fcvt\0";
        n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_St; args[1]=disasm_arg_Dn; 
    } else
    if(((ic32>>8)&0b111111110011111001111100)==0b000111100010001001000000) {
        names="fcvt\0";
        z=((ic32>>22)&0x3); k=((ic32>>15)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPk5t; args[1]=disasm_arg_FPz5n; 
    } else
    if(((ic32>>8)&0b111111110011100001111100)==0b000111100010000001000000) {
        names="fmov\0fabs\0fneg\0fsqrt\0?\0?\0?\0?\0frintn\0frintp\0frintm\0frintz\0frinta\0?\0frintx\0frinti\0";
        op=((ic32>>15)&0xf); z=((ic32>>22)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPz5t; args[1]=disasm_arg_FPz5n; 
    } else
    if((ic32&0b11111111001000001111110000001111)==0b00011110001000000010000000000000) {
        names="fcmp\0fcmpe\0";
        op=((ic32>>4)&0x1); z=((ic32>>22)&0x3); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); 
        args[0]=disasm_arg_FPz5n; args[1]=disasm_arg_FPz5m; 
    } else
    if((ic32&0b11111111001000001111110000001111)==0b00011110001000000010000000001000) {
        names="fcmp\0fcmpe\0";
        op=((ic32>>4)&0x1); z=((ic32>>22)&0x3); n=((ic32>>5)&0x1f); 
        args[0]=disasm_arg_FPz5n; args[1]=disasm_arg_simd0; 
    } else
    if((ic32&0b11111111001000000001111111100000)==0b00011110001000000001000000000000) {
        names="fmov\0";
        z=((ic32>>22)&0x3); j=((ic32>>13)&0xff); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPz5t; args[1]=disasm_arg_jz; 
    } else
    if(((ic32>>8)&0b111111110010000000001100)==0b000111100010000000000100) {
        names="ffcmp\0ffcmpe\0";
        op=((ic32>>4)&0x1); z=((ic32>>22)&0x3); m=((ic32>>16)&0x1f); c=((ic32>>12)&0xf); n=((ic32>>5)&0x1f); j=((ic32>>0)&0xf); 
        args[0]=disasm_arg_FPz5n; args[1]=disasm_arg_FPz5m; args[2]=disasm_arg_j; args[3]=disasm_arg_c; 
    } else
    if(((ic32>>8)&0b111111110010000000001100)==0b000111100010000000001000) {
        names="fmul\0fdiv\0fadd\0fsub\0fmax\0fmin\0fmaxnm\0fminmn\0fnmul\0";
        op=((ic32>>12)&0xf); z=((ic32>>22)&0x3); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPz5t; args[1]=disasm_arg_FPz5n; args[2]=disasm_arg_FPz5m; 
    } else
    if(((ic32>>8)&0b111111110010000000001100)==0b000111100010000000001100) {
        names="fcsel\0";
        z=((ic32>>22)&0x3); m=((ic32>>16)&0x1f); c=((ic32>>12)&0xf); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPz5t; args[1]=disasm_arg_FPz5n; args[2]=disasm_arg_FPz5m; args[3]=disasm_arg_c; 
    } else
    if(((ic32>>24)&0b11111111)==0b00011111) {
        names="fmadd\0fmsub\0fnmadd\0fnmsub\0";
        op=((ic32>>20)&0x2)|((ic32>>15)&0x1); z=((ic32>>22)&0x3); m=((ic32>>16)&0x1f); d=((ic32>>10)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPz5t; args[1]=disasm_arg_FPz5n; args[2]=disasm_arg_FPz5m; args[3]=disasm_arg_FPz5t; args[4]=disasm_arg_FPz5n; args[5]=disasm_arg_FPz5d; 
    } else
    if(((ic32>>8)&0b111111111111100011111100)==0b001011110000000011100100) {
        names="movi\0";
        j=((ic32>>11)&0xe0)|((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Dt; args[1]=disasm_arg_imm64; 
    } else
    if(((ic32>>8)&0b111111110010000000001100)==0b001110000010000000000000) {
        names="ldaddb\0ldclrb\0ldeorb\0ldsetb\0ldsmaxb\0ldsminb\0ldumaxb\0lduminb\0swpb\0?\0?\0?\0?\0?\0?\0?\0ldaddlb\0ldclrlb\0ldeorlb\0ldsetlb\0ldsmaxlb\0ldsminlb\0ldumaxlb\0lduminlb\0swplb\0?\0?\0?\0?\0?\0?\0?\0ldaddab\0ldclrab\0ldeorab\0ldsetab\0ldsmaxab\0ldsminab\0ldumaxab\0lduminab\0swpab\0?\0?\0?\0?\0?\0?\0?\0ldaddalb\0ldclralb\0ldeoralb\0ldsetalb\0ldsmaxalb\0ldsminalb\0ldumaxalb\0lduminalb\0swpalb\0";
        op=((ic32>>18)&0x30)|((ic32>>12)&0xf); d=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Wd; args[1]=disasm_arg_Wt; args[2]=disasm_arg_offs; args[3]=disasm_arg_XnS; args[4]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b110111110011111110011100)==0b000011100010000100001000) {
        names="?\0xtn\0sqxtn\0?\0?\0sqxtun\0uqxtn\0fcvtxn\0";
        op=((ic32>>27)&0x4)|((ic32>>13)&0x3); z=((ic32>>22)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        q=0;
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_VnT3; 
    } else
    if(((ic32>>8)&0b110111110010000011011100)==0b000011100010000000010000) {
        names="saddw\0ssubw\0uaddw\0usubw\0";
        op=((ic32>>28)&0x2)|((ic32>>13)&0x1); z=((ic32>>22)&0x3); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        q=0;
        args[0]=disasm_arg_VtT3; args[1]=disasm_arg_VnT3; args[2]=disasm_arg_VmT; 
    } else
    if(((ic32>>8)&0b110111110010000011011100)==0b000011100010000001000000) {
        names="addhn\0subhn\0raddhn\0rsubhn\0";
        op=((ic32>>28)&0x2)|((ic32>>13)&0x1); z=((ic32>>22)&0x3); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        q=0;
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_VnT3; args[2]=disasm_arg_VmT3; 
    } else
    if(((ic32>>8)&0b110111110010000011111100)==0b000011100010000011100000) {
        names="pmull\0umull\0";
        op=((ic32>>29)&0x1); z=((ic32>>22)&0x3); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        q=0;
        args[0]=disasm_arg_VtT4; args[1]=disasm_arg_VnT; args[2]=disasm_arg_VmT; 
    } else
    if(((ic32>>8)&0b110111110010000000001100)==0b000011100010000000000000) {
        names="saddl\0saddw\0ssubl\0ssubw\0addhn\0sabal\0subhn\0sabdl\0smlal\0sqdmlal\0smlsl\0sqdmlsl\0?\0sqdmull\0pmull\0?\0uaddl\0uaddw\0usubl\0usubw\0raddhn\0uabal\0rsubhn\0uabdl\0umlal\0?\0umlsl\0?\0?\0?\0umull\0";
        op=((ic32>>25)&0x10)|((ic32>>12)&0xf); z=((ic32>>22)&0x3); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        q=0;
        args[0]=disasm_arg_VtT3; args[1]=disasm_arg_VnT; args[2]=disasm_arg_VmT; 
    } else
    if(((ic32>>8)&0b110111111100000000100100)==0b000011110100000000100000) {
        names="smlal\0sqdmlal\0smlsl\0sqdmlsl\0smull\0sqdmull\0?\0?\0umlal\0?\0umlsl\0?\0umull\0";
        op=((ic32>>26)&0x8)|((ic32>>13)&0x6)|((ic32>>12)&0x1); j=((ic32>>9)&0x4)|((ic32>>20)&0x3); m=((ic32>>16)&0xf); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=1;q=0;
        args[0]=disasm_arg_Vtz; args[1]=disasm_arg_VnT; args[2]=disasm_arg_VmTs; 
    } else
    if(((ic32>>8)&0b110111111000000011100100)==0b000011110000000010000100) {
        names="?\0rshrn\0sqshrn\0sqrshrn\0sqshrun\0sqrshrun\0uqshrn\0uqrshrn\0";
        op=((ic32>>27)&0x4)|((ic32>>11)&0x3); j=((ic32>>16)&0x7f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        q=0;
        args[0]=disasm_arg_Vtj2; args[1]=disasm_arg_VnTa; args[2]=disasm_arg_shrshift; 
    } else
    if(((ic32>>8)&0b110111111000000011111100)==0b000011110000000010100100) {
        names="sshll\0usshll\0";
        op=((ic32>>29)&0x1); j=((ic32>>16)&0x7f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        q=0;
        args[0]=disasm_arg_Vtj2; args[1]=disasm_arg_VnTa; args[2]=disasm_arg_shlshift; 
    } else
    if(((ic32>>8)&0b110111111100000000100100)==0b000011111000000000100000) {
        names="smlal\0sqdmlal\0smlsl\0sqdmlsl\0smull\0sqdmull\0?\0?\0umlal\0?\0umlsl\0?\0umull\0";
        op=((ic32>>26)&0x8)|((ic32>>13)&0x6)|((ic32>>12)&0x1); j=((ic32>>10)&0x2)|((ic32>>21)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=2;q=0;
        args[0]=disasm_arg_Vtz; args[1]=disasm_arg_VnT; args[2]=disasm_arg_VmTs; 
    } else
    if(((ic32>>8)&0b111111110000000001111100)==0b010010000000000001111100) {
        names="stxrh\0stlxrh\0?\0?\0?\0?\0?\0?\0?\0?\0cash\0caslh\0?\0?\0casah\0casalh\0";
        op=((ic32>>20)&0xe)|((ic32>>15)&0x1); d=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Wd; args[1]=disasm_arg_Wt; args[2]=disasm_arg_offs; args[3]=disasm_arg_XnS; args[4]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b111111111110000011111100)==0b010011100000000000011100) {
        names="ins\0";
        j=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vtj; args[1]=disasm_arg_offs; args[2]=disasm_arg_FPidx; args[3]=disasm_arg_offe; args[4]=disasm_arg_R2n; 
    } else
    if(((ic32>>8)&0b111111111111111111001100)==0b010011100010100001001000) {
        names="aese\0aesd\0aesmc\0aesimc\0";
        op=((ic32>>12)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt16b; args[1]=disasm_arg_Vn16b; 
    } else
    if(((ic32>>8)&0b111111111011111111111100)==0b010011100010000101101000) {
        names="fcvtn2\0";
        z=((ic32>>22)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        q=1;
        args[0]=disasm_arg_Vtzq2; args[1]=disasm_arg_Vnz3; 
    } else
    if(((ic32>>8)&0b111111111011111111111100)==0b010011100010000111101000) {
        names="fcvtl2\0";
        z=((ic32>>22)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        q=1;
        args[0]=disasm_arg_Vtz3; args[1]=disasm_arg_Vnzq2; 
    } else
    if(((ic32>>8)&0b111111110011111111111100)==0b010011100010000100101000) {
        names="xtn2\0";
        z=((ic32>>22)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        q=1;
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_VnT2; 
    } else
    if(((ic32>>8)&0b111111110011111111111100)==0b010011100010000100111000) {
        names="shll2\0";
        z=((ic32>>22)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        q=1;
        args[0]=disasm_arg_Vtz; args[1]=disasm_arg_VnT; args[2]=disasm_arg_shift8; 
    } else
    if(((ic32>>8)&0b111111110010000010011100)==0b010011100010000010010000) {
        names="sqdmlal2\0sqdmlsl2\0sqdmull2\0";
        op=((ic32>>13)&0x3); z=((ic32>>22)&0x3); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        q=1;
        args[0]=disasm_arg_Vtz; args[1]=disasm_arg_VnT; args[2]=disasm_arg_VmT; 
    } else
    if(((ic32>>8)&0b111111111100000010110100)==0b010011110100000000100000) {
        names="smlal2\0smlsl2\0";
        op=((ic32>>14)&0x1); j=((ic32>>9)&0x4)|((ic32>>20)&0x3); m=((ic32>>16)&0xf); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=1;q=1;
        args[0]=disasm_arg_Vtz3; args[1]=disasm_arg_VnT; args[2]=disasm_arg_VmTs; 
    } else
    if(((ic32>>8)&0b111111111100000011110100)==0b010011110100000010100000) {
        names="smull2\0";
        j=((ic32>>9)&0x4)|((ic32>>20)&0x3); m=((ic32>>16)&0xf); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=1;q=1;
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_VnT; args[2]=disasm_arg_VmTs; 
    } else
    if(((ic32>>8)&0b111111111100000010110100)==0b010011111000000000100000) {
        names="smlal2\0smlsl2\0";
        op=((ic32>>14)&0x1); j=((ic32>>10)&0x2)|((ic32>>21)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=2;q=1;
        args[0]=disasm_arg_Vtz3; args[1]=disasm_arg_VnT; args[2]=disasm_arg_VmTs; 
    } else
    if(((ic32>>8)&0b111111111100000011110100)==0b010011111000000010100000) {
        names="smull2\0";
        j=((ic32>>10)&0x2)|((ic32>>21)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=2;q=1;
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_VnT; args[2]=disasm_arg_VmTs; 
    } else
    if((ic32&0b11111111000000000000000000010000)==0b01010100000000000000000000000000) {
        names="b.%s\0";
        i=((ic32>>23)&1?(-1<<19):0)|((ic32>>5)&0x7ffff); c=((ic32>>0)&0xf); 
        args[0]=disasm_arg_labeli4; 
    } else
    if(((ic32>>8)&0b111111111110000011111100)==0b010111100000000000000100) {
        names="dup\0";
        j=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPjt; args[1]=disasm_arg_Vnj; args[2]=disasm_arg_offs; args[3]=disasm_arg_FPidx; args[4]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b111111111110000011111100)==0b010111100000000000110000) {
        names="sha1su0\0";
        m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt4s; args[1]=disasm_arg_Vn4s; args[2]=disasm_arg_Vm4s; 
    } else
    if(((ic32>>8)&0b111111111110000011001100)==0b010111100000000000000000) {
        names="sha1c\0sha1p\0sha1m\0sha1su0\0";
        op=((ic32>>12)&0x3); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Qt; args[1]=disasm_arg_Sn; args[2]=disasm_arg_Vm4s; 
    } else
    if(((ic32>>8)&0b111111111110000011101100)==0b010111100000000001000000) {
        names="sha256h\0sha256h2\0";
        op=((ic32>>12)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Qt; args[1]=disasm_arg_Qn; args[2]=disasm_arg_Vm4s; 
    } else
    if(((ic32>>8)&0b111111111110000011111100)==0b010111100000000001100000) {
        names="sha256su1\0";
        m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt4s; args[1]=disasm_arg_Vn4s; args[2]=disasm_arg_Vm4s; 
    } else
    if(((ic32>>8)&0b111111111111111111111100)==0b010111100010100000001000) {
        names="sha1h\0";
        n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_St; args[1]=disasm_arg_Sn; 
    } else
    if(((ic32>>8)&0b111111111111111111111100)==0b010111100010100000011000) {
        names="sha1su1\0";
        n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt4s; args[1]=disasm_arg_Vn4s; 
    } else
    if(((ic32>>8)&0b111111111111111111111100)==0b010111100010100000101000) {
        names="sha256su0\0";
        n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt4s; args[1]=disasm_arg_Vn4s; 
    } else
    if(((ic32>>8)&0b111111111110000011111100)==0b010111100100000000011100) {
        names="fmulx\0";
        m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Ht; args[1]=disasm_arg_Hn; args[2]=disasm_arg_Hm; 
    } else
    if(((ic32>>8)&0b111111111110000011111100)==0b010111100100000000100100) {
        names="fcmeq\0";
        m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Ht; args[1]=disasm_arg_Hn; args[2]=disasm_arg_Hm; 
    } else
    if(((ic32>>8)&0b111111111010000011111100)==0b010111100010000011011100) {
        names="fmulx\0";
        z=((ic32>>22)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPt; args[1]=disasm_arg_FPn; args[2]=disasm_arg_FPm; 
    } else
    if(((ic32>>8)&0b111111111010000011111100)==0b010111100010000011100100) {
        names="fcmeq\0";
        z=((ic32>>22)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPt; args[1]=disasm_arg_FPn; args[2]=disasm_arg_FPm; 
    } else
    if(((ic32>>8)&0b111111110111111111001100)==0b010111100011000011001000) {
        names="fmaxnmp\0faddp\0?\0fmaxp\0fminnmp\0?\0?\0fminp\0";
        op=((ic32>>21)&0x4)|((ic32>>12)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Ht; args[1]=disasm_arg_Vn2h; 
    } else
    if(((ic32>>8)&0b111111110110000011111100)==0b010111100100000000111100) {
        names="frecps\0frsqrts\0";
        op=((ic32>>23)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Ht; args[1]=disasm_arg_Hn; args[2]=disasm_arg_Hm; 
    } else
    if(((ic32>>8)&0b111111110011111111111100)==0b010111100011000110111000) {
        names="addp\0";
        z=((ic32>>22)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPz3t; args[1]=disasm_arg_Vn2d; 
    } else
    if(((ic32>>8)&0b111111110010000011111100)==0b010111100010000011111100) {
        names="frecps\0frsqrts\0";
        op=((ic32>>23)&0x1); z=((ic32>>22)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPt; args[1]=disasm_arg_FPn; args[2]=disasm_arg_FPm; 
    } else
    if(((ic32>>8)&0b111111110010000010011100)==0b010111100010000010010000) {
        names="sqdmlal\0sqdmlsl\0sqdmull\0";
        op=((ic32>>13)&0x3); z=((ic32>>22)&0x3); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPz4t; args[1]=disasm_arg_FPz2n; args[2]=disasm_arg_FPz2m; 
    } else
    if(((ic32>>8)&0b111111111100000011100100)==0b010111110100000011000000) {
        names="sqdmulh\0sqrdmulh\0";
        op=((ic32>>12)&0x1); j=((ic32>>9)&0x4)|((ic32>>20)&0x3); m=((ic32>>16)&0xf); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=1;
        args[0]=disasm_arg_FPz4t; args[1]=disasm_arg_FPz4n; args[2]=disasm_arg_VmTs; 
    } else
    if(((ic32>>8)&0b111111111100000000110100)==0b010111110100000000110000) {
        names="sqdmlal\0sqdmlsl\0sqdmull\0";
        op=((ic32>>14)&0x3); j=((ic32>>9)&0x4)|((ic32>>20)&0x3); m=((ic32>>16)&0xf); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=1;
        args[0]=disasm_arg_FPz4t; args[1]=disasm_arg_FPz3n; args[2]=disasm_arg_VmTs; 
    } else
    if(((ic32>>8)&0b111111111000000011011100)==0b010111110000000001010100) {
        names="shl\0sqshl\0";
        op=((ic32>>13)&0x1); j=((ic32>>16)&0x7f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Dt; args[1]=disasm_arg_Dn; args[2]=disasm_arg_shlshift; 
    } else
    if(((ic32>>8)&0b111111111100000011100100)==0b010111111000000011000000) {
        names="sqdmulh\0sqrdmulh\0";
        op=((ic32>>12)&0x1); j=((ic32>>10)&0x2)|((ic32>>21)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=2;
        args[0]=disasm_arg_FPz4t; args[1]=disasm_arg_FPz4n; args[2]=disasm_arg_VmTs; 
    } else
    if(((ic32>>8)&0b111111111100000000110100)==0b010111111000000000110000) {
        names="sqdmlal\0sqdmlsl\0sqdmull\0";
        op=((ic32>>14)&0x3); j=((ic32>>10)&0x2)|((ic32>>21)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=2;
        args[0]=disasm_arg_FPz4t; args[1]=disasm_arg_FPz3n; args[2]=disasm_arg_VmTs; 
    } else
    if(((ic32>>16)&0b1111111111000000)==0b0110100011000000) {
        names="ldpsw\0";
        i=((ic32>>21)&1?(-1<<7):0)|((ic32>>15)&0x7f); m=((ic32>>10)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Xt; args[1]=disasm_arg_Xm; args[2]=disasm_arg_offs; args[3]=disasm_arg_XnS; args[4]=disasm_arg_offe; args[5]=disasm_arg_im4_opt; 
    } else
    if(((ic32>>16)&0b1111111101000000)==0b0110100101000000) {
        names="ldpsw\0";
        p=((ic32>>23)&0x1); i=((ic32>>21)&1?(-1<<7):0)|((ic32>>15)&0x7f); m=((ic32>>10)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Xt; args[1]=disasm_arg_Xm; args[2]=disasm_arg_offs; args[3]=disasm_arg_XnS; args[4]=disasm_arg_im4_opt; args[5]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b111111111110000010000100)==0b011011100000000000000100) {
        names="ins\0";
        j=((ic32>>16)&0x1f); k=((ic32>>11)&0xf); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vtj; args[1]=disasm_arg_offs; args[2]=disasm_arg_FPidx; args[3]=disasm_arg_offe; args[4]=disasm_arg_Vnj; args[5]=disasm_arg_offs; args[6]=disasm_arg_FPidxk; args[7]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b111111110011111111001100)==0b011011100011000011001000) {
        names="fmaxnmv\0?\0?\0fmaxv\0fminnmv\0?\0?\0fminv\0";
        op=((ic32>>21)&0x4)|((ic32>>12)&0x3); z=((ic32>>22)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPt; args[1]=disasm_arg_Vn4s; 
    } else
    if(((ic32>>8)&0b111111111111100011111100)==0b011011110000000011100100) {
        names="movi\0";
        j=((ic32>>11)&0xe0)|((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt2d; args[1]=disasm_arg_imm64; 
    } else
    if(((ic32>>8)&0b111111111111100011111100)==0b011011110000000011110100) {
        names="fmov\0";
        j=((ic32>>11)&0xe0)|((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt2d; args[1]=disasm_arg_F64; 
    } else
    if(((ic32>>8)&0b111111110010000000001100)==0b011110000010000000000000) {
        names="ldaddh\0ldclrh\0ldeorh\0ldseth\0ldsmaxh\0ldsminh\0ldumaxh\0lduminh\0swph\0?\0?\0?\0?\0?\0?\0?\0ldaddlh\0ldclrlh\0ldeorlh\0ldsetlh\0ldsmaxlh\0ldsminlh\0ldumaxlh\0lduminlh\0swplh\0?\0?\0?\0?\0?\0?\0?\0ldaddah\0ldclrah\0ldeorah\0ldsetah\0ldsmaxah\0ldsminah\0ldumaxah\0lduminah\0swpah\0?\0?\0?\0?\0?\0?\0?\0ldaddalh\0ldclralh\0ldeoralh\0ldsetalh\0ldsmaxalh\0ldsminalh\0ldumaxalh\0lduminalh\0swpalh\0";
        op=((ic32>>18)&0x30)|((ic32>>12)&0xf); d=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Wd; args[1]=disasm_arg_Wt; args[2]=disasm_arg_offs; args[3]=disasm_arg_XnS; args[4]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b111111111101111111111100)==0b011111100001000011001000) {
        names="fmaxnmp\0";
        z=((ic32>>21)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPt; args[1]=disasm_arg_Vnz; 
    } else
    if(((ic32>>8)&0b111111111110000011110100)==0b011111100100000000100100) {
        names="fcmge\0facge\0";
        op=((ic32>>11)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Ht; args[1]=disasm_arg_Hn; args[2]=disasm_arg_Hm; 
    } else
    if(((ic32>>8)&0b111111111010000011110100)==0b011111100010000011100100) {
        names="fcmge\0facge\0";
        op=((ic32>>11)&0x1); z=((ic32>>22)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPt; args[1]=disasm_arg_FPn; args[2]=disasm_arg_FPm; 
    } else
    if(((ic32>>8)&0b111111111110000011111100)==0b011111101100000000010100) {
        names="fabd\0";
        m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Ht; args[1]=disasm_arg_Hn; args[2]=disasm_arg_Hm; 
    } else
    if(((ic32>>8)&0b111111111110000011110100)==0b011111101100000000100100) {
        names="fcmgt\0facgt\0";
        op=((ic32>>11)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Ht; args[1]=disasm_arg_Hn; args[2]=disasm_arg_Hm; 
    } else
    if(((ic32>>8)&0b111111111010000011111100)==0b011111101010000011010100) {
        names="fabd\0";
        z=((ic32>>22)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPt; args[1]=disasm_arg_FPn; args[2]=disasm_arg_FPm; 
    } else
    if(((ic32>>8)&0b111111111010000011110100)==0b011111101010000011100100) {
        names="fcmgt\0facgt\0";
        op=((ic32>>11)&0x1); z=((ic32>>22)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPt; args[1]=disasm_arg_FPn; args[2]=disasm_arg_FPm; 
    } else
    if(((ic32>>8)&0b111111110010000011110100)==0b011111100000000010000100) {
        names="sqrdmlah\0sqrdmlsh\0";
        op=((ic32>>11)&0x1); z=((ic32>>22)&0x3); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPz2t; args[1]=disasm_arg_FPz2n; args[2]=disasm_arg_FPz2m; 
    } else
    if(((ic32>>8)&0b111111110011111111001100)==0b011111100011000011001000) {
        names="?\0faddp\0?\0fmaxp\0fminnmp\0?\0?\0fminp\0";
        op=((ic32>>21)&0x4)|((ic32>>12)&0x3); z=((ic32>>22)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPt; args[1]=disasm_arg_Vnz; 
    } else
    if(((ic32>>8)&0b111111111100000011010100)==0b011111110100000011010000) {
        names="sqrdmlah\0sqrdmlsh\0";
        op=((ic32>>13)&0x1); j=((ic32>>9)&0x4)|((ic32>>20)&0x3); m=((ic32>>16)&0xf); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=1;
        args[0]=disasm_arg_FPz4t; args[1]=disasm_arg_FPz3n; args[2]=disasm_arg_VmTs; 
    } else
    if(((ic32>>8)&0b111111111000000011111100)==0b011111110000000001100100) {
        names="sqshlu\0";
        j=((ic32>>16)&0x7f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Dt; args[1]=disasm_arg_Dn; args[2]=disasm_arg_shlshift; 
    } else
    if(((ic32>>8)&0b111111111000000011111100)==0b011111110000000001110100) {
        names="uqshl\0";
        j=((ic32>>16)&0x7f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPjt2; args[1]=disasm_arg_FPjn2; args[2]=disasm_arg_shlshift; 
    } else
    if(((ic32>>8)&0b111111111100000011010100)==0b011111111000000011010000) {
        names="sqrdmlah\0sqrdmlsh\0";
        op=((ic32>>13)&0x1); j=((ic32>>10)&0x2)|((ic32>>21)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=2;
        args[0]=disasm_arg_FPz4t; args[1]=disasm_arg_FPz3n; args[2]=disasm_arg_VmTs; 
    } else
    if(((ic32>>8)&0b110111110011111110011100)==0b010011100010000100001000) {
        names="?\0xtn2\0sqxtn2\0?\0?\0sqxtun2\0uqxtn2\0fcvtxn2\0";
        op=((ic32>>27)&0x4)|((ic32>>13)&0x3); z=((ic32>>22)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        q=1;
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_VnT3; 
    } else
    if(((ic32>>8)&0b110111110010000011011100)==0b010011100010000000010000) {
        names="saddw2\0ssubw2\0uaddw2\0usubw2\0";
        op=((ic32>>28)&0x2)|((ic32>>13)&0x1); z=((ic32>>22)&0x3); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        q=1;
        args[0]=disasm_arg_VtT3; args[1]=disasm_arg_VnT3; args[2]=disasm_arg_VmT; 
    } else
    if(((ic32>>8)&0b110111110010000011011100)==0b010011100010000001000000) {
        names="addhn2\0subhn2\0raddhn2\0rsubhn2\0";
        op=((ic32>>28)&0x2)|((ic32>>13)&0x1); z=((ic32>>22)&0x3); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        q=1;
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_VnT3; args[2]=disasm_arg_VmT3; 
    } else
    if(((ic32>>8)&0b110111110010000011111100)==0b010011100010000011100000) {
        names="pmull2\0umull2\0";
        op=((ic32>>29)&0x1); z=((ic32>>22)&0x3); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        q=1;
        args[0]=disasm_arg_VtT4; args[1]=disasm_arg_VnT; args[2]=disasm_arg_VmT; 
    } else
    if(((ic32>>8)&0b110111110010000000001100)==0b010011100010000000000000) {
        names="saddl2\0saddw2\0ssubl2\0ssubw2\0addhn2\0sabal2\0subhn2\0sabdl2\0smlal2\0sqdmlal2\0smlsl2\0sqdmlsl2\0?\0sqdmull2\0pmull2\0?\0uaddl2\0uaddw2\0usubl2\0usubw2\0raddhn2\0uabal2\0rsubhn2\0uabdl2\0umlal2\0?\0umlsl2\0?\0?\0?\0umull2\0";
        op=((ic32>>25)&0x10)|((ic32>>12)&0xf); z=((ic32>>22)&0x3); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        q=1;
        args[0]=disasm_arg_VtT3; args[1]=disasm_arg_VnT; args[2]=disasm_arg_VmT; 
    } else
    if(((ic32>>8)&0b110111111100000000100100)==0b010011110100000000100000) {
        names="smlal2\0sqdmlal2\0smlsl2\0sqdmlsl2\0smull2\0sqdmull2\0?\0?\0umlal2\0?\0umlsl2\0?\0umull2\0";
        op=((ic32>>26)&0x8)|((ic32>>13)&0x6)|((ic32>>12)&0x1); j=((ic32>>9)&0x4)|((ic32>>20)&0x3); m=((ic32>>16)&0xf); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=1;q=1;
        args[0]=disasm_arg_Vtz; args[1]=disasm_arg_VnT; args[2]=disasm_arg_VmTs; 
    } else
    if(((ic32>>8)&0b110111111000000011100100)==0b010011110000000010000100) {
        names="?\0rshrn2\0sqshrn2\0sqrshrn2\0sqshrun2\0sqrshrun2\0uqshrn2\0uqrshrn2\0";
        op=((ic32>>27)&0x4)|((ic32>>11)&0x3); j=((ic32>>16)&0x7f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        q=1;
        args[0]=disasm_arg_Vtj2; args[1]=disasm_arg_VnTa; args[2]=disasm_arg_shrshift; 
    } else
    if(((ic32>>8)&0b110111111000000011111100)==0b010011110000000010100100) {
        names="sshll2\0usshll2\0";
        op=((ic32>>29)&0x1); j=((ic32>>16)&0x7f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        q=1;
        args[0]=disasm_arg_Vtj2; args[1]=disasm_arg_VnTa; args[2]=disasm_arg_shlshift; 
    } else
    if(((ic32>>8)&0b110111111100000000100100)==0b010011111000000000100000) {
        names="smlal2\0sqdmlal2\0smlsl2\0sqdmlsl2\0smull2\0sqdmull2\0?\0?\0umlal2\0?\0umlsl2\0?\0umull2\0";
        op=((ic32>>26)&0x8)|((ic32>>13)&0x6)|((ic32>>12)&0x1); j=((ic32>>10)&0x2)|((ic32>>21)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=2;q=1;
        args[0]=disasm_arg_Vtz; args[1]=disasm_arg_VnT; args[2]=disasm_arg_VmTs; 
    } else
    if(((ic32>>8)&0b110111111111111111001100)==0b010111101111100011001000) {
        names="fcmgt\0fcmeq\0fcmlt\0?\0fcmge\0fcmle\0";
        op=((ic32>>27)&0x4)|((ic32>>12)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Ht; args[1]=disasm_arg_Hn; args[2]=disasm_arg_simd0; 
    } else
    if(((ic32>>8)&0b110111111011111111001100)==0b010111101010000011001000) {
        names="fcmgt\0fcmeq\0fcmlt\0?\0fcmge\0fcmle\0";
        op=((ic32>>27)&0x4)|((ic32>>12)&0x3); z=((ic32>>22)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPt; args[1]=disasm_arg_FPn; args[2]=disasm_arg_simd0; 
    } else
    if(((ic32>>8)&0b110111110111111110001100)==0b010111100111100110001000) {
        names="?\0?\0fcvtns\0fcvtms\0fcvtas\0scvtf\0?\0?\0?\0?\0fcvtps\0fcvtzs\0?\0frecpe\0?\0frecpx\0?\0?\0fcvtnu\0fcvtmu\0fcvtau\0ucvtf\0?\0?\0?\0?\0fcvtpu\0fcvtzu\0?\0frsqrte\0";
        op=((ic32>>25)&0x10)|((ic32>>20)&0x8)|((ic32>>12)&0x7); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Ht; args[1]=disasm_arg_Hn; 
    } else
    if(((ic32>>8)&0b110111110011111111001100)==0b010111100010000010001000) {
        names="cmgt\0cmeq\0cmlt\0abs\0cmge\0cmle\0?\0neg\0";
        op=((ic32>>27)&0x4)|((ic32>>12)&0x3); z=((ic32>>22)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPz3t; args[1]=disasm_arg_FPz3n; args[2]=disasm_arg_simd0; 
    } else
    if(((ic32>>8)&0b110111110011111100111100)==0b010111100010000000111000) {
        names="suqadd\0sqabs\0abs\0?\0usqadd\0sqneg\0neg\0";
        op=((ic32>>27)&0x4)|((ic32>>14)&0x3); z=((ic32>>22)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPz3t; args[1]=disasm_arg_FPz3n; 
    } else
    if(((ic32>>8)&0b110111110011111110011100)==0b010111100010000100001000) {
        names="?\0?\0sqxtn\0?\0?\0sqxtun\0uqxtn\0fcvtxn\0";
        op=((ic32>>27)&0x4)|((ic32>>13)&0x3); z=((ic32>>22)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPz3t; args[1]=disasm_arg_FPz4n; 
    } else
    if(((ic32>>8)&0b110111110011111110001100)==0b010111100010000110001000) {
        names="?\0?\0fcvtns\0fcvtms\0fcvtas\0scvtf\0?\0?\0?\0?\0fcvtps\0fcvtzs\0?\0frecpe\0?\0frecpx\0?\0?\0fcvtnu\0fcvtmu\0fcvtau\0ucvtf\0?\0?\0?\0?\0fcvtpu\0fcvtzu\0?\0frsqrte\0";
        op=((ic32>>25)&0x10)|((ic32>>20)&0x8)|((ic32>>12)&0x7); z=((ic32>>22)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPt; args[1]=disasm_arg_FPn; 
    } else
    if(((ic32>>8)&0b110111110010000000000100)==0b010111100010000000000100) {
        names="?\0sqadd\0?\0?\0?\0sqsub\0cmgt\0cmge\0sshl\0sqshl\0srshl\0sqrshl\0?\0?\0?\0?\0add\0cmtst\0?\0?\0?\0?\0sqdmulh\0?\0?\0?\0?\0?\0?\0?\0?\0?\0?\0uqadd\0?\0?\0?\0uqsub\0cmhi\0cmhs\0ushl\0uqshl\0urshl\0uqrshl\0?\0?\0?\0?\0sub\0cmeq\0?\0?\0?\0?\0sqrdmulh\0";
        op=((ic32>>24)&0x20)|((ic32>>11)&0x1f); z=((ic32>>22)&0x3); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPz3t; args[1]=disasm_arg_FPz3n; args[2]=disasm_arg_FPz3m; 
    } else
    if(((ic32>>8)&0b110111111100000000110100)==0b010111110000000000010000) {
        names="fmla\0fmls\0fmul\0?\0?\0?\0fmulx\0";
        op=((ic32>>27)&0x4)|((ic32>>14)&0x3); j=((ic32>>9)&0x4)|((ic32>>20)&0x3); m=((ic32>>16)&0xf); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Ht; args[1]=disasm_arg_Hn; args[2]=disasm_arg_VmHs; 
    } else
    if(((ic32>>8)&0b110111111000000010001100)==0b010111110000000000000100) {
        names="sshr\0ssra\0srshr\0srsra\0?\0shl\0?\0sqshl\0ushr\0usra\0urshr\0ursra\0sri\0sli\0sqshlu\0uqshl\0";
        op=((ic32>>26)&0x8)|((ic32>>12)&0x7); j=((ic32>>16)&0x7f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Dt; args[1]=disasm_arg_Dn; args[2]=disasm_arg_shrshift; 
    } else
    if(((ic32>>8)&0b110111111000000011100100)==0b010111110000000010000100) {
        names="?\0?\0sqshrn\0sqrshrn\0sqshrun\0sqrshrun\0uqshrn\0uqrshrn\0";
        op=((ic32>>27)&0x4)|((ic32>>11)&0x3); j=((ic32>>16)&0x7f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPjt; args[1]=disasm_arg_FPnj; args[2]=disasm_arg_shrshift; 
    } else
    if(((ic32>>8)&0b110111111000000011111100)==0b010111110000000011100100) {
        names="scvtf\0ucvtf\0";
        op=((ic32>>29)&0x1); j=((ic32>>16)&0x7f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPjt2; args[1]=disasm_arg_FPjn2; args[2]=disasm_arg_shrshift; 
    } else
    if(((ic32>>8)&0b110111111000000011111100)==0b010111110000000011111100) {
        names="fcvtzs\0fcvtzu\0";
        op=((ic32>>29)&0x1); j=((ic32>>16)&0x7f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPjt; args[1]=disasm_arg_FPjn2; args[2]=disasm_arg_shrshift; 
    } else
    if(((ic32>>8)&0b110111111100000000110100)==0b010111111000000000010000) {
        names="fmla\0fmls\0fmul\0sqrdmulh\0?\0?\0fmulx\0sqrdmlah\0";
        op=((ic32>>27)&0x4)|((ic32>>14)&0x3); j=((ic32>>10)&0x2)|((ic32>>21)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=0;
        args[0]=disasm_arg_FPt; args[1]=disasm_arg_FPn; args[2]=disasm_arg_VmTs2; 
    } else
    if(((ic32>>8)&0b110111111110000000110100)==0b010111111100000000010000) {
        names="fmla\0fmls\0fmul\0?\0?\0?\0fmulx\0";
        op=((ic32>>27)&0x4)|((ic32>>14)&0x3); j=((ic32>>11)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=1;
        args[0]=disasm_arg_FPt; args[1]=disasm_arg_FPn; args[2]=disasm_arg_VmTs2; 
    } else
    if(((ic32>>8)&0b101111111010000001111100)==0b000010000010000001111100) {
        names="casp\0caspl\0caspa\0caspal\0";
        op=((ic32>>21)&0x2)|((ic32>>15)&0x1); s=((ic32>>30)&0x1); d=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Rd; args[1]=disasm_arg_Rd1; args[2]=disasm_arg_Rt; args[3]=disasm_arg_Rt1; args[4]=disasm_arg_offs; args[5]=disasm_arg_XnS; args[6]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111110011111101111100)==0b000010000001111101111100) {
        names="?\0?\0ldxrb\0ldaxrb\0stllrb\0stlrb\0ldlarb\0ldarb\0?\0?\0ldxrh\0ldaxrh\0stllrh\0stlrh\0ldlarh\0ldarh\0";
        op=((ic32>>27)&0x8)|((ic32>>21)&0x6)|((ic32>>15)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Wt; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111111011111111010000)==0b000011000000000000000000) {
        names="st4\0st1\0ld4\0ld1\0";
        op=((ic32>>21)&0x2)|((ic32>>13)&0x1); q=((ic32>>30)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_Vt2T; args[2]=disasm_arg_Vt3T; args[3]=disasm_arg_Vt4T; args[4]=disasm_arg_offs; args[5]=disasm_arg_XnS; args[6]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111111011111111110000)==0b000011000000000001110000) {
        names="st1\0ld1\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111111011111111010000)==0b000011000000000001000000) {
        names="st3\0st1\0ld3\0ld1\0";
        op=((ic32>>21)&0x2)|((ic32>>13)&0x1); q=((ic32>>30)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_Vt2T; args[2]=disasm_arg_Vt3T; args[3]=disasm_arg_offs; args[4]=disasm_arg_XnS; args[5]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111111011111111010000)==0b000011000000000010000000) {
        names="st2\0st1\0ld2\0ld1\0";
        op=((ic32>>21)&0x2)|((ic32>>13)&0x1); q=((ic32>>30)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_Vt2T; args[2]=disasm_arg_offs; args[3]=disasm_arg_XnS; args[4]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111111011111111010000)==0b000011001001111100000000) {
        names="st4\0st1\0ld4\0ld1\0";
        op=((ic32>>21)&0x2)|((ic32>>13)&0x1); q=((ic32>>30)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_Vt2T; args[2]=disasm_arg_Vt3T; args[3]=disasm_arg_Vt4T; args[4]=disasm_arg_offs; args[5]=disasm_arg_XnS; args[6]=disasm_arg_offe; args[7]=disasm_arg_Qi; 
    } else
    if(((ic32>>8)&0b101111111011111111110000)==0b000011001001111101110000) {
        names="st1\0ld1\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_Qi1; 
    } else
    if(((ic32>>8)&0b101111111011111111010000)==0b000011001001111101000000) {
        names="st3\0st1\0ld3\0ld1\0";
        op=((ic32>>21)&0x2)|((ic32>>13)&0x1); q=((ic32>>30)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_Vt2T; args[2]=disasm_arg_Vt3T; args[3]=disasm_arg_offs; args[4]=disasm_arg_XnS; args[5]=disasm_arg_offe; args[6]=disasm_arg_Qi3; 
    } else
    if(((ic32>>8)&0b101111111011111111010000)==0b000011001001111110000000) {
        names="st2\0st1\0ld2\0ld1\0";
        op=((ic32>>21)&0x2)|((ic32>>13)&0x1); q=((ic32>>30)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_Vt2T; args[2]=disasm_arg_offs; args[3]=disasm_arg_XnS; args[4]=disasm_arg_offe; args[5]=disasm_arg_Qi2; 
    } else
    if(((ic32>>8)&0b101111111010000011010000)==0b000011001000000000000000) {
        names="st4\0st1\0ld4\0ld1\0";
        op=((ic32>>21)&0x2)|((ic32>>13)&0x1); q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_Vt2T; args[2]=disasm_arg_Vt3T; args[3]=disasm_arg_Vt4T; args[4]=disasm_arg_offs; args[5]=disasm_arg_XnS; args[6]=disasm_arg_offe; args[7]=disasm_arg_Xm; 
    } else
    if(((ic32>>8)&0b101111111010000011110000)==0b000011001000000001110000) {
        names="st1\0ld1\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_Xm; 
    } else
    if(((ic32>>8)&0b101111111010000011010000)==0b000011001000000001000000) {
        names="st3\0st1\0ld3\0ld1\0";
        op=((ic32>>21)&0x2)|((ic32>>13)&0x1); q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_Vt2T; args[2]=disasm_arg_Vt3T; args[3]=disasm_arg_offs; args[4]=disasm_arg_XnS; args[5]=disasm_arg_offe; args[6]=disasm_arg_Xm; 
    } else
    if(((ic32>>8)&0b101111111010000011010000)==0b000011001000000010000000) {
        names="st2\0st1\0ld2\0ld1\0";
        op=((ic32>>21)&0x2)|((ic32>>13)&0x1); q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_Vt2T; args[2]=disasm_arg_offs; args[3]=disasm_arg_XnS; args[4]=disasm_arg_offe; args[5]=disasm_arg_Xm; 
    } else
    if(((ic32>>8)&0b101111111111111111110000)==0b000011010100000011000000) {
        names="ld1r\0";
        q=((ic32>>30)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111111111111111110000)==0b000011010100000011100000) {
        names="ld3r\0";
        q=((ic32>>30)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_Vt2T; args[2]=disasm_arg_Vt3T; args[3]=disasm_arg_offs; args[4]=disasm_arg_XnS; args[5]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111111111111111110000)==0b000011010110000011000000) {
        names="ld2r\0";
        q=((ic32>>30)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_Vt2T; args[2]=disasm_arg_offs; args[3]=disasm_arg_XnS; args[4]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111111111111111110000)==0b000011010110000011100000) {
        names="ld4r\0";
        q=((ic32>>30)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_Vt2T; args[2]=disasm_arg_Vt3T; args[3]=disasm_arg_Vt4T; args[4]=disasm_arg_offs; args[5]=disasm_arg_XnS; args[6]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111111011111111100000)==0b000011010000000000000000) {
        names="st1\0ld1\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); s=((ic32>>12)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtB; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111111011111111100000)==0b000011010000000000100000) {
        names="st3\0ld3\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); s=((ic32>>12)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt3B; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111111011111111100000)==0b000011010000000001000000) {
        names="st1\0ld1\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); s=((ic32>>12)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtH; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111111011111111100000)==0b000011010000000001100000) {
        names="st3\0ld3\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); s=((ic32>>12)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt3H; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111111011111111111100)==0b000011010000000010000100) {
        names="st1\0ld1\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtD; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111111011111111101100)==0b000011010000000010000000) {
        names="st1\0ld1\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); s=((ic32>>12)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtS; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111111011111111111100)==0b000011010000000010100100) {
        names="st3\0ld3\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt3D; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111111011111111101100)==0b000011010000000010100000) {
        names="st3\0ld3\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); s=((ic32>>12)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt3S; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111111011111111100000)==0b000011010010000000000000) {
        names="st2\0ld2\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); s=((ic32>>12)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt2B; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111111011111111100000)==0b000011010010000000100000) {
        names="st4\0ld4\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); s=((ic32>>12)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt4B; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111111011111111100000)==0b000011010010000001000000) {
        names="st2\0ld2\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); s=((ic32>>12)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt2H; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111111011111111100000)==0b000011010010000001100000) {
        names="st4\0ld4\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); s=((ic32>>12)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt4H; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111111011111111111100)==0b000011010010000010000100) {
        names="st2\0ld2\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt2D; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111111011111111101100)==0b000011010010000010000000) {
        names="st2\0ld2\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); s=((ic32>>12)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt2S; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111111011111111111100)==0b000011010010000010100100) {
        names="st4\0ld4\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt4D; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111111011111111101100)==0b000011010010000010100000) {
        names="st4\0ld4\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); s=((ic32>>12)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt4S; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111111111111111110000)==0b000011011101111111000000) {
        names="ld1r\0";
        q=((ic32>>30)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_z; 
    } else
    if(((ic32>>8)&0b101111111111111111110000)==0b000011011101111111100000) {
        names="ld3r\0";
        q=((ic32>>30)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_Vt2T; args[2]=disasm_arg_Vt3T; args[3]=disasm_arg_offs; args[4]=disasm_arg_XnS; args[5]=disasm_arg_offe; args[6]=disasm_arg_z3; 
    } else
    if(((ic32>>8)&0b101111111110000011110000)==0b000011011100000011000000) {
        names="ld1r\0";
        q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_Xm; 
    } else
    if(((ic32>>8)&0b101111111110000011110000)==0b000011011100000011100000) {
        names="ld3r\0";
        q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_Vt2T; args[2]=disasm_arg_Vt3T; args[3]=disasm_arg_offs; args[4]=disasm_arg_XnS; args[5]=disasm_arg_offe; args[6]=disasm_arg_Xm; 
    } else
    if(((ic32>>8)&0b101111111111111111110000)==0b000011011111111111000000) {
        names="ld2r\0";
        q=((ic32>>30)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_Vt2T; args[2]=disasm_arg_offs; args[3]=disasm_arg_XnS; args[4]=disasm_arg_offe; args[5]=disasm_arg_z2; 
    } else
    if(((ic32>>8)&0b101111111111111111110000)==0b000011011111111111100000) {
        names="ld4r\0";
        q=((ic32>>30)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_Vt2T; args[2]=disasm_arg_Vt3T; args[3]=disasm_arg_Vt4T; args[4]=disasm_arg_offs; args[5]=disasm_arg_XnS; args[6]=disasm_arg_offe; args[7]=disasm_arg_z4; 
    } else
    if(((ic32>>8)&0b101111111110000011110000)==0b000011011110000011000000) {
        names="ld2r\0";
        q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_Vt2T; args[2]=disasm_arg_offs; args[3]=disasm_arg_XnS; args[4]=disasm_arg_offe; args[5]=disasm_arg_Xm; 
    } else
    if(((ic32>>8)&0b101111111110000011110000)==0b000011011110000011100000) {
        names="ld4r\0";
        q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_Vt2T; args[2]=disasm_arg_Vt3T; args[3]=disasm_arg_Vt4T; args[4]=disasm_arg_offs; args[5]=disasm_arg_XnS; args[6]=disasm_arg_offe; args[7]=disasm_arg_Xm; 
    } else
    if(((ic32>>8)&0b101111111011111111100000)==0b000011011001111100000000) {
        names="st1\0ld1\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); s=((ic32>>12)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtB; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_i1; 
    } else
    if(((ic32>>8)&0b101111111011111111100000)==0b000011011001111100100000) {
        names="st3\0ld3\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); s=((ic32>>12)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt3B; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_i3; 
    } else
    if(((ic32>>8)&0b101111111011111111100000)==0b000011011001111101000000) {
        names="st1\0ld1\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); s=((ic32>>12)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtH; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_i2; 
    } else
    if(((ic32>>8)&0b101111111011111111100000)==0b000011011001111101100000) {
        names="st3\0ld3\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); s=((ic32>>12)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt3H; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_i6; 
    } else
    if(((ic32>>8)&0b101111111011111111111100)==0b000011011001111110000100) {
        names="st1\0ld1\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtD; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_i8; 
    } else
    if(((ic32>>8)&0b101111111011111111101100)==0b000011011001111110000000) {
        names="st1\0ld1\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); s=((ic32>>12)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtS; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_i4; 
    } else
    if(((ic32>>8)&0b101111111011111111111100)==0b000011011001111110100100) {
        names="st3\0ld3\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt3D; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_i24; 
    } else
    if(((ic32>>8)&0b101111111011111111101100)==0b000011011001111110100000) {
        names="st3\0ld3\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); s=((ic32>>12)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt3S; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_i12; 
    } else
    if(((ic32>>8)&0b101111111010000011100000)==0b000011011000000000000000) {
        names="st1\0ld1\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); s=((ic32>>12)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtB; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_Xm; 
    } else
    if(((ic32>>8)&0b101111111010000011100000)==0b000011011000000000100000) {
        names="st3\0ld3\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); s=((ic32>>12)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt3B; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_Xm; 
    } else
    if(((ic32>>8)&0b101111111010000011100000)==0b000011011000000001000000) {
        names="st1\0ld1\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); s=((ic32>>12)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtH; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_Xm; 
    } else
    if(((ic32>>8)&0b101111111010000011100000)==0b000011011000000001100000) {
        names="st3\0ld3\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); s=((ic32>>12)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt3H; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_Xm; 
    } else
    if(((ic32>>8)&0b101111111010000011111100)==0b000011011000000010000100) {
        names="st1\0ld1\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtD; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_Xm; 
    } else
    if(((ic32>>8)&0b101111111010000011101100)==0b000011011000000010000000) {
        names="st1\0ld1\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); s=((ic32>>12)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtS; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_Xm; 
    } else
    if(((ic32>>8)&0b101111111010000011111100)==0b000011011000000010100100) {
        names="st3\0ld3\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt3D; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_Xm; 
    } else
    if(((ic32>>8)&0b101111111010000011101100)==0b000011011000000010100000) {
        names="st3\0ld3\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); s=((ic32>>12)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt3S; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_Xm; 
    } else
    if(((ic32>>8)&0b101111111011111111100000)==0b000011011011111100000000) {
        names="st2\0ld2\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); s=((ic32>>12)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt2B; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_i2; 
    } else
    if(((ic32>>8)&0b101111111011111111100000)==0b000011011011111100100000) {
        names="st4\0ld4\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); s=((ic32>>12)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt4B; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_i4; 
    } else
    if(((ic32>>8)&0b101111111011111111100000)==0b000011011011111101000000) {
        names="st2\0ld2\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); s=((ic32>>12)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt2H; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_i4; 
    } else
    if(((ic32>>8)&0b101111111011111111100000)==0b000011011011111101100000) {
        names="st4\0ld4\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); s=((ic32>>12)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt4H; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_i8; 
    } else
    if(((ic32>>8)&0b101111111011111111111100)==0b000011011011111110000100) {
        names="st2\0ld2\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt2D; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_i16; 
    } else
    if(((ic32>>8)&0b101111111011111111101100)==0b000011011011111110000000) {
        names="st2\0ld2\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); s=((ic32>>12)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt2S; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_i8; 
    } else
    if(((ic32>>8)&0b101111111011111111111100)==0b000011011011111110100100) {
        names="st4\0ld4\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt4D; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_i32; 
    } else
    if(((ic32>>8)&0b101111111011111111101100)==0b000011011011111110100000) {
        names="st4\0ld4\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); s=((ic32>>12)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt4S; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_i16; 
    } else
    if(((ic32>>8)&0b101111111010000011100000)==0b000011011010000000000000) {
        names="st2\0ld2\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); s=((ic32>>12)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt2B; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_Xm; 
    } else
    if(((ic32>>8)&0b101111111010000011100000)==0b000011011010000000100000) {
        names="st4\0ld4\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); s=((ic32>>12)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt4B; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_Xm; 
    } else
    if(((ic32>>8)&0b101111111010000011100000)==0b000011011010000001000000) {
        names="st2\0ld2\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); s=((ic32>>12)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt2H; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_Xm; 
    } else
    if(((ic32>>8)&0b101111111010000011100000)==0b000011011010000001100000) {
        names="st4\0ld4\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); s=((ic32>>12)&0x1); z=((ic32>>10)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt4H; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_Xm; 
    } else
    if(((ic32>>8)&0b101111111010000011111100)==0b000011011010000010000100) {
        names="st2\0ld2\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt2D; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_Xm; 
    } else
    if(((ic32>>8)&0b101111111010000011101100)==0b000011011010000010000000) {
        names="st2\0ld2\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); s=((ic32>>12)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt2S; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_Xm; 
    } else
    if(((ic32>>8)&0b101111111010000011111100)==0b000011011010000010100100) {
        names="st4\0ld4\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt4D; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_Xm; 
    } else
    if(((ic32>>8)&0b101111111010000011101100)==0b000011011010000010100000) {
        names="st4\0ld4\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); s=((ic32>>12)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt4S; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_Xm; 
    } else
    if(((ic32>>8)&0b101111111110000011111100)==0b000011100000000000000100) {
        names="dup\0";
        q=((ic32>>30)&0x1); j=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vtjq; args[1]=disasm_arg_Vnj; args[2]=disasm_arg_offs; args[3]=disasm_arg_FPidx; args[4]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111111110000011101100)==0b000011100000000000000000) {
        names="tbl\0tbx\0";
        op=((ic32>>12)&0x1); q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=0;
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_Vn116b; args[2]=disasm_arg_VmT; 
    } else
    if(((ic32>>8)&0b101111111110000011101100)==0b000011100000000000100000) {
        names="tbl\0tbx\0";
        op=((ic32>>12)&0x1); q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=0;
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_Vn216b; args[2]=disasm_arg_VmT; 
    } else
    if(((ic32>>8)&0b101111111110000011101100)==0b000011100000000000101100) {
        names="smov\0umov\0";
        op=((ic32>>12)&0x1); s=((ic32>>30)&0x1); j=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Rt; args[1]=disasm_arg_Vnj; args[2]=disasm_arg_offs; args[3]=disasm_arg_FPidx; args[4]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111111110000011101100)==0b000011100000000001000000) {
        names="tbl\0tbx\0";
        op=((ic32>>12)&0x1); q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=0;
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_Vn316b; args[2]=disasm_arg_VmT; 
    } else
    if(((ic32>>8)&0b101111111110000011101100)==0b000011100000000001100000) {
        names="tbl\0tbx\0";
        op=((ic32>>12)&0x1); q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=0;
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_Vn416b; args[2]=disasm_arg_VmT; 
    } else
    if(((ic32>>8)&0b101111111110000011111100)==0b000011100100000000011100) {
        names="fmulx\0";
        q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtH1; args[1]=disasm_arg_VnH1; args[2]=disasm_arg_VmH1; 
    } else
    if(((ic32>>8)&0b101111111110000011111100)==0b000011100100000000100100) {
        names="fcmeq\0";
        q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtH1; args[1]=disasm_arg_VnH1; args[2]=disasm_arg_VmH1; 
    } else
    if(((ic32>>8)&0b101111111111111111101100)==0b000011100111100110001000) {
        names="frintn\0frintm\0";
        op=((ic32>>12)&0x1); q=((ic32>>30)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=1;
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_VnT; 
    } else
    if(((ic32>>8)&0b101111111111111111111100)==0b000011100111100111111000) {
        names="fabs\0";
        q=((ic32>>30)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=1;
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_VnT; 
    } else
    if(((ic32>>8)&0b101111111010000011111100)==0b000011100010000000011100) {
        names="fmulx\0";
        q=((ic32>>30)&0x1); z=((ic32>>22)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vtzq; args[1]=disasm_arg_Vnzq; args[2]=disasm_arg_Vmzq; 
    } else
    if(((ic32>>8)&0b101111111111111111101100)==0b000011101111100110001000) {
        names="frintp\0frintz\0";
        op=((ic32>>12)&0x1); q=((ic32>>30)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=1;
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_VnT; 
    } else
    if(((ic32>>8)&0b101111110111111111001100)==0b000011100011000011001000) {
        names="fmaxnmv\0?\0?\0fmaxv\0fminnmv\0?\0?\0fminv\0";
        op=((ic32>>21)&0x4)|((ic32>>12)&0x3); q=((ic32>>30)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=0;
        args[0]=disasm_arg_Ht; args[1]=disasm_arg_Vnzq2; 
    } else
    if(((ic32>>8)&0b101111110110000011111100)==0b000011100100000000111100) {
        names="frecps\0frsqrts\0";
        op=((ic32>>23)&0x1); q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtH1; args[1]=disasm_arg_VnH1; args[2]=disasm_arg_VmH1; 
    } else
    if(((ic32>>8)&0b101111110000000010000000)==0b000011100000000000000000) {
        names="?\0?\0?\0?\0?\0?\0uzp1\0?\0?\0?\0trn1\0?\0?\0?\0zip1\0?\0?\0?\0?\0?\0?\0?\0uzp2\0?\0?\0?\0trn2\0?\0?\0?\0zip2\0?\0?\0shadd\0?\0sqadd\0?\0srhadd\0?\0?\0?\0?\0?\0sqsub\0?\0cmgt\0?\0cmge\0?\0sshl\0?\0sqshl\0?\0srshl\0?\0sqrshl\0?\0smax\0?\0smin\0?\0sabd\0?\0saba\0";
        op=((ic32>>16)&0x20)|((ic32>>10)&0x1f); q=((ic32>>30)&0x1); z=((ic32>>22)&0x3); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_VnT; args[2]=disasm_arg_VmT; 
    } else
    if(((ic32>>8)&0b101111111111100011111100)==0b000011110000000011100100) {
        names="movi\0";
        q=((ic32>>30)&0x1); j=((ic32>>11)&0xe0)|((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=0;
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_imm8; 
    } else
    if(((ic32>>8)&0b101111111111100011111100)==0b000011110000000011110100) {
        names="fmov\0";
        q=((ic32>>30)&0x1); j=((ic32>>11)&0xe0)|((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=2;
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_F32; 
    } else
    if(((ic32>>8)&0b101111111111100011111100)==0b000011110000000011111100) {
        names="fmov\0";
        q=((ic32>>30)&0x1); j=((ic32>>11)&0xe0)|((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=1;
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_F16; 
    } else
    if(((ic32>>8)&0b101111111100000010100100)==0b000011110100000010000000) {
        names="mul\0?\0sqdmulh\0sqrdmulh\0";
        op=((ic32>>13)&0x2)|((ic32>>12)&0x1); j=((ic32>>9)&0x4)|((ic32>>20)&0x3); q=((ic32>>30)&0x1); m=((ic32>>16)&0xf); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=1;
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_VnT; args[2]=disasm_arg_VmTs; 
    } else
    if(((ic32>>8)&0b101111111000000011001100)==0b000011110000000000000100) {
        names="sshr\0ssra\0srshr\0srsra\0";
        op=((ic32>>12)&0x3); q=((ic32>>30)&0x1); j=((ic32>>16)&0x7f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vtj2; args[1]=disasm_arg_Vnj2; args[2]=disasm_arg_shrshift; 
    } else
    if(((ic32>>8)&0b101111111000000011111100)==0b000011110000000011100100) {
        names="scvtf\0";
        q=((ic32>>30)&0x1); j=((ic32>>16)&0x7f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vtj2; args[1]=disasm_arg_Vnj2; args[2]=disasm_arg_shrshift; 
    } else
    if(((ic32>>8)&0b101111111000000011111100)==0b000011110000000011111100) {
        names="fcvtzs\0";
        q=((ic32>>30)&0x1); j=((ic32>>16)&0x7f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vtj2; args[1]=disasm_arg_Vnj2; args[2]=disasm_arg_shrshift; 
    } else
    if(((ic32>>8)&0b101111111100000010100100)==0b000011111000000010000000) {
        names="mul\0fmul\0sqdmulh\0sqrdmulh\0";
        op=((ic32>>13)&0x2)|((ic32>>12)&0x1); j=((ic32>>10)&0x2)|((ic32>>21)&0x1); q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=2;
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_VnT; args[2]=disasm_arg_VmTs; 
    } else
    if(((ic32>>24)&0b10111111)==0b00011000) {
        names="ldr\0";
        s=((ic32>>30)&0x1); i=((ic32>>23)&1?(-1<<19):0)|((ic32>>5)&0x7ffff); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Rt; args[1]=disasm_arg_labeli4; 
    } else
    if(((ic32>>8)&0b101111111110000011111100)==0b000111100000000000001100) {
        names="dup\0";
        q=((ic32>>30)&0x1); j=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        s=q;
        args[0]=disasm_arg_Vtjq; args[1]=disasm_arg_Rn; 
    } else
    if(((ic32>>8)&0b101111111110000010000100)==0b001011100000000000000000) {
        names="ext\0";
        q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); i=((ic32>>14)&1?(-1<<4):0)|((ic32>>11)&0xf); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=0;
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_VnT; args[2]=disasm_arg_VmT; args[3]=disasm_arg_i; 
    } else
    if(((ic32>>8)&0b101111111110000011110100)==0b001011100100000000100100) {
        names="fcmge\0facge\0";
        op=((ic32>>11)&0x1); q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtH1; args[1]=disasm_arg_VnH1; args[2]=disasm_arg_VmH1; 
    } else
    if(((ic32>>8)&0b101111111111111111111100)==0b001011100111100110011000) {
        names="frintx\0";
        q=((ic32>>30)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=1;
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_VnT; 
    } else
    if(((ic32>>8)&0b101111111011111111111100)==0b001011100010000001011000) {
        names="not\0rbit\0";
        op=((ic32>>22)&0x1); q=((ic32>>30)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=0;
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_VnT; 
    } else
    if(((ic32>>8)&0b101111111110000011111100)==0b001011101100000000010100) {
        names="fabd\0";
        q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtH1; args[1]=disasm_arg_VnH1; args[2]=disasm_arg_VmH1; 
    } else
    if(((ic32>>8)&0b101111111110000011110100)==0b001011101100000000100100) {
        names="fcmgt\0facgt\0";
        op=((ic32>>11)&0x1); q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtH1; args[1]=disasm_arg_VnH1; args[2]=disasm_arg_VmH1; 
    } else
    if(((ic32>>8)&0b101111111111111111111100)==0b001011101111100011111000) {
        names="fneg\0";
        q=((ic32>>30)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=1;
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_VnT; 
    } else
    if(((ic32>>8)&0b101111111111111111101100)==0b001011101111100110001000) {
        names="frinta\0frinti\0";
        op=((ic32>>12)&0x1); q=((ic32>>30)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=1;
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_VnT; 
    } else
    if(((ic32>>8)&0b101111111111111111111100)==0b001011101111100111111000) {
        names="fsqrt\0";
        q=((ic32>>30)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=1;
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_VnT; 
    } else
    if(((ic32>>8)&0b101111110010000000000100)==0b001011100010000000000100) {
        names="uhadd\0uqadd\0urhadd\0?\0uhsub\0uqsub\0cmhi\0cmhs\0ushl\0uqshl\0urshl\0uqrshl\0umax\0umin\0uabd\0uaba\0sub\0cmeq\0mls\0pmul\0umaxp\0uminp\0cqrdmulh\0";
        op=((ic32>>11)&0x1f); q=((ic32>>30)&0x1); z=((ic32>>22)&0x3); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_VnT; args[2]=disasm_arg_VmT; 
    } else
    if(((ic32>>8)&0b101111111100000011010100)==0b001011110100000011010000) {
        names="sqrdmlah\0sqrdmlsh\0";
        op=((ic32>>13)&0x1); j=((ic32>>9)&0x4)|((ic32>>20)&0x3); q=((ic32>>30)&0x1); m=((ic32>>16)&0xf); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=1;
        args[0]=disasm_arg_Vtz; args[1]=disasm_arg_VnT; args[2]=disasm_arg_VmTs; 
    } else
    if(((ic32>>8)&0b101111111000000011111100)==0b001011110000000011111100) {
        names="fcvtzu\0";
        q=((ic32>>30)&0x1); j=((ic32>>16)&0x7f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vtj2; args[1]=disasm_arg_Vnj2; args[2]=disasm_arg_shrshift; 
    } else
    if(((ic32>>8)&0b101111111000000000001100)==0b001011110000000000000100) {
        names="ushr\0usra\0urshr\0ursra\0sri\0sli\0sqshlu\0uqshl\0?\0?\0?\0?\0?\0?\0ucvtf\0";
        op=((ic32>>12)&0xf); q=((ic32>>30)&0x1); j=((ic32>>16)&0x7f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vtj2; args[1]=disasm_arg_Vnj2; args[2]=disasm_arg_shrshift; 
    } else
    if(((ic32>>8)&0b101111111100000011010100)==0b001011111000000011010000) {
        names="sqrdmlah\0sqrdmlsh\0";
        op=((ic32>>13)&0x1); j=((ic32>>10)&0x2)|((ic32>>21)&0x1); q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=2;
        args[0]=disasm_arg_Vtz; args[1]=disasm_arg_VnT; args[2]=disasm_arg_VmTs; 
    } else
    if(((ic32>>8)&0b101111110000000010110100)==0b001011110000000000000000) {
        names="mla\0mls\0";
        op=((ic32>>14)&0x1); j=((ic32>>10)&0x2)|((ic32>>21)&0x1); q=((ic32>>30)&0x1); z=((ic32>>22)&0x3); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_VnT; args[2]=disasm_arg_VmTs; 
    } else
    if(((ic32>>8)&0b101111111010000000001100)==0b001110000000000000000100) {
        names="strb\0ldrb\0strh\0ldrh\0";
        op=((ic32>>29)&0x2)|((ic32>>22)&0x1); i=((ic32>>20)&1?(-1<<9):0)|((ic32>>12)&0x1ff); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Wt; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_i_opt; 
    } else
    if(((ic32>>8)&0b101111111010000000000100)==0b001110000000000000000100) {
        names="strb\0ldrb\0strh\0ldrh\0";
        op=((ic32>>29)&0x2)|((ic32>>22)&0x1); i=((ic32>>20)&1?(-1<<9):0)|((ic32>>12)&0x1ff); p=((ic32>>11)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Wt; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_i_opt; args[4]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111111010000000001100)==0b001110000010000000001000) {
        names="strb\0ldrb\0strh\0ldrh\0";
        op=((ic32>>29)&0x2)|((ic32>>22)&0x1); m=((ic32>>16)&0x1f); o=((ic32>>13)&0x7); j=((ic32>>12)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Wt; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_Rom; args[4]=disasm_arg_amountj; args[5]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111111010000000001100)==0b001110001000000000000100) {
        names="ldrsb\0ldrsh\0";
        op=((ic32>>30)&0x1); s=((ic32>>22)&0x1); i=((ic32>>20)&1?(-1<<9):0)|((ic32>>12)&0x1ff); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_nRt; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_i_opt; 
    } else
    if(((ic32>>8)&0b101111111010000000000100)==0b001110001000000000000000) {
        names="ldursb\0?\0ldursh\0ldtrsh\0";
        op=((ic32>>29)&0x2)|((ic32>>11)&0x1); s=((ic32>>22)&0x1); i=((ic32>>20)&1?(-1<<9):0)|((ic32>>12)&0x1ff); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_nRt; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_i_opt; args[4]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111111010000000000100)==0b001110001000000000000100) {
        names="ldrsb\0ldrsh\0";
        op=((ic32>>30)&0x1); s=((ic32>>22)&0x1); i=((ic32>>20)&1?(-1<<9):0)|((ic32>>12)&0x1ff); p=((ic32>>11)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_nRt; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_i_opt; args[4]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111111010000000001100)==0b001110001010000000001000) {
        names="ldrsb\0ldrsh\0";
        op=((ic32>>30)&0x1); s=((ic32>>22)&0x1); m=((ic32>>16)&0x1f); o=((ic32>>13)&0x7); j=((ic32>>12)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_nRt; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_Rom; args[4]=disasm_arg_amountj; args[5]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111110010000000000100)==0b001110000000000000000000) {
        names="sturb\0sttrb\0ldurb\0ldtrb\0?\0ldtrsb\0?\0ldtrsb\0sturh\0sttrh\0ldurh\0ldtrh\0";
        op=((ic32>>27)&0x8)|((ic32>>21)&0x6)|((ic32>>11)&0x1); i=((ic32>>20)&1?(-1<<9):0)|((ic32>>12)&0x1ff); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Wt; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_i_opt; args[4]=disasm_arg_offe; 
    } else
    if(((ic32>>16)&0b1011111110000000)==0b0011100100000000) {
        names="strb\0ldrb\0strh\0ldrh\0";
        op=((ic32>>29)&0x2)|((ic32>>22)&0x1); j=((ic32>>10)&0xfff); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Wt; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_j_opt; args[4]=disasm_arg_offe; 
    } else
    if(((ic32>>16)&0b1011111110000000)==0b0011100110000000) {
        names="ldrsb\0ldrsh\0";
        op=((ic32>>30)&0x1); s=((ic32>>22)&0x1); j=((ic32>>10)&0xfff); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_nRt; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_j_opt; args[4]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b100111111111111111001100)==0b000011101111100011001000) {
        names="fcmgt\0fcmeq\0fcmlt\0?\0fcmge\0fcmle\0?\0fneg\0";
        op=((ic32>>27)&0x4)|((ic32>>12)&0x3); q=((ic32>>30)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtH1; args[1]=disasm_arg_VnH1; args[2]=disasm_arg_simd0; 
    } else
    if(((ic32>>8)&0b100111111011111111001100)==0b000011101010000011001000) {
        names="fcmgt\0fcmeq\0fcmlt\0?\0fcmge\0fcmle\0?\0fneg\0";
        op=((ic32>>27)&0x4)|((ic32>>12)&0x3); q=((ic32>>30)&0x1); z=((ic32>>22)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vtzq; args[1]=disasm_arg_Vnzq; args[2]=disasm_arg_simd0; 
    } else
    if(((ic32>>8)&0b100111110110000011000100)==0b000011100100000000000100) {
        names="fmaxnm\0fmla\0fadd\0fmulx\0fcmeq\0?\0fmax\0frecps\0fminnm\0fmls\0fsub\0?\0?\0?\0fmin\0frsqrts\0fmaxnmp\0?\0faddp\0fmul\0fcmge\0facge\0fmaxp\0fdiv\0fminnmp\0?\0fabd\0?\0fcmgt\0facgt\0fminp\0";
        op=((ic32>>25)&0x10)|((ic32>>20)&0x8)|((ic32>>11)&0x7); q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=1;
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_VnT; args[2]=disasm_arg_VmT; 
    } else
    if(((ic32>>8)&0b100111110111111110001100)==0b000011100111100110001000) {
        names="frintn\0frintm\0fcvtns\0fcvtms\0fcvtas\0scvtf\0?\0fabs\0frintp\0frintz\0fcvtps\0fcvtzs\0?\0frecpe\0?\0frecpx\0?\0frintx\0fcvtnu\0fcvtmu\0fcvtau\0ucvtf\0?\0?\0frinta\0frinti\0fcvtpu\0fcvtzu\0?\0frsqrte\0?\0fsqrt\0";
        op=((ic32>>25)&0x10)|((ic32>>20)&0x8)|((ic32>>12)&0x7); q=((ic32>>30)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtH1; args[1]=disasm_arg_VnH1; 
    } else
    if(((ic32>>8)&0b100111110010000011111100)==0b000011100000000010010100) {
        names="sdot\0udot\0";
        op=((ic32>>29)&0x1); q=((ic32>>30)&0x1); z=((ic32>>22)&0x3); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_Vnzq; args[2]=disasm_arg_Vmzq; 
    } else
    if(((ic32>>8)&0b100111110011111110111100)==0b000011100010000000101000) {
        names="saddlp\0sadalp\0uaddlp\0uadalp\0";
        op=((ic32>>28)&0x2)|((ic32>>14)&0x1); q=((ic32>>30)&0x1); z=((ic32>>22)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vtzq2; args[1]=disasm_arg_VnT; 
    } else
    if(((ic32>>8)&0b100111110011111111001100)==0b000011100010000010001000) {
        names="cmgt\0cmeq\0cmlt\0abs\0cmge\0cmle\0?\0neg\0";
        op=((ic32>>27)&0x4)|((ic32>>12)&0x3); q=((ic32>>30)&0x1); z=((ic32>>22)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_VnT; args[2]=disasm_arg_simd0; 
    } else
    if(((ic32>>8)&0b100111110011111100001100)==0b000011100010000000001000) {
        names="rev64\0rev16\0saddlp\0suqadd\0cls\0cnt\0sadalp\0sqabs\0cmgt\0cmeq\0cmlt\0abs\0?\0?\0?\0?\0rev32\0?\0uaddlp\0usqadd\0clz\0?\0uadalp\0sqneg\0cmge\0cmle\0?\0neg\0";
        op=((ic32>>25)&0x10)|((ic32>>12)&0xf); q=((ic32>>30)&0x1); z=((ic32>>22)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_VnT; 
    } else
    if(((ic32>>8)&0b100111110011111010001100)==0b000011100010000010001000) {
        names="?\0?\0?\0?\0?\0?\0?\0?\0frintn\0frintm\0fcvtns\0fcvtms\0fcvtas\0scvtf\0?\0fabs\0?\0?\0?\0?\0fcmgt\0fcmeq\0fcmlt\0?\0frintp\0frintz\0fcvtps\0fcvtzs\0urecpe\0frecpe\0?\0frecpx\0?\0?\0?\0?\0?\0?\0?\0?\0?\0frintx\0fcvtnu\0fcvtmu\0fcvtau\0ucvtf\0?\0?\0?\0?\0?\0?\0fcmge\0fcmle\0?\0fneg\0frinta\0frinti\0fcvtpu\0fcvtzu\0?\0frsqrte\0?\0fsqrt\0";
        op=((ic32>>24)&0x20)|((ic32>>19)&0x10)|((ic32>>13)&0x8)|((ic32>>12)&0x7); q=((ic32>>30)&0x1); z=((ic32>>22)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vtzq; args[1]=disasm_arg_Vnzq; 
    } else
    if(((ic32>>8)&0b100111110011111111111100)==0b000011100011000000111000) {
        names="saddlv\0uaddlv\0";
        op=((ic32>>29)&0x1); q=((ic32>>30)&0x1); z=((ic32>>22)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPz4t; args[1]=disasm_arg_VnT; 
    } else
    if(((ic32>>8)&0b100111110011111011101100)==0b000011100011000010101000) {
        names="smaxv\0?\0sminv\0addv\0umaxv\0?\0uminv\0";
        op=((ic32>>27)&0x4)|((ic32>>15)&0x2)|((ic32>>12)&0x1); q=((ic32>>30)&0x1); z=((ic32>>22)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPz3t; args[1]=disasm_arg_VnT; 
    } else
    if(((ic32>>8)&0b100111110010000011111100)==0b000011100010000000011100) {
        names="and\0bic\0orr\0orn\0eor\0bsl\0bit\0bif\0";
        op=((ic32>>27)&0x4)|((ic32>>22)&0x3); q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=0;
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_VnT; args[2]=disasm_arg_VmT; 
    } else
    if(((ic32>>8)&0b100111110010000011000100)==0b000011100010000011000100) {
        names="fmaxnm\0fmla\0fadd\0?\0fcmeq\0?\0fmax\0frecps\0fminnm\0fmls\0fsub\0?\0?\0?\0fmin\0frsqrts\0fmaxnmp\0?\0faddp\0fmul\0fcmge\0facge\0fmaxp\0fdiv\0fminnmp\0?\0fabd\0?\0fcmgt\0facgt\0fminp\0";
        op=((ic32>>25)&0x10)|((ic32>>20)&0x8)|((ic32>>11)&0x7); q=((ic32>>30)&0x1); z=((ic32>>22)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vtzq; args[1]=disasm_arg_Vnzq; args[2]=disasm_arg_Vmzq; 
    } else
    if(((ic32>>8)&0b100111110000000011000100)==0b000011100000000010000100) {
        names="?\0?\0sdot\0?\0?\0?\0?\0?\0add\0cmtst\0mla\0mul\0smaxp\0sminp\0sqdmulh\0addp\0sqrdmlah\0sqrdmlsh\0udot\0?\0?\0?\0?\0?\0sub\0cmeq\0mls\0pmul\0umaxp\0uminp\0cqrdmulh\0";
        op=((ic32>>25)&0x10)|((ic32>>18)&0x8)|((ic32>>11)&0x7); q=((ic32>>30)&0x1); z=((ic32>>22)&0x3); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_VnT; args[2]=disasm_arg_VmT; 
    } else
    if(((ic32>>8)&0b100111111111100010001100)==0b000011110000000000000100) {
        names="movi\0orr\0mvni\0bic\0";
        op=((ic32>>28)&0x2)|((ic32>>12)&0x1); q=((ic32>>30)&0x1); j=((ic32>>11)&0xe0)|((ic32>>5)&0x1f); k=((ic32>>13)&0x3); t=((ic32>>0)&0x1f); 
        z=2;
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_imm8; args[2]=disasm_arg_amountk_opt; 
    } else
    if(((ic32>>8)&0b100111111111100011001100)==0b000011110000000010000100) {
        names="movi\0orr\0mvni\0bic\0";
        op=((ic32>>28)&0x2)|((ic32>>12)&0x1); q=((ic32>>30)&0x1); j=((ic32>>11)&0xe0)|((ic32>>5)&0x1f); k=((ic32>>13)&0x1); t=((ic32>>0)&0x1f); 
        z=1;
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_imm8; args[2]=disasm_arg_amountk_opt; 
    } else
    if(((ic32>>8)&0b100111111111100011101100)==0b000011110000000011000100) {
        names="movi\0mvni\0";
        op=((ic32>>29)&0x1); q=((ic32>>30)&0x1); j=((ic32>>11)&0xe0)|((ic32>>5)&0x1f); k=((ic32>>12)&0x1); t=((ic32>>0)&0x1f); 
        z=2;
        args[0]=disasm_arg_VtT; args[1]=disasm_arg_imm8; args[2]=disasm_arg_amountk2_opt; 
    } else
    if(((ic32>>8)&0b100111111100000000110100)==0b000011110000000000010000) {
        names="fmla\0fmls\0fmul\0?\0?\0?\0fmulx\0";
        op=((ic32>>27)&0x4)|((ic32>>14)&0x3); j=((ic32>>9)&0x4)|((ic32>>20)&0x3); q=((ic32>>30)&0x1); m=((ic32>>16)&0xf); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_VtH1; args[1]=disasm_arg_VnH1; args[2]=disasm_arg_VmHs; 
    } else
    if(((ic32>>8)&0b100111111000000011001100)==0b000011110000000001000100) {
        names="?\0shl\0?\0sqshl\0sri\0sli\0sqshlu\0uqshl\0";
        op=((ic32>>27)&0x4)|((ic32>>12)&0x3); q=((ic32>>30)&0x1); j=((ic32>>16)&0x7f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vtj2; args[1]=disasm_arg_Vnj2; args[2]=disasm_arg_shlshift; 
    } else
    if(((ic32>>8)&0b100111111100000011110100)==0b000011111000000011100000) {
        names="sdot\0udot\0";
        op=((ic32>>29)&0x1); j=((ic32>>10)&0x2)|((ic32>>21)&0x1); q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=0;
        args[0]=disasm_arg_Vtzq; args[1]=disasm_arg_VnT; args[2]=disasm_arg_VmTs4b; 
    } else
    if(((ic32>>8)&0b100111111100000000110100)==0b000011111000000000010000) {
        names="fmla\0fmls\0fmul\0sqrdmulh\0?\0?\0fmulx\0sqrdmlah\0";
        op=((ic32>>27)&0x4)|((ic32>>14)&0x3); j=((ic32>>10)&0x2)|((ic32>>21)&0x1); q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=0;
        args[0]=disasm_arg_Vtzq; args[1]=disasm_arg_Vnzq; args[2]=disasm_arg_VmTs2; 
    } else
    if(((ic32>>8)&0b100111111110000000110100)==0b000011111100000000010000) {
        names="fmla\0fmls\0fmul\0?\0?\0?\0fmulx\0";
        op=((ic32>>27)&0x4)|((ic32>>14)&0x3); j=((ic32>>11)&0x1); q=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=1;
        args[0]=disasm_arg_Vtzq; args[1]=disasm_arg_Vnzq; args[2]=disasm_arg_VmTs2; 
    } else
    if(((ic32>>8)&0b111111111110000001111100)==0b100010000000000001111100) {
        names="stxr\0stlxr\0";
        op=((ic32>>15)&0x1); d=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Wd; args[1]=disasm_arg_Wt; args[2]=disasm_arg_offs; args[3]=disasm_arg_XnS; args[4]=disasm_arg_offe; 
    } else
    if(((ic32>>24)&0b11111111)==0b10011000) {
        names="ldrsw\0";
        i=((ic32>>23)&1?(-1<<19):0)|((ic32>>5)&0x7ffff); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Xt; args[1]=disasm_arg_labeli4; 
    } else
    if(((ic32>>8)&0b111111110110000001111100)==0b100110110010000001111100) {
        names="smull\0smnegl\0umull\0umnegl\0";
        op=((ic32>>22)&0x2)|((ic32>>15)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Xt; args[1]=disasm_arg_Wn; args[2]=disasm_arg_Wm; 
    } else
    if(((ic32>>16)&0b1111111101100000)==0b1001101100100000) {
        names="smaddl\0smsubl\0umaddl\0umsubl\0";
        op=((ic32>>22)&0x2)|((ic32>>15)&0x1); m=((ic32>>16)&0x1f); d=((ic32>>10)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Xt; args[1]=disasm_arg_Wn; args[2]=disasm_arg_Wm; args[3]=disasm_arg_Xd; 
    } else
    if(((ic32>>8)&0b111111110110000011111100)==0b100110110100000001111100) {
        names="smulh\0umulh\0";
        op=((ic32>>23)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Xt; args[1]=disasm_arg_Xn; args[2]=disasm_arg_Xm; 
    } else
    if(((ic32>>8)&0b111111111110000000001100)==0b101110001000000000000100) {
        names="ldrsw\0";
        i=((ic32>>20)&1?(-1<<9):0)|((ic32>>12)&0x1ff); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Xt; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_i_opt; 
    } else
    if(((ic32>>8)&0b111111111110000000000100)==0b101110001000000000000100) {
        names="ldrsw\0";
        i=((ic32>>20)&1?(-1<<9):0)|((ic32>>12)&0x1ff); p=((ic32>>11)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Xt; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_i_opt; args[4]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b111111111110000000001100)==0b101110001010000000001000) {
        names="ldrsw\0";
        m=((ic32>>16)&0x1f); o=((ic32>>13)&0x7); j=((ic32>>12)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Xt; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_Rom; args[4]=disasm_arg_amountj2; args[5]=disasm_arg_offe; 
    } else
    if(((ic32>>16)&0b1111111111000000)==0b1011100110000000) {
        names="ldrsw\0";
        j=((ic32>>10)&0xfff); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Xt; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_j_opt; args[4]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b111111111100000010000000)==0b110011100000000000000000) {
        names="eor3\0bcax\0";
        op=((ic32>>21)&0x1); m=((ic32>>16)&0x1f); d=((ic32>>10)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt16b; args[1]=disasm_arg_Vn16b; args[2]=disasm_arg_Vm16b; args[3]=disasm_arg_Vd16b; 
    } else
    if(((ic32>>8)&0b111111111110000010000000)==0b110011100100000000000000) {
        names="sm3ss1\0";
        m=((ic32>>16)&0x1f); d=((ic32>>10)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt4s; args[1]=disasm_arg_Vn4s; args[2]=disasm_arg_Vm4s; args[3]=disasm_arg_Vd4s; 
    } else
    if(((ic32>>8)&0b111111111110000011000000)==0b110011100100000010000000) {
        names="sm3tt1a\0sm3tt1b\0sm3tt2a\0sm3tt2b\0";
        op=((ic32>>10)&0x3); m=((ic32>>16)&0x1f); j=((ic32>>12)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        z=0;
        args[0]=disasm_arg_Vt4s; args[1]=disasm_arg_Vn4s; args[2]=disasm_arg_VmTs; 
    } else
    if(((ic32>>8)&0b111111111110000011111000)==0b110011100110000010000000) {
        names="sha512h\0sha512h2\0";
        op=((ic32>>10)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Qt; args[1]=disasm_arg_Qn; args[2]=disasm_arg_Vm2d; 
    } else
    if(((ic32>>8)&0b111111111110000011111000)==0b110011100110000010001000) {
        names="sha512su1\0rax1\0";
        op=((ic32>>10)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt2d; args[1]=disasm_arg_Vn2d; args[2]=disasm_arg_Vm2d; 
    } else
    if(((ic32>>8)&0b111111111110000011110000)==0b110011100110000011000000) {
        names="sm3partw1\0sm3partw2\0sm4ekey\0";
        op=((ic32>>10)&0x3); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt4s; args[1]=disasm_arg_Vn4s; args[2]=disasm_arg_Vm4s; 
    } else
    if(((ic32>>8)&0b111111111111111111111100)==0b110011101100000010000000) {
        names="sha512su0\0";
        n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt2d; args[1]=disasm_arg_Vn2d; 
    } else
    if(((ic32>>8)&0b111111111111111111111100)==0b110011101100000010000100) {
        names="sm4e\0";
        n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt4s; args[1]=disasm_arg_Vn4s; 
    } else
    if(((ic32>>16)&0b1111111111000000)==0b1101010000000000) {
        names="?\0svc\0hvc\0smc\0brk\0";
        op=((ic32>>19)&0x4)|((ic32>>0)&0x3); i=((ic32>>20)&1?(-1<<16):0)|((ic32>>5)&0xffff); 
        args[0]=disasm_arg_i; 
    } else
    if((ic32&0b11111111111000000000000000000011)==0b11010100010000000000000000000000) {
        names="hlt\0";
    } else
    if(((ic32>>16)&0b1111111111100000)==0b1101010010100000) {
        names="?\0dcsp1\0dcps2\0dcps3\0";
        op=((ic32>>0)&0x3); i=((ic32>>20)&1?(-1<<16):0)|((ic32>>5)&0xffff); 
        args[0]=disasm_arg_i_opt; 
    } else
    if((ic32&0b11111111111111111111110100011111)==0b11010101000000110010000000011111) {
        names="nop\0yield\0wfe\0wfi\0sev\0sevl\0?\0?\0esb\0psc\0";
        op=((ic32>>6)&0x8)|((ic32>>5)&0x7); 
    } else
    if((ic32&0b11111111111111111111000011111111)==0b11010101000000110011000001011111) {
        names="clrex\0";
        i=((ic32>>11)&1?(-1<<4):0)|((ic32>>8)&0xf); 
        args[0]=disasm_arg_i_opt; 
    } else
    if((ic32&0b11111111111111111111000010011111)==0b11010101000000110011000010011111) {
        names="dsb\0dmb\0?\0isb\0";
        op=((ic32>>5)&0x3); j=((ic32>>8)&0xf); 
        args[0]=disasm_arg_sh; 
    } else
    if((ic32&0b11111111111110001111000000011111)==0b11010101000000000100000000011111) {
        names="msr\0";
        i=((ic32>>11)&1?(-1<<4):0)|((ic32>>8)&0xf); p=((ic32>>5)&0x7); 
        args[0]=disasm_arg_pstate; args[1]=disasm_arg_i; 
    } else
    if((ic32&0b11111111111111111111111110000000)==0b11010101000010000111011000000000) {
        names="dc\0";
        d=((ic32>>5)&0x3); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_dc0; args[1]=disasm_arg_Xt; 
    } else
    if((ic32&0b11111111111111111111111110000000)==0b11010101000010000111100000000000) {
        names="at\0";
        a=((ic32>>5)&0x3); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_a0; args[1]=disasm_arg_Xt; 
    } else
    if(((ic32>>8)&0b111111111111111111111111)==0b110101010000100001111001) {
        names="at\0";
        a=((ic32>>5)&0x7); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_a1; args[1]=disasm_arg_Xt; 
    } else
    if((ic32&0b11111111111111111111101111100000)==0b11010101000010000111101001000000) {
        names="dc\0";
        d=((ic32>>10)&0x1); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_dc1; args[1]=disasm_arg_Xt; 
    } else
    if((ic32&0b11111111111111111111111111100000)==0b11010101000010110111010000100000) {
        names="dc\0";
        t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_ZVA; args[1]=disasm_arg_Xt; 
    } else
    if((ic32&0b11111111111111111111101011100000)==0b11010101000010110111101000100000) {
        names="dc\0";
        d=((ic32>>9)&0x2)|((ic32>>8)&0x1); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_dc2; args[1]=disasm_arg_Xt; 
    } else
    if((ic32&0b11111111111111001111101111000000)==0b11010101000010000111000100000000) {
        names="ic\0";
        c=((ic32>>15)&0x2)|((ic32>>10)&0x1); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_ic; args[1]=disasm_arg_Xt_opt; 
    } else
    if((ic32&0b11111111111111111111101101100000)==0b11010101000011001000000000100000) {
        names="tlbi\0";
        n=((ic32>>9)&0x2)|((ic32>>7)&0x1); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_tl1; args[1]=disasm_arg_Xt_opt; 
    } else
    if((ic32&0b11111111111111111111101101000000)==0b11010101000011101000001100000000) {
        names="tlbi\0";
        n=((ic32>>8)&0x4)|((ic32>>6)&0x2)|((ic32>>5)&0x1); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_tl2; args[1]=disasm_arg_Xt_opt; 
    } else
    if(((ic32>>8)&0b111111111111110111111111)==0b110101010000110001111000) {
        names="at\0";
        a=((ic32>>14)&0x8)|((ic32>>5)&0x7); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_a2; args[1]=disasm_arg_Xt; 
    } else
    if(((ic32>>8)&0b111111111111101111111011)==0b110101010000100010000011) {
        names="tlbi\0";
        n=((ic32>>14)&0x10)|((ic32>>7)&0x8)|((ic32>>5)&0x7); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_tl0; args[1]=disasm_arg_Xt_opt; 
    } else
    if(((ic32>>16)&0b1111111111100000)==0b1101010100000000) {
        names="msr\0";
        p=((ic32>>19)&0x3); k=((ic32>>16)&0x7); n=((ic32>>12)&0xf); m=((ic32>>8)&0xf); j=((ic32>>5)&0x7); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_sysreg; args[1]=disasm_arg_Xt; 
    } else
    if(((ic32>>16)&0b1111111111111000)==0b1101010100101000) {
        names="sysl\0";
        i=((ic32>>18)&1?(-1<<3):0)|((ic32>>16)&0x7); n=((ic32>>12)&0xf); m=((ic32>>8)&0xf); j=((ic32>>5)&0x7); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Xt; args[1]=disasm_arg_i; args[2]=disasm_arg_Cn; args[3]=disasm_arg_Cm; args[4]=disasm_arg_j; 
    } else
    if(((ic32>>16)&0b1111111111100000)==0b1101010100100000) {
        names="mrs\0";
        p=((ic32>>19)&0x3); k=((ic32>>16)&0x7); n=((ic32>>12)&0xf); m=((ic32>>8)&0xf); j=((ic32>>5)&0x7); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Xt; args[1]=disasm_arg_sysreg; 
    } else
    if((ic32&0b11111111100111111111110000011111)==0b11010110000111110000000000000000) {
        names="br\0blr\0ret\0";
        op=((ic32>>21)&0x3); n=((ic32>>5)&0x1f); 
        args[0]=disasm_arg_Xn; 
    } else
    if((ic32&0b11111111110111111111111111111111)==0b11010110100111110000001111100000) {
        names="eret\0drps\0";
        op=((ic32>>21)&0x1); 
    } else
    if(((ic32>>24)&0b11111111)==0b11011000) {
        names="prfm\0";
        i=((ic32>>23)&1?(-1<<19):0)|((ic32>>5)&0x7ffff); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_prf_op; args[1]=disasm_arg_labeli4; 
    } else
    if(((ic32>>8)&0b111111111110000000001100)==0b111110001000000000000000) {
        names="prfum\0";
        i=((ic32>>20)&1?(-1<<9):0)|((ic32>>12)&0x1ff); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_prf_op; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_i_opt; args[4]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b111111111110000000001100)==0b111110001010000000001000) {
        names="prfm\0";
        m=((ic32>>16)&0x1f); o=((ic32>>13)&0x7); j=((ic32>>12)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_prf_op; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_Rom; args[4]=disasm_arg_amountj3; args[5]=disasm_arg_offe; 
    } else
    if(((ic32>>16)&0b1111111111000000)==0b1111100110000000) {
        names="prfm\0";
        j=((ic32>>10)&0xfff); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_prf_op; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_j_opt; args[4]=disasm_arg_offe; 
    } else
    if(((ic32>>16)&0b1011111111100000)==0b1000100000100000) {
        names="stxp\0stlxp\0";
        op=((ic32>>15)&0x1); s=((ic32>>30)&0x1); d=((ic32>>16)&0x1f); m=((ic32>>10)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Wd; args[1]=disasm_arg_Rt; args[2]=disasm_arg_Rm; args[3]=disasm_arg_offs; args[4]=disasm_arg_XnS; args[5]=disasm_arg_offe; 
    } else
    if(((ic32>>16)&0b1011111111111111)==0b1000100001111111) {
        names="ldxp\0ldaxp\0";
        op=((ic32>>15)&0x1); s=((ic32>>30)&0x1); m=((ic32>>10)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Rt; args[1]=disasm_arg_Rm; args[2]=disasm_arg_offs; args[3]=disasm_arg_XnS; args[4]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111111010000001111100)==0b100010000010000001111100) {
        names="cas\0casl\0casa\0casal\0";
        op=((ic32>>21)&0x2)|((ic32>>15)&0x1); s=((ic32>>30)&0x1); d=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Rd; args[1]=disasm_arg_Rt; args[2]=disasm_arg_offs; args[3]=disasm_arg_XnS; args[4]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111110011111101111100)==0b100010000001111101111100) {
        names="?\0?\0ldxr\0ldaxr\0stllr\0stlr\0ldlar\0ldar\0";
        op=((ic32>>21)&0x6)|((ic32>>15)&0x1); s=((ic32>>30)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Rt; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111111010000000001100)==0b101110000000000000000100) {
        names="str\0ldr\0";
        op=((ic32>>22)&0x1); s=((ic32>>30)&0x1); i=((ic32>>20)&1?(-1<<9):0)|((ic32>>12)&0x1ff); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Rt; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_i_opt; 
    } else
    if(((ic32>>8)&0b101111111010000000000100)==0b101110000000000000000000) {
        names="stur\0sttr\0ldur\0ldtr\0";
        op=((ic32>>21)&0x2)|((ic32>>11)&0x1); s=((ic32>>30)&0x1); i=((ic32>>20)&1?(-1<<9):0)|((ic32>>12)&0x1ff); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Rt; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_i_opt; args[4]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111111010000000000100)==0b101110000000000000000100) {
        names="str\0ldr\0";
        op=((ic32>>22)&0x1); s=((ic32>>30)&0x1); i=((ic32>>20)&1?(-1<<9):0)|((ic32>>12)&0x1ff); p=((ic32>>11)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Rt; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_i_opt; args[4]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111111010000000001100)==0b101110000010000000001000) {
        names="str\0ldr\0";
        op=((ic32>>22)&0x1); s=((ic32>>30)&0x1); m=((ic32>>16)&0x1f); o=((ic32>>13)&0x7); j=((ic32>>12)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Rt; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_Rom; args[4]=disasm_arg_amountjs; args[5]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b101111110010000000001100)==0b101110000010000000000000) {
        names="ldadd\0ldclr\0ldeor\0ldset\0ldsmax\0ldsmin\0ldumax\0ldumin\0swp\0?\0?\0?\0?\0?\0?\0?\0ldaddl\0ldclrl\0ldeorl\0ldsetl\0ldsmaxl\0ldsminl\0ldumaxl\0lduminl\0swpl\0?\0?\0?\0?\0?\0?\0?\0ldadda\0ldclra\0ldeora\0ldseta\0ldsmaxa\0ldsmina\0ldumaxa\0ldumina\0swpa\0?\0?\0?\0?\0?\0?\0?\0ldaddal\0ldclral\0ldeoral\0ldsetal\0ldsmaxal\0ldsminal\0ldumaxal\0lduminal\0swpal\0";
        op=((ic32>>18)&0x30)|((ic32>>12)&0xf); s=((ic32>>30)&0x1); d=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Rd; args[1]=disasm_arg_Rt; args[2]=disasm_arg_offs; args[3]=disasm_arg_XnS; args[4]=disasm_arg_offe; 
    } else
    if(((ic32>>16)&0b1011111110000000)==0b1011100100000000) {
        names="str\0ldr\0";
        op=((ic32>>22)&0x1); s=((ic32>>30)&0x1); j=((ic32>>10)&0xfff); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Rt; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_j_opt; args[4]=disasm_arg_offe; 
    } else
    if(((ic32>>16)&0b0111111110100000)==0b0001001110000000) {
        names="extr\0";
        s=((ic32>>31)&0x1); m=((ic32>>16)&0x1f); i=((ic32>>15)&1?(-1<<6):0)|((ic32>>10)&0x3f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Rt; args[1]=disasm_arg_Rn; args[2]=disasm_arg_Rm; args[3]=disasm_arg_i; 
    } else
    if(((ic32>>24)&0b01111100)==0b00010100) {
        names="b\0bl\0";
        op=((ic32>>31)&0x1); i=((ic32>>25)&1?(-1<<26):0)|((ic32>>0)&0x3ffffff); 
        args[0]=disasm_arg_labeli4; 
    } else
    if(((ic32>>8)&0b011111111110000011111100)==0b000110100000000000000000) {
        names="adc\0";
        s=((ic32>>31)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Rt; args[1]=disasm_arg_Rn; args[2]=disasm_arg_Rm; 
    } else
    if(((ic32>>8)&0b011111111110000010000000)==0b000110101100000000000000) {
        names="?\0?\0udiv\0sdiv\0?\0?\0?\0?\0lslv\0lsrv\0asrv\0rorv\0?\0?\0?\0?\0crc32b\0crc32h\0crc32w\0crc32x\0crc32cb\0crc32ch\0crc32cw\0crc32cx\0";
        op=((ic32>>10)&0x1f); s=((ic32>>31)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Rt; args[1]=disasm_arg_Rn; args[2]=disasm_arg_Rm; 
    } else
    if(((ic32>>8)&0b011111111110000001111100)==0b000110110000000001111100) {
        names="mul\0mneg\0";
        op=((ic32>>15)&0x1); s=((ic32>>31)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Rt; args[1]=disasm_arg_Rn; args[2]=disasm_arg_Rm; 
    } else
    if(((ic32>>16)&0b0111111111100000)==0b0001101100000000) {
        names="madd\0msub\0";
        op=((ic32>>15)&0x1); s=((ic32>>31)&0x1); m=((ic32>>16)&0x1f); d=((ic32>>10)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Rt; args[1]=disasm_arg_Rn; args[2]=disasm_arg_Rm; args[3]=disasm_arg_Rd; 
    } else
    if(((ic32>>8)&0b011111111111111111111100)==0b000111101010111000000000) {
        names="fmov\0";
        s=((ic32>>31)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Rt; args[1]=disasm_arg_Vn1d; 
    } else
    if(((ic32>>8)&0b011111111111111111111100)==0b000111101010111100000000) {
        names="fmov\0";
        s=((ic32>>31)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Vt1d; args[1]=disasm_arg_Rn; 
    } else
    if(((ic32>>16)&0b0111111100111110)==0b0001111000000010) {
        names="scvtf\0ucvtf\0";
        op=((ic32>>16)&0x1); s=((ic32>>31)&0x1); z=((ic32>>22)&0x3); j=((ic32>>10)&0x3f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPz5t; args[1]=disasm_arg_Rn; args[2]=disasm_arg_fbits; 
    } else
    if(((ic32>>16)&0b0111111100111110)==0b0001111000011000) {
        names="fcvtzs\0fcvtzu\0";
        op=((ic32>>16)&0x1); s=((ic32>>31)&0x1); z=((ic32>>22)&0x3); j=((ic32>>10)&0x3f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Rt; args[1]=disasm_arg_FPz5n; args[2]=disasm_arg_fbits; 
    } else
    if(((ic32>>8)&0b011111110011101011111100)==0b000111100010001000000000) {
        names="scvtf\0ucvtf\0fmov\0fmov\0";
        op=((ic32>>17)&0x2)|((ic32>>16)&0x1); s=((ic32>>31)&0x1); z=((ic32>>22)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPz5t; args[1]=disasm_arg_Rn; 
    } else
    if(((ic32>>8)&0b011111110011000011111100)==0b000111100010000000000000) {
        names="fcvtns\0fcvtnu\0scvtf\0ucvtf\0fcvtas\0fcvtau\0fmov\0fmov\0fcvtns\0fcvtnu\0";
        op=((ic32>>16)&0xf); s=((ic32>>31)&0x1); z=((ic32>>22)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Rt; args[1]=disasm_arg_FPz5n; 
    } else
    if(((ic32>>8)&0b011111110011111011111100)==0b000111100011000000000000) {
        names="fcvtms\0fcvtmu\0";
        op=((ic32>>16)&0x1); s=((ic32>>31)&0x1); z=((ic32>>22)&0x3); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Rt; args[1]=disasm_arg_FPz5n; 
    } else
    if(((ic32>>16)&0b0111111110000000)==0b0010100010000000) {
        names="stp\0ldp\0";
        op=((ic32>>22)&0x1); s=((ic32>>31)&0x1); i=((ic32>>21)&1?(-1<<7):0)|((ic32>>15)&0x7f); m=((ic32>>10)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Rt; args[1]=disasm_arg_Rm; args[2]=disasm_arg_offs; args[3]=disasm_arg_XnS; args[4]=disasm_arg_offe; args[5]=disasm_arg_is4_opt; 
    } else
    if(((ic32>>24)&0b01111110)==0b00101000) {
        names="stnp\0ldnp\0stp\0ldp\0";
        op=((ic32>>23)&0x2)|((ic32>>22)&0x1); s=((ic32>>31)&0x1); p=((ic32>>23)&0x1); i=((ic32>>21)&1?(-1<<7):0)|((ic32>>15)&0x7f); m=((ic32>>10)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Rt; args[1]=disasm_arg_Rm; args[2]=disasm_arg_offs; args[3]=disasm_arg_XnS; args[4]=disasm_arg_is4_opt; args[5]=disasm_arg_offe; 
    } else
    if(((ic32>>24)&0b01111110)==0b00110100) {
        names="cbz\0cbnz\0";
        op=((ic32>>24)&0x1); s=((ic32>>31)&0x1); i=((ic32>>23)&1?(-1<<19):0)|((ic32>>5)&0x7ffff); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Rt; args[1]=disasm_arg_labeli4; 
    } else
    if(((ic32>>24)&0b01111110)==0b00110110) {
        names="tbz\0tbnz\0";
        op=((ic32>>24)&0x1); b=((ic32>>26)&0x20)|((ic32>>19)&0x1f); i=((ic32>>18)&1?(-1<<14):0)|((ic32>>5)&0x3fff); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Xt; args[1]=disasm_arg_b; args[2]=disasm_arg_labeli4; 
    } else
    if(((ic32>>8)&0b011111111110000000000100)==0b001110001000000000000000) {
        names="?\0ldtrsb\0ldursw\0ldtrsw\0";
        op=((ic32>>30)&0x2)|((ic32>>11)&0x1); i=((ic32>>20)&1?(-1<<9):0)|((ic32>>12)&0x1ff); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Xt; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_i_opt; args[4]=disasm_arg_offe; 
    } else
    if((ic32&0b01111111111000001111111111100000)==0b01011010000000000000001111100000) {
        names="ngc\0";
        s=((ic32>>31)&0x1); m=((ic32>>16)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Rt; args[1]=disasm_arg_Rm; 
    } else
    if(((ic32>>8)&0b011111111111111111111000)==0b010110101100000000001000) {
        names="rev\0";
        s=((ic32>>31)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Rt; args[1]=disasm_arg_Rn; 
    } else
    if(((ic32>>8)&0b011111111111111111101000)==0b010110101100000000000000) {
        names="rbit\0rev16\0clz\0cls\0";
        op=((ic32>>11)&0x2)|((ic32>>10)&0x1); s=((ic32>>31)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Rt; args[1]=disasm_arg_Rn; 
    } else
    if(((ic32>>8)&0b001111111110000000001000)==0b000110101000000000000000) {
        names="csel\0csinc\0csinv\0csneg\0";
        op=((ic32>>29)&0x2)|((ic32>>10)&0x1); s=((ic32>>31)&0x1); m=((ic32>>16)&0x1f); c=((ic32>>12)&0xf); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Rt; args[1]=disasm_arg_Rn; args[2]=disasm_arg_Rm; args[3]=disasm_arg_c; 
    } else
    if(((ic32>>24)&0b00111111)==0b00011100) {
        names="ldr\0";
        z=((ic32>>30)&0x3); i=((ic32>>23)&1?(-1<<19):0)|((ic32>>5)&0x7ffff); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPt; args[1]=disasm_arg_labeli4; 
    } else
    if((ic32&0b00111111111000000000000000011111)==0b00101011001000000000000000011111) {
        names="cmn\0cmp\0";
        op=((ic32>>30)&0x1); s=((ic32>>31)&0x1); m=((ic32>>16)&0x1f); o=((ic32>>13)&0x7); j=((ic32>>10)&0x7); n=((ic32>>5)&0x1f); 
        args[0]=disasm_arg_RnS; args[1]=disasm_arg_Rsom; args[2]=disasm_arg_exts; 
    } else
    if(((ic32>>16)&0b0011111110000000)==0b0010110010000000) {
        names="stp\0ldp\0";
        op=((ic32>>22)&0x1); z=((ic32>>30)&0x3); i=((ic32>>21)&1?(-1<<7):0)|((ic32>>15)&0x7f); m=((ic32>>10)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPt; args[1]=disasm_arg_FPm; args[2]=disasm_arg_offs; args[3]=disasm_arg_XnS; args[4]=disasm_arg_offe; args[5]=disasm_arg_iz4_opt; 
    } else
    if(((ic32>>24)&0b00111110)==0b00101100) {
        names="stnp\0ldnp\0stp\0ldp\0";
        op=((ic32>>23)&0x2)|((ic32>>22)&0x1); z=((ic32>>30)&0x3); p=((ic32>>23)&0x1); i=((ic32>>21)&1?(-1<<7):0)|((ic32>>15)&0x7f); m=((ic32>>10)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPt; args[1]=disasm_arg_FPm; args[2]=disasm_arg_offs; args[3]=disasm_arg_XnS; args[4]=disasm_arg_iz4_opt; args[5]=disasm_arg_offe; 
    } else
    if((ic32&0b00111111111000000000110000010000)==0b00111010010000000000000000000000) {
        names="ccmn\0ccmp\0";
        op=((ic32>>30)&0x1); s=((ic32>>31)&0x1); m=((ic32>>16)&0x1f); c=((ic32>>12)&0xf); n=((ic32>>5)&0x1f); j=((ic32>>0)&0xf); 
        args[0]=disasm_arg_Rn; args[1]=disasm_arg_Rm; args[2]=disasm_arg_j; args[3]=disasm_arg_c; 
    } else
    if((ic32&0b00111111111000000000110000010000)==0b00111010010000000000100000000000) {
        names="ccmn\0ccmp\0";
        op=((ic32>>30)&0x1); s=((ic32>>31)&0x1); b=((ic32>>16)&0x1f); c=((ic32>>12)&0xf); n=((ic32>>5)&0x1f); j=((ic32>>0)&0xf); 
        args[0]=disasm_arg_Rn; args[1]=disasm_arg_b; args[2]=disasm_arg_j; args[3]=disasm_arg_c; 
    } else
    if(((ic32>>8)&0b001111110010000000001100)==0b001111000000000000000000) {
        names="stur\0ldur\0";
        op=((ic32>>22)&0x1); z=((ic32>>30)&0x3); s=((ic32>>23)&0x1); i=((ic32>>20)&1?(-1<<9):0)|((ic32>>12)&0x1ff); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPst; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_i_opt; args[4]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b001111110010000000001100)==0b001111000000000000000100) {
        names="str\0ldr\0";
        op=((ic32>>22)&0x1); z=((ic32>>30)&0x3); s=((ic32>>23)&0x1); i=((ic32>>20)&1?(-1<<9):0)|((ic32>>12)&0x1ff); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPst; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_offe; args[4]=disasm_arg_i_opt; 
    } else
    if(((ic32>>8)&0b001111110010000000000100)==0b001111000000000000000100) {
        names="str\0ldr\0";
        op=((ic32>>22)&0x1); z=((ic32>>30)&0x3); s=((ic32>>23)&0x1); i=((ic32>>20)&1?(-1<<9):0)|((ic32>>12)&0x1ff); p=((ic32>>11)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPst; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_i_opt; args[4]=disasm_arg_offe; 
    } else
    if(((ic32>>8)&0b001111110010000000001100)==0b001111000010000000001000) {
        names="str\0ldr\0";
        op=((ic32>>22)&0x1); z=((ic32>>30)&0x3); s=((ic32>>23)&0x1); m=((ic32>>16)&0x1f); o=((ic32>>13)&0x7); j=((ic32>>12)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPst; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_Rom; args[4]=disasm_arg_amountz; args[5]=disasm_arg_offe; 
    } else
    if(((ic32>>24)&0b00111111)==0b00111101) {
        names="str\0ldr\0";
        op=((ic32>>22)&0x1); z=((ic32>>30)&0x3); s=((ic32>>23)&0x1); j=((ic32>>10)&0xfff); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_FPst; args[1]=disasm_arg_offs; args[2]=disasm_arg_XnS; args[3]=disasm_arg_j_opt; args[4]=disasm_arg_offe; 
    } else
    if(((ic32>>16)&0b0001111111100000)==0b0000101100100000) {
        names="add\0adds\0sub\0subs\0";
        op=((ic32>>29)&0x3); s=((ic32>>31)&0x1); m=((ic32>>16)&0x1f); o=((ic32>>13)&0x7); j=((ic32>>10)&0x7); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_RtS; args[1]=disasm_arg_RnS; args[2]=disasm_arg_Rsom; args[3]=disasm_arg_exts; 
    } else
    if(((ic32>>24)&0b00011110)==0b00001010) {
        names="and\0bic\0add\0?\0orr\0orn\0adds\0?\0eor\0eon\0sub\0?\0ands\0bics\0subs\0";
        op=((ic32>>27)&0xc)|((ic32>>23)&0x2)|((ic32>>21)&0x1); s=((ic32>>31)&0x1); z=((ic32>>22)&0x3); m=((ic32>>16)&0x1f); j=((ic32>>10)&0x3f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Rt; args[1]=disasm_arg_Rn; args[2]=disasm_arg_Rm; args[3]=disasm_arg_shiftj_opt; 
    } else
    if(((ic32>>24)&0b00011111)==0b00010000) {
        names="adr\0adrp\0";
        op=((ic32>>31)&0x1); j=((ic32>>29)&0x3); i=((ic32>>23)&1?(-1<<19):0)|((ic32>>5)&0x7ffff); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Xt; args[1]=disasm_arg_labelij1; 
    } else
    if(((ic32>>24)&0b00011111)==0b00010001) {
        names="add\0adds\0sub\0subs\0";
        op=((ic32>>29)&0x3); s=((ic32>>31)&0x1); j=((ic32>>22)&0x3); i=((ic32>>21)&1?(-1<<12):0)|((ic32>>10)&0xfff); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_RtS; args[1]=disasm_arg_RnS; args[2]=disasm_arg_i; args[3]=disasm_arg_j12_opt; 
    } else
    if(((ic32>>16)&0b0001111110000000)==0b0001001000000000) {
        names="and\0orr\0eor\0ands\0";
        op=((ic32>>29)&0x3); i=((ic32>>22)&1?(-1<<13):0)|((ic32>>10)&0x1000)|((ic32>>4)&0xfc0)|((ic32>>16)&0x3f); s=((ic32>>31)&0x1); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_RtS; args[1]=disasm_arg_Rn; args[2]=disasm_arg_i; 
    } else
    if(((ic32>>16)&0b0001111110000000)==0b0001001010000000) {
        names="movn\0?\0movz\0movk\0";
        op=((ic32>>29)&0x3); s=((ic32>>31)&0x1); j=((ic32>>21)&0x3); i=((ic32>>20)&1?(-1<<16):0)|((ic32>>5)&0xffff); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Rt; args[1]=disasm_arg_i; args[2]=disasm_arg_j16_opt; 
    } else
    if(((ic32>>16)&0b0001111110000000)==0b0001001100000000) {
        names="sbfm\0bfm\0ubfm\0";
        op=((ic32>>29)&0x3); s=((ic32>>31)&0x1); i=((ic32>>21)&1?(-1<<6):0)|((ic32>>16)&0x3f); j=((ic32>>10)&0x3f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Rt; args[1]=disasm_arg_Rn; args[2]=disasm_arg_i; args[3]=disasm_arg_j; 
    } else
    if(((ic32>>8)&0b000111111110000011111100)==0b000110100000000000000000) {
        names="adc\0adcs\0sbc\0sbcs\0";
        op=((ic32>>29)&0x3); s=((ic32>>31)&0x1); m=((ic32>>16)&0x1f); n=((ic32>>5)&0x1f); t=((ic32>>0)&0x1f); 
        args[0]=disasm_arg_Rt; args[1]=disasm_arg_Rn; args[2]=disasm_arg_Rm; 
    } else
        names=NULL;

    if(str!=NULL) {
        str+=sprintf(str,disasm_str(names,op),disasm_str(conds,c));
        if(str-olds<10)om=10-(str-olds);else om=1;for(op=0;op<om;op++) *str++=' ';
        for(op=0;op<sizeof(args) && args[op]!=disasm_arg_NONE;op++) {
            if(op&&args[op-1]!=disasm_arg_offs&&args[op]!=disasm_arg_offe) { *str++=','; *str++=' '; }
            switch(args[op]) {
                case disasm_arg_Xt: str+=sprintf(str,t==31?"xzr":"x%d", t); break;
                case disasm_arg_labelij1: str+=sprintf(str,"0x%lx", iaddr+(i<<2)+j); break;
                case disasm_arg_RtS: str+=sprintf(str,t==31?"%csp":"%c%d", (s?'x':'w'), t); break;
                case disasm_arg_RnS: str+=sprintf(str,n==31?"%csp":"%c%d", (s?'x':'w'), n); break;
                case disasm_arg_i: str+=sprintf(str,"#0x%x", i); break;
                case disasm_arg_j12_opt: str+=sprintf(str,!j?"":"lsl #%d", j*12); break;
                case disasm_arg_Rn: str+=sprintf(str,n==31?"%czr":"%c%d", (s?'x':'w'), n); break;
                case disasm_arg_Rt: str+=sprintf(str,t==31?"%czr":"%c%d", (s?'x':'w'), t); break;
                case disasm_arg_j16_opt: str+=sprintf(str,!j?"":"lsl #%d", j*16); break;
                case disasm_arg_j: str+=sprintf(str,"#0x%x", j); break;
                case disasm_arg_Rm: str+=sprintf(str,m==31?"%czr":"%c%d", (s?'x':'w'), m); break;
                case disasm_arg_c: str+=sprintf(str,"%s", disasm_str(conds,c)); break;
                case disasm_arg_labeli4: str+=sprintf(str,"0x%lx", iaddr+(i<<2)); break;
                case disasm_arg_i_opt: str+=sprintf(str,!i?"":"#0x%x", i); break;
                case disasm_arg_pstate: str+=sprintf(str,"%s", disasm_str(pstate,p)); break;
                case disasm_arg_sh: str+=sprintf(str,"%s", disasm_str(share,j)); break;
                case disasm_arg_a0: str+=sprintf(str,"%s", disasm_str(at_op0,a)); break;
                case disasm_arg_a1: str+=sprintf(str,"%s", disasm_str(at_op1,a)); break;
                case disasm_arg_a2: str+=sprintf(str,"%s", disasm_str(at_op2,a)); break;
                case disasm_arg_dc0: str+=sprintf(str,"%s", disasm_str(dc_op0,d)); break;
                case disasm_arg_dc1: str+=sprintf(str,"%s", disasm_str(dc_op1,d)); break;
                case disasm_arg_ZVA: str+=sprintf(str,"ZVA"); break;
                case disasm_arg_dc2: str+=sprintf(str,"%s", disasm_str(dc_op2,d)); break;
                case disasm_arg_ic: str+=sprintf(str,"%s", disasm_str(ic_op,c)); break;
                case disasm_arg_Xt_opt: str+=sprintf(str,t==31?"":"x%d", t); break;
                case disasm_arg_tl0: str+=sprintf(str,"%s", disasm_str(tlbi_op0,n)); break;
                case disasm_arg_tl1: str+=sprintf(str,"%s", disasm_str(tlbi_op1,n)); break;
                case disasm_arg_tl2: str+=sprintf(str,"%s", disasm_str(tlbi_op2,n)); break;
                case disasm_arg_sysreg: str+=sprintf(str,disasm_sysreg(p,k,n,m,j)?disasm_sysreg(p,k,n,m,j):"S%d_%d_%d_%d_%d", p,k,n,m,j); break;
                case disasm_arg_Cn: str+=sprintf(str,"C%d", n); break;
                case disasm_arg_Cm: str+=sprintf(str,"C%d", m); break;
                case disasm_arg_Xn: str+=sprintf(str,n==31?"xzr":"x%d", n); break;
                case disasm_arg_b: str+=sprintf(str,"#0x%x", b); break;
                case disasm_arg_VtT: str+=sprintf(str,"%s", "V%d.%s", t, disasm_str(quantum,(z<<1)|q)); break;
                case disasm_arg_Vt2T: str+=sprintf(str,"%s", "V%d.%s", (t+1)&0x1f, disasm_str(quantum,(z<<1)|q)); break;
                case disasm_arg_Vt3T: str+=sprintf(str,"%s", "V%d.%s", (t+2)&0x1f, disasm_str(quantum,(z<<1)|q)); break;
                case disasm_arg_Vt4T: str+=sprintf(str,"%s", "V%d.%s", (t+3)&0x1f, disasm_str(quantum,(z<<1)|q)); break;
                case disasm_arg_offs: str+=sprintf(str,"["); break;
                case disasm_arg_XnS: str+=sprintf(str,n==31?"xsp":"x%d", n); break;
                case disasm_arg_offe: str+=sprintf(str,"]%s", p?"!":""); break;
                case disasm_arg_Qi: str+=sprintf(str,"#%d", q?64:32); break;
                case disasm_arg_Xm: str+=sprintf(str,m==31?"xzr":"x%d", m); break;
                case disasm_arg_Qi3: str+=sprintf(str,"#%d", q?48:24); break;
                case disasm_arg_Qi2: str+=sprintf(str,"#%d", q?32:16); break;
                case disasm_arg_Qi1: str+=sprintf(str,"#%d", q?16:8); break;
                case disasm_arg_VtB: str+=sprintf(str,"V%d.b[%d]", t, (q<<3)|(s<<2)|z); break;
                case disasm_arg_VtH: str+=sprintf(str,"V%d.h[%d]", t, (q<<3)|(s<<2)|z); break;
                case disasm_arg_VtS: str+=sprintf(str,"V%d.s[%d]", t, (q<<1)|s); break;
                case disasm_arg_VtD: str+=sprintf(str,"V%d.d[%d]", t, q); break;
                case disasm_arg_i1: str+=sprintf(str,"1"); break;
                case disasm_arg_i2: str+=sprintf(str,"2"); break;
                case disasm_arg_i4: str+=sprintf(str,"4"); break;
                case disasm_arg_i8: str+=sprintf(str,"8"); break;
                case disasm_arg_Vt3B: str+=sprintf(str,"V%d.b V%d.b V%d.b[%d]", t, (t+1)&0x1f, (t+2)&0x1f, (q<<3)|(s<<2)|z); break;
                case disasm_arg_Vt3H: str+=sprintf(str,"V%d.h V%d.h V%d.h[%d]", t, (t+1)&0x1f, (t+2)&0x1f, (q<<3)|(s<<2)|z); break;
                case disasm_arg_Vt3S: str+=sprintf(str,"V%d.s V%d.s V%d.s[%d]", t, (t+1)&0x1f, (t+2)&0x1f, (q<<1)|s); break;
                case disasm_arg_Vt3D: str+=sprintf(str,"V%d.d V%d.d V%d.d[%d]", t, (t+1)&0x1f, (t+2)&0x1f, q); break;
                case disasm_arg_i3: str+=sprintf(str,"3"); break;
                case disasm_arg_i6: str+=sprintf(str,"6"); break;
                case disasm_arg_i12: str+=sprintf(str,"12"); break;
                case disasm_arg_i24: str+=sprintf(str,"24"); break;
                case disasm_arg_Vt2B: str+=sprintf(str,"V%d.b V%d.b[%d]", t, (t+1)&0x1f, (q<<3)|(s<<2)|z); break;
                case disasm_arg_Vt2H: str+=sprintf(str,"V%d.h V%d.h[%d]", t, (t+1)&0x1f, (q<<3)|(s<<2)|z); break;
                case disasm_arg_Vt2S: str+=sprintf(str,"V%d.s V%d.s[%d]", t, (t+1)&0x1f, (q<<1)|s); break;
                case disasm_arg_Vt2D: str+=sprintf(str,"V%d.d V%d.d[%d]", t, (t+1)&0x1f, q); break;
                case disasm_arg_i16: str+=sprintf(str,"16"); break;
                case disasm_arg_Vt4B: str+=sprintf(str,"V%d.b V%d.b V%d.b V%d.b[%d]", t, (t+1)&0x1f, (t+2)&0x1f, (t+3)&0x1f, (q<<3)|(s<<2)|z); break;
                case disasm_arg_Vt4H: str+=sprintf(str,"V%d.h V%d.h V%d.h V%d.h[%d]", t, (t+1)&0x1f, (t+2)&0x1f, (t+3)&0x1f, (q<<3)|(s<<2)|z); break;
                case disasm_arg_Vt4S: str+=sprintf(str,"V%d.s V%d.s V%d.s V%d.s[%d]", t, (t+1)&0x1f, (t+2)&0x1f, (t+3)&0x1f, (q<<1)|s); break;
                case disasm_arg_Vt4D: str+=sprintf(str,"V%d.d V%d.d V%d.d V%d.d[%d]", t, (t+1)&0x1f, (t+2)&0x1f, (t+3)&0x1f, q); break;
                case disasm_arg_i32: str+=sprintf(str,"32"); break;
                case disasm_arg_z: str+=sprintf(str,"#%d", 1<<z); break;
                case disasm_arg_z3: str+=sprintf(str,"#%d", 3<<z); break;
                case disasm_arg_z2: str+=sprintf(str,"#%d", 2<<z); break;
                case disasm_arg_z4: str+=sprintf(str,"#%d", 4<<z); break;
                case disasm_arg_Rd: str+=sprintf(str,d==31?"%czr":"%c%d", (s?'x':'w'), d); break;
                case disasm_arg_Rd1: str+=sprintf(str,d+1==31?"%czr":"%c%d", (s?'x':'w'), (d+1)&0x1f); break;
                case disasm_arg_Rt1: str+=sprintf(str,t+1==31?"%czr":"%c%d", (s?'x':'w'), (t+1)&0x1f); break;
                case disasm_arg_Wd: str+=sprintf(str,d==31?"wzr":"w%d", d); break;
                case disasm_arg_Wt: str+=sprintf(str,t==31?"wzr":"w%d", t); break;
                case disasm_arg_FPt: str+=sprintf(str,"%c%d", z==2?'q':(z==1?'d':'s'), t); break;
                case disasm_arg_prf_op: str+=sprintf(str,"%s", "%s L%d %s", disasm_str(prf_typ,(t>>3)&3), ((t>>1)&3)+1, disasm_str(prf_pol,t&1)); break;
                case disasm_arg_is4_opt: str+=sprintf(str,!i?"":"#0x%x", i<<(2+s)); break;
                case disasm_arg_FPm: str+=sprintf(str,"%c%d", z==2?'q':(z==1?'d':'s'), m); break;
                case disasm_arg_iz4_opt: str+=sprintf(str,!i?"":"#0x%x", i<<(2+z)); break;
                case disasm_arg_im4_opt: str+=sprintf(str,!i?"":"#0x%x", i<<2); break;
                case disasm_arg_nRt: str+=sprintf(str,t==31?"%czr":"%c%d", (s?'w':'x'), t); break;
                case disasm_arg_FPst: str+=sprintf(str,"%c%d", s==1?'q':(z==3?'d':(z==2?'s':(z==1?'h':'b'))), t); break;
                case disasm_arg_j_opt: str+=sprintf(str,!j?"":"#0x%x", j); break;
                case disasm_arg_Rom: str+=sprintf(str,m==31?"%czr":"%c%d", (o&1?'x':'w'), m); break;
                case disasm_arg_amountj: str+=sprintf(str,"%s", "%s #%d", disasm_str(extend64,o), j); break;
                case disasm_arg_amountz: str+=sprintf(str,"%s", "%s #%d", disasm_str(extend64,o), j?(s?4:z):0); break;
                case disasm_arg_amountjs: str+=sprintf(str,"%s", "%s #%d", disasm_str(extend64,o), j?(s?3:2):0); break;
                case disasm_arg_amountj2: str+=sprintf(str,"%s", "%s #%d", disasm_str(extend64,o), j?2:0); break;
                case disasm_arg_amountj3: str+=sprintf(str,"%s", "%s #%d", disasm_str(extend64,o), j?3:0); break;
                case disasm_arg_shiftj_opt: str+=sprintf(str,"%s", !j?"":"%s #%d", disasm_str(shift,z), j); break;
                case disasm_arg_Rsom: str+=sprintf(str,m==31?"%czr":"%c%d", (s&&(o&3)==3?'x':'w'), m); break;
                case disasm_arg_exts: str+=sprintf(str,"%s", "%s #%d", s?disasm_str(extend64,o):disasm_str(extend32,o), j); break;
                case disasm_arg_Wn: str+=sprintf(str,n==31?"wzr":"w%d", n); break;
                case disasm_arg_Wm: str+=sprintf(str,m==31?"wzr":"w%d", m); break;
                case disasm_arg_Xd: str+=sprintf(str,d==31?"xzr":"x%d", d); break;
                case disasm_arg_Vt16b: str+=sprintf(str,"V%d.16b", t); break;
                case disasm_arg_Vn16b: str+=sprintf(str,"V%d.16b", n); break;
                case disasm_arg_Qt: str+=sprintf(str,"q%d", t); break;
                case disasm_arg_Sn: str+=sprintf(str,"s%d", n); break;
                case disasm_arg_Vm4s: str+=sprintf(str,"V%d.4s", m); break;
                case disasm_arg_Vt4s: str+=sprintf(str,"V%d.4s", t); break;
                case disasm_arg_Vn4s: str+=sprintf(str,"V%d.4s", n); break;
                case disasm_arg_Qn: str+=sprintf(str,"q%d", n); break;
                case disasm_arg_St: str+=sprintf(str,"s%d", t); break;
                case disasm_arg_FPjt: str+=sprintf(str,"%c%d", j&1?'b':((j&3)==2?'h':((j&7)==4?'s':'d')), t); break;
                case disasm_arg_Vnj: str+=sprintf(str,"V%d.%c", n, j&1?'b':((j&3)==2?'h':((j&7)==4?'s':'d'))); break;
                case disasm_arg_FPidx: str+=sprintf(str,"%d", j>>(j&1?1:((j&3)==2?2:((j&7)==4?3:4))), t); break;
                case disasm_arg_Vtjq: str+=sprintf(str,"%s", "V%d.%s", t, disasm_str(quantum,(j&1?0:((j&3)==2?2:(j&7)==4?4:6))+q)); break;
                case disasm_arg_Ht: str+=sprintf(str,"h%d", t); break;
                case disasm_arg_Hn: str+=sprintf(str,"h%d", n); break;
                case disasm_arg_Hm: str+=sprintf(str,"h%d", m); break;
                case disasm_arg_FPn: str+=sprintf(str,"%c%d", z==2?'q':(z==1?'d':'s'), n); break;
                case disasm_arg_VtH1: str+=sprintf(str,"V%d.%dh", t, q?8:4); break;
                case disasm_arg_VnH1: str+=sprintf(str,"V%d.%dh", n, q?8:4); break;
                case disasm_arg_VmH1: str+=sprintf(str,"V%d.%dh", m, q?8:4); break;
                case disasm_arg_Vtzq: str+=sprintf(str,"%s", "V%d.%s", t, disasm_str(quantum,4+(z*2)+q)); break;
                case disasm_arg_Vnzq: str+=sprintf(str,"%s", "V%d.%s", n, disasm_str(quantum,4+(z*2)+q)); break;
                case disasm_arg_Vmzq: str+=sprintf(str,"%s", "V%d.%s", m, disasm_str(quantum,4+(z*2)+q)); break;
                case disasm_arg_simd0: str+=sprintf(str,"#0.0"); break;
                case disasm_arg_FPz2t: str+=sprintf(str,"%c%d", z==1?'h':'s', t); break;
                case disasm_arg_FPz2n: str+=sprintf(str,"%c%d", z==1?'h':'s', n); break;
                case disasm_arg_FPz2m: str+=sprintf(str,"%c%d", z==1?'h':'s', m); break;
                case disasm_arg_VnT: str+=sprintf(str,"%s", "V%d.%s", n, disasm_str(quantum,(z<<1)|q)); break;
                case disasm_arg_VmT: str+=sprintf(str,"%s", "V%d.%s", m, disasm_str(quantum,(z<<1)|q)); break;
                case disasm_arg_FPz3t: str+=sprintf(str,"%c%d", z==3?'d':(z==2?'s':(z==1?'h':'b')), t); break;
                case disasm_arg_FPz3n: str+=sprintf(str,"%c%d", z==3?'d':(z==2?'s':(z==1?'h':'b')), n); break;
                case disasm_arg_FPz4n: str+=sprintf(str,"%c%d", z==2?'d':(z==1?'s':'h'), n); break;
                case disasm_arg_VnT3: str+=sprintf(str,"%s", "V%d.%s", n, disasm_str(quantum,(z<<1)+3)); break;
                case disasm_arg_Vn2d: str+=sprintf(str,"V%d.2d", n); break;
                case disasm_arg_Vn2h: str+=sprintf(str,"V%d.2h", n); break;
                case disasm_arg_Vnz: str+=sprintf(str,"V%d.2%c", n, z?'d':'s'); break;
                case disasm_arg_FPz4t: str+=sprintf(str,"%c%d", z==2?'d':(z==1?'s':'h'), t); break;
                case disasm_arg_Vtz: str+=sprintf(str,"%s", "V%d.%s", t, disasm_str(quantum,4+(z*2))); break;
                case disasm_arg_FPz3m: str+=sprintf(str,"%c%d", z==3?'d':(z==2?'s':(z==1?'h':'b')), m); break;
                case disasm_arg_Dt: str+=sprintf(str,"d%d", t); break;
                case disasm_arg_Dn: str+=sprintf(str,"d%d", n); break;
                case disasm_arg_shrshift: str+=sprintf(str,"#%d", ((j>>3)==1?16:((j>>4)==1?32:((j>>5)==1?64:128)))-j); break;
                case disasm_arg_Vtj2: str+=sprintf(str,"%s", "V%d.%s", t, disasm_str(quantum,((j>>3)==1?0:((j>>4)==1?2:((j>>5)==1?4:6)))|q)); break;
                case disasm_arg_Vnj2: str+=sprintf(str,"%s", "V%d.%s", n, disasm_str(quantum,((j>>3)==1?0:((j>>4)==1?2:((j>>5)==1?4:6)))|q)); break;
                case disasm_arg_shlshift: str+=sprintf(str,"#%d", j-((j>>3)==1?8:((j>>4)==1?16:((j>>5)==1?32:64)))); break;
                case disasm_arg_FPnj: str+=sprintf(str,"%c%d", (j>>3)==1?'h':((j>>4)==1?'s':'d'), n); break;
                case disasm_arg_VnTa: str+=sprintf(str,"%s", "V%d.%s", n, disasm_str(quantum,((j>>3)==1?3:((j>>4)==1?4:7)))); break;
                case disasm_arg_FPjt2: str+=sprintf(str,"%c%d", (j>>3)==1?'b':((j>>4)==1?'h':((j>>5)==1?'s':'d')), t); break;
                case disasm_arg_FPjn2: str+=sprintf(str,"%c%d", (j>>3)==1?'b':((j>>4)==1?'h':((j>>5)==1?'s':'d')), n); break;
                case disasm_arg_Vtz3: str+=sprintf(str,"%s", "V%d.%s", t, disasm_str(quantum,(z<<1)+6)); break;
                case disasm_arg_VmTs: str+=sprintf(str,"V%d.%c[%d]", m, z==1?'h':'s', j); break;
                case disasm_arg_VmHs: str+=sprintf(str,"V%d.h[%d]", m, j); break;
                case disasm_arg_VmTs2: str+=sprintf(str,"V%d.%c[%d]", m, z==1?'d':'s', j); break;
                case disasm_arg_Vn116b: str+=sprintf(str,"{ V%d.16b }", n); break;
                case disasm_arg_Vn216b: str+=sprintf(str,"{ V%d.16b, V%d.16b }", n, (n+1)&0x1f); break;
                case disasm_arg_Vn316b: str+=sprintf(str,"{ V%d.16b, V%d.16b, V%d.16b }", n, (n+1)&0x1f, (n+2)&0x1f); break;
                case disasm_arg_Vn416b: str+=sprintf(str,"{ V%d.16b, V%d.16b, V%d.16b, V%d.16b }", n, (n+1)&0x1f, (n+2)&0x1f, (n+3)&0x1f); break;
                case disasm_arg_Vtj: str+=sprintf(str,"V%d.%c", t, j&1?'b':((j&3)==2?'h':((j&7)==4?'s':'d'))); break;
                case disasm_arg_R2n: str+=sprintf(str,n==31?"%czr":"%c%d", ((j&15)==8?'x':'w'), n); break;
                case disasm_arg_FPidxk: str+=sprintf(str,"%d", k>>(k&1?1:((k&3)==2?2:((k&7)==4?3:4))), t); break;
                case disasm_arg_Vtzq2: str+=sprintf(str,"%s", "V%d.%s", t, disasm_str(quantum,2+(z*2)+q)); break;
                case disasm_arg_VnT2: str+=sprintf(str,"%s", "V%d.%s", n, disasm_str(quantum,z+3)); break;
                case disasm_arg_Vnz3: str+=sprintf(str,"%s", "V%d.%s", n, disasm_str(quantum,(z<<1)+6)); break;
                case disasm_arg_Vnzq2: str+=sprintf(str,"%s", "V%d.%s", n, disasm_str(quantum,2+(z*2)+q)); break;
                case disasm_arg_shift8: str+=sprintf(str,"#%d", 1<<(z+3)); break;
                case disasm_arg_VtT3: str+=sprintf(str,"%s", "V%d.%s", t, disasm_str(quantum,(z<<1)+3)); break;
                case disasm_arg_VmT3: str+=sprintf(str,"%s", "V%d.%s", m, disasm_str(quantum,(z<<1)+3)); break;
                case disasm_arg_VtT4: str+=sprintf(str,"%s", "V%d.%s", t, disasm_str(quantum,z?8:3)); break;
                case disasm_arg_imm8: str+=sprintf(str,"#%x", j); break;
                case disasm_arg_amountk_opt: str+=sprintf(str,!k?"":"lsl #%d", 1<<(k*3)); break;
                case disasm_arg_amountk2_opt: str+=sprintf(str,!k?"":"msl #%d", 1<<(k*3)); break;
                case disasm_arg_imm64: str+=sprintf(str,"#0x%02x%02x%02x%02x%02x%02x%02x%02x", j&128?255:0,j&64?255:0,j&32?255:0,j&16?255:0,j&8?255:0,j&4?255:0,j&2?255:0,j&1?255:0); break;
                case disasm_arg_Vt2d: str+=sprintf(str,"V%d.2d", t); break;
                case disasm_arg_F16: str+=sprintf(str,"#0x02x%02x", (j&128)|(j&64?0:64)|(j&64?32:0)|(j&64?16:0)|((j>>2)&0xF), (j&3)<<6); break;
                case disasm_arg_F32: str+=sprintf(str,"#0x02x%02x0000", (j&128)|(j&64?0:64)|(j&64?32:0)|(j&64?16:0)|(j&64?8:0)|(j&64?4:0)|(j&64?2:0)|(j&32?1:0), (j&0x1f)<<3); break;
                case disasm_arg_F64: str+=sprintf(str,"#0x02x%02x%06x", (j&128)|(j&64?0:64)|(j&64?32:0)|(j&64?16:0)|(j&64?8:0)|(j&64?4:0)|(j&64?2:0)|(j&64?1:0), (j&64?128:0)|(j&64?64:0)|(j&0x3f), 0); break;
                case disasm_arg_VmTs4b: str+=sprintf(str,"V%d.4b[%d]", m, j); break;
                case disasm_arg_Vm2d: str+=sprintf(str,"V%d.2d", m); break;
                case disasm_arg_Vm16b: str+=sprintf(str,"V%d.16b", m); break;
                case disasm_arg_Vd16b: str+=sprintf(str,"V%d.16b", d); break;
                case disasm_arg_Vd4s: str+=sprintf(str,"V%d.4s", d); break;
                case disasm_arg_FPz5t: str+=sprintf(str,"%c%d", z==1?'d':(z==0?'s':'h'), t); break;
                case disasm_arg_fbits: str+=sprintf(str,"#%d", 64-j); break;
                case disasm_arg_FPz5n: str+=sprintf(str,"%c%d", z==1?'d':(z==0?'s':'h'), n); break;
                case disasm_arg_Vn1d: str+=sprintf(str,"V%d.1d[n]", n); break;
                case disasm_arg_Vt1d: str+=sprintf(str,"V%d.1d[1]", t); break;
                case disasm_arg_FPk5t: str+=sprintf(str,"%c%d", k==1?'d':(k==0?'s':'h'), t); break;
                case disasm_arg_FPz5m: str+=sprintf(str,"%c%d", z==1?'d':(z==0?'s':'h'), m); break;
                case disasm_arg_jz: str+=sprintf(str,z==3?"#0x02x%02x":(z==0?"#0x02x%02x0000":"#0x02x%02x%06x"), z==3?(j&128)|(j&64?0:64)|(j&64?32:0)|(j&64?16:0)|((j>>2)&0xF):(j&128)|(j&64?0:64)|(j&64?32:0)|(j&64?16:0)|(j&64?8:0)|(j&64?4:0)|(j&64?2:0)|(j&(z==0?32:64)?1:0),z==3?(j&3)<<6:(z==0?(j&0x1f)<<3:(j&64?128:0)|(j&64?64:0)|(j&0x3f)), 0); break;
                case disasm_arg_FPz5d: str+=sprintf(str,"%c%d", z==1?'d':(z==0?'s':'h'), d); break;
                default: break;
            }
            if(*(str-2)==',')str-=2;
        }
        *str=0;
    }
    return addr+4;
}

