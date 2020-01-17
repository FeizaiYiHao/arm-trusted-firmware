/*
 * Copyright (c) 2013-2019, ARM Limited and Contributors. All rights reserved.
 *
 * SPDX-License-Identifier: BSD-3-Clause
 */

/*
 * Pulled from TF-A:
 *      lib/aarch64/cache_helpers.S for dcache macros.
 *      include/common/asm_macros_common.S for func and endfunc
 *      include/arch/aarch64/asm_macros.S for vector macros, mov_imm macros, and stack macros.
 */
#ifndef FSP_ASM_MACROS_S
#define FSP_ASM_MACROS_S

/*
 * This macro is used to create a function label and place the
 * code into a separate text section based on the function name
 * to enable elimination of unused code during linking. It also adds
 * basic debug information to enable call stack printing most of the
 * time. The optional _align parameter can be used to force a
 * non-standard alignment (indicated in powers of 2). The default is
 * _align=2 because both Aarch32 and Aarch64 instructions must be
 * word aligned. Do *not* try to use a raw .align directive. Since func
 * switches to a new section, this would not have the desired effect.
 */
.macro func _name, _align=2
/*
 * Add Call Frame Information entry in the .debug_frame section for
 * debugger consumption. This enables callstack printing in debuggers.
 * This does not use any space in the final loaded binary, only in the
 * ELF file.
 * Note that a function manipulating the CFA pointer location (i.e. the
 * x29 frame pointer on AArch64) should declare it using the
 * appropriate .cfi* directives, or be prepared to have a degraded
 * debugging experience.
 */
    .cfi_sections .debug_frame
    .section .text.asm.\_name, "ax"
    .type \_name, %function
    /*
     * .cfi_startproc and .cfi_endproc are needed to output entries in
     * .debug_frame
     */
    .cfi_startproc
    .align \_align
\_name:
#if ENABLE_BTI
    /* When Branch Target Identification is enabled, insert "bti jc"
     * instruction to enable indirect calls and branches
     */
     bti    jc
#endif
.endm

/*
 * This macro is used to mark the end of a function.
 */
.macro endfunc _name
    .cfi_endproc
    .size \_name, . - \_name
.endm

/*
 * Assembler macro to enable asm_assert. Use this macro wherever
 * assert is required in assembly. Please note that the macro makes
 * use of label '300' to provide the logic and the caller
 * should make sure that this label is not used to branch prior
 * to calling this macro.
 */
 /*
#define ASM_ASSERT(_cc) \
.ifndef .L_assert_filename ;\
.pushsection .rodata.str1.1, "aS" ;\
.L_assert_filename: ;\
        .string __FILE__ ;\
.popsection ;\
.endif ;\
b._cc   300f ;\
adr x0, .L_assert_filename ;\
mov x1, __LINE__ ;\
b   asm_assert ;\
300:
*/

.macro  dcache_line_size  reg, tmp
    mrs \tmp, ctr_el0
    ubfx    \tmp, \tmp, #16, #4
    mov \reg, #4
    lsl \reg, \reg, \tmp
.endm

/*
 * This macro can be used for implementing various data cache operations `op`
 */
.macro do_dcache_maintenance_by_mva op
    /* Exit early if size is zero */
    cbz x1, exit_loop_\op
    dcache_line_size x2, x3
    add x1, x0, x1
    sub x3, x2, #1
    bic x0, x0, x3
loop_\op:
    dc  \op, x0
    add x0, x0, x2
    cmp x0, x1
    b.lo    loop_\op
    dsb sy
exit_loop_\op:
    ret
.endm

/*
 * Declare the exception vector table, enforcing it is aligned on a
 * 2KB boundary, as required by the ARMv8 architecture.
 * Use zero bytes as the fill value to be stored in the padding bytes
 * so that it inserts illegal AArch64 instructions. This increases
 * security, robustness and potentially facilitates debugging.
 */
.macro vector_base  label, section_name=.vectors
    .section \section_name, "ax"
    .align 11, 0
\label:
.endm

/*
 * Create an entry in the exception vector table, enforcing it is
 * aligned on a 128-byte boundary, as required by the ARMv8 architecture.
 * Use zero bytes as the fill value to be stored in the padding bytes
 * so that it inserts illegal AArch64 instructions. This increases
 * security, robustness and potentially facilitates debugging.
 */
.macro vector_entry  label, section_name=.vectors
    .cfi_sections .debug_frame
    .section \section_name, "ax"
    .align 7, 0
    .type \label, %function
    .cfi_startproc
\label:
.endm

/*
 * Add the bytes until fill the full exception vector, whose size is always
 * 32 instructions. If there are more than 32 instructions in the
 * exception vector then an error is emitted.
 */
.macro end_vector_entry label
    .cfi_endproc
    .fill   \label + (32 * 4) - .
.endm

/*
 * Helper macro to generate the best mov/movk combinations according
 * the value to be moved. The 16 bits from '_shift' are tested and
 * if not zero, they are moved into '_reg' without affecting
 * other bits.
 */
.macro _mov_imm16 _reg, _val, _shift
    .if (\_val >> \_shift) & 0xffff
        .if (\_val & (1 << \_shift - 1))
            movk    \_reg, (\_val >> \_shift) & 0xffff, LSL \_shift
        .else
            mov \_reg, \_val & (0xffff << \_shift)
        .endif
    .endif
.endm

/*
 * Helper macro to load arbitrary values into 32 or 64-bit registers
 * which generates the best mov/movk combinations. Many base addresses
 * are 64KB aligned the macro will eliminate updating bits 15:0 in
 * that case
 */
.macro mov_imm _reg, _val
    .if (\_val) == 0
        mov \_reg, #0
    .else
        _mov_imm16  \_reg, (\_val), 0
        _mov_imm16  \_reg, (\_val), 16
        _mov_imm16  \_reg, (\_val), 32
        _mov_imm16  \_reg, (\_val), 48
    .endif
.endm

/*
 * Macro to mark instances where we're jumping to a function and don't
 * expect a return. To provide the function being jumped to with
 * additional information, we use 'bl' instruction to jump rather than
 * 'b'.
     *
 * Debuggers infer the location of a call from where LR points to, which
 * is usually the instruction after 'bl'. If this macro expansion
 * happens to be the last location in a function, that'll cause the LR
 * to point a location beyond the function, thereby misleading debugger
 * back trace. We therefore insert a 'nop' after the function call for
 * debug builds, unless 'skip_nop' parameter is non-zero.
 */
.macro no_ret _func:req, skip_nop=0
bl  \_func
#if DEBUG
    .ifeq \skip_nop
        nop
    .endif
#endif
.endm

/*
 * Helper assembler macro to count trailing zeros. The output is
 * populated in the `TZ_COUNT` symbol.
 */
.macro count_tz _value, _tz_count
    .if \_value
        count_tz "(\_value >> 1)", "(\_tz_count + 1)"
    .else
        .equ TZ_COUNT, (\_tz_count - 1)
    .endif
.endm

/*
 * This macro declares an array of 1 or more stacks, properly
 * aligned and in the requested section
 */
#define DEFAULT_STACK_ALIGN (1 << 6)   /* In case the caller doesnt provide alignment */

.macro declare_stack _name, _section, _size, _count, _align=DEFAULT_STACK_ALIGN
    count_tz \_align, 0
    .if (\_align - (1 << TZ_COUNT))
      .error "Incorrect stack alignment specified (Must be a power of 2)."
    .endif
    .if ((\_size & ((1 << TZ_COUNT) - 1)) <> 0)
      .error "Stack size not correctly aligned"
    .endif
    .section    \_section, "aw", %nobits
    .align TZ_COUNT
\_name:
    .space ((\_count) * (\_size)), 0
.endm

/*
 * This macro calculates the base address of a UP stack using the
 * name of the stack storage and the size of the stack
 * Out: X0 = physical address of stack base
 */
.macro get_up_stack _name, _size
    adrp    x0, (\_name + \_size)
    add x0, x0, :lo12:(\_name + \_size)
.endm

/* ---------------------------------------------
 * Populate the params in x0-x7 from the pointer
 * to the smc args structure in x0.
 * ---------------------------------------------
 */
.macro restore_args_call_smc
    ldp x6, x7, [x0, #FSP_ARG6]
    ldp x4, x5, [x0, #FSP_ARG4]
    ldp x2, x3, [x0, #FSP_ARG2]
    ldp x0, x1, [x0, #FSP_ARG0]
    smc #0
.endm

.macro  save_eret_context reg1 reg2
    mrs \reg1, elr_el1
    mrs \reg2, spsr_el1
    stp \reg1, \reg2, [sp, #-0x10]!
    stp x30, x18, [sp, #-0x10]!
.endm

.macro restore_eret_context reg1 reg2
    ldp x30, x18, [sp], #0x10
    ldp \reg1, \reg2, [sp], #0x10
    msr elr_el1, \reg1
    msr spsr_el1, \reg2
.endm

#endif /* FSP_ASM_MACROS_S */
