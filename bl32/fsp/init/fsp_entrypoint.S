/*
 * Copyright (c) 2013-2019, ARM Limited and Contributors. All rights reserved.
 *
 * SPDX-License-Identifier: BSD-3-Clause
 */

/*
 * Stripped down from TF-A's bl32/tsp/aarch64/tsp_entrypoint.S

 * Also pulled from TF-A:
 *      lib/aarch64/cache_helpers.S for inv_dcache_range
 *      lib/aarch64/misc_helpers.S for zeromem
 *      plat/common/aarch64/platform_up_stack.S for stack functions
 */

#include "../include/fsp.h"
#include "../include/fsp_private.h"
#include "../include/qemu_defs.h"
#include "../include/fsp_asm_macros.S"


.globl  fsp_entrypoint
.globl  fsp_vector_table

.local  fsp_zeromem
.local  fsp_zeromem_dczva
.local  fsp_inv_dcache_range
.local  fsp_plat_get_my_stack
.local  fsp_plat_set_my_stack
.local  fsp_platform_coherent_stacks

func fsp_entrypoint _align=3

/*
 * This is an infinite loop so we can use GDB.
 * The execution will spin here and we can attach GDB.
 */
#if SPIN_ON_FSP
fsp_debug_loop:
	b	fsp_debug_loop
#endif

    /* ---------------------------------------------
     * Set the exception vector to something sane.
     * ---------------------------------------------
     */
    adr x0, fsp_exceptions
    msr vbar_el1, x0
    isb

    /* ---------------------------------------------
     * Enable the SError interrupt now that the
     * exception vectors have been setup.
     * ---------------------------------------------
     */
    msr daifclr, #DAIF_ABT_BIT

    /* ---------------------------------------------
     * Enable the instruction cache, stack pointer
     * and data access alignment checks and disable
     * speculative loads.
     * ---------------------------------------------
     */
    mov x1, #(SCTLR_I_BIT | SCTLR_A_BIT | SCTLR_SA_BIT)
    mrs x0, sctlr_el1
    orr x0, x0, x1
    bic x0, x0, #SCTLR_DSSBS_BIT
    msr sctlr_el1, x0
    isb

    /* ---------------------------------------------
     * Invalidate the RW memory used by the BL32
     * image. This includes the data and NOBITS
     * sections. This is done to safeguard against
     * possible corruption of this memory by dirty
     * cache lines in a system cache as a result of
     * use by an earlier boot loader stage.
     * ---------------------------------------------
     */
    adr x0, __RW_START__
    adr x1, __RW_END__
    sub x1, x1, x0
    bl  fsp_inv_dcache_range

    /* ---------------------------------------------
     * Zero out NOBITS sections. There are 2 of them:
     *   - the .bss section;
     *   - the coherent memory section.
     * ---------------------------------------------
     */
    ldr   x0, =__BSS_START__
    ldr   x1, =__BSS_SIZE__
    bl    fsp_zeromem

#if USE_COHERENT_MEM
    ldr   x0, =__COHERENT_RAM_START__
    ldr   x1, =__COHERENT_RAM_UNALIGNED_SIZE__
    bl    fsp_zeromem
#endif

    ///* --------------------------------------------
    // * Allocate a stack whose memory will be marked
    // * as Normal-IS-WBWA when the MMU is enabled.
    // * There is no risk of reading stale stack
    // * memory after enabling the MMU as only the
    // * primary cpu is running at the moment.
    // * --------------------------------------------
    // */
    bl    fsp_plat_set_my_stack

    /* ---------------------------------------------
     * Jump to FSP main
     * ---------------------------------------------
     */
    //bl  fsp_c_main
    bl  fsp_main

    /* ---------------------------------------------
     * Tell FSPD that we are done initialising
     * ---------------------------------------------
     */
    mov x1, x0
    mov x0, #FSP_ENTRY_DONE
    smc #0

fsp_entrypoint_panic:
    b   fsp_entrypoint_panic
endfunc fsp_entrypoint


/* -------------------------------------------
 * Table of entrypoint vectors provided to the
 * FSPD for the various entrypoints
 * -------------------------------------------
 */
vector_base fsp_vector_table
    b   fsp_yield_smc_entry
    b   fsp_fast_smc_entry
    b   fsp_cpu_on_entry
    b   fsp_cpu_off_entry
    b   fsp_cpu_resume_entry
    b   fsp_cpu_suspend_entry
    b   fsp_sel1_intr_entry
    b   fsp_system_off_entry
    b   fsp_system_reset_entry
    b   fsp_abort_yield_smc_entry

/*---------------------------------------------
 * This entrypoint is used by the FSPD when this
 * cpu is to be turned off through a CPU_OFF
 * psci call to ask the FSP to perform any
 * bookeeping necessary. In the current
 * implementation, the FSPD expects the FSP to
 * re-initialise its state so nothing is done
 * here except for acknowledging the request.
 * ---------------------------------------------
 */
func fsp_cpu_off_entry
    bl  fsp_cpu_off_main
    restore_args_call_smc
endfunc fsp_cpu_off_entry

/*---------------------------------------------
 * This entrypoint is used by the FSPD when the
 * system is about to be switched off (through
 * a SYSTEM_OFF psci call) to ask the FSP to
 * perform any necessary bookkeeping.
 * ---------------------------------------------
 */
func fsp_system_off_entry
    bl  fsp_system_off_main
    restore_args_call_smc
endfunc fsp_system_off_entry

/*---------------------------------------------
 * This entrypoint is used by the FSPD when the
 * system is about to be reset (through a
 * SYSTEM_RESET psci call) to ask the FSP to
 * perform any necessary bookkeeping.
 * ---------------------------------------------
 */
func fsp_system_reset_entry
    bl  fsp_system_reset_main
    restore_args_call_smc
endfunc fsp_system_reset_entry

/*---------------------------------------------
 * This entrypoint is used by the FSPD when this
 * cpu is turned on using a CPU_ON psci call to
 * ask the FSP to initialise itself i.e. setup
 * the mmu, stacks etc. Minimal architectural
 * state will be initialised by the FSPD when
 * this function is entered i.e. Caches and MMU
 * will be turned off, the execution state
 * will be aarch64 and exceptions masked.
 * ---------------------------------------------
 */
func fsp_cpu_on_entry
    /* ---------------------------------------------
     * Set the exception vector to something sane.
     * ---------------------------------------------
     */
    adr x0, fsp_exceptions
    msr vbar_el1, x0
    isb

    /* Enable the SError interrupt */
    msr daifclr, #DAIF_ABT_BIT

    /* ---------------------------------------------
     * Enable the instruction cache, stack pointer
     * and data access alignment checks
     * ---------------------------------------------
     */
    mov x1, #(SCTLR_I_BIT | SCTLR_A_BIT | SCTLR_SA_BIT)
    mrs x0, sctlr_el1
    orr x0, x0, x1
    msr sctlr_el1, x0
    isb

    /* --------------------------------------------
     * Give ourselves a stack whose memory will be
     * marked as Normal-IS-WBWA when the MMU is
     * enabled.
     * --------------------------------------------
     */
    bl  fsp_plat_set_my_stack

    /* --------------------------------------------
     * Enable MMU and D-caches together.
     * --------------------------------------------
     */
    mov x0, #0
    bl  bl32_plat_enable_mmu

#if ENABLE_PAUTH
    /* ---------------------------------------------
     * Program APIAKey_EL1
     * and enable pointer authentication
     * ---------------------------------------------
     */
    bl  pauth_init_enable_el1
#endif /* ENABLE_PAUTH */

    /* ---------------------------------------------
     * Enter C runtime to perform any remaining
     * book keeping
     * ---------------------------------------------
     */
    bl  fsp_cpu_on_main
    restore_args_call_smc

/* Should never reach here */
fsp_cpu_on_entry_panic:
    b   fsp_cpu_on_entry_panic
endfunc fsp_cpu_on_entry

/*---------------------------------------------
 * This entrypoint is used by the FSPD when this
 * cpu is to be suspended through a CPU_SUSPEND
 * psci call to ask the FSP to perform any
 * bookeeping necessary. In the current
 * implementation, the FSPD saves and restores
 * the EL1 state.
 * ---------------------------------------------
 */
func fsp_cpu_suspend_entry
    bl  fsp_cpu_suspend_main
    restore_args_call_smc
endfunc fsp_cpu_suspend_entry

/*-------------------------------------------------
 * This entrypoint is used by the FSPD to pass
 * control for `synchronously` handling a S-EL1
 * Interrupt which was triggered while executing
 * in normal world. 'x0' contains a magic number
 * which indicates this. FSPD expects control to
 * be handed back at the end of interrupt
 * processing. This is done through an SMC.
 * The handover agreement is:
 *
 * 1. PSTATE.DAIF are set upon entry. 'x1' has
 *    the ELR_EL3 from the non-secure state.
 * 2. FSP has to preserve the callee saved
 *    general purpose registers, SP_EL1/EL0 and
 *    LR.
 * 3. FSP has to preserve the system and vfp
 *    registers (if applicable).
 * 4. FSP can use 'x0-x18' to enable its C
 *    runtime.
 * 5. FSP returns to FSPD using an SMC with
 *    'x0' = FSP_HANDLED_S_EL1_INTR
 * ------------------------------------------------
 */
func    fsp_sel1_intr_entry
#if DEBUG
    mov_imm x2, FSP_HANDLE_SEL1_INTR_AND_RETURN
    cmp x0, x2
    b.ne    fsp_sel1_int_entry_panic
#endif
    /*-------------------------------------------------
     * Save any previous context needed to perform
     * an exception return from S-EL1 e.g. context
     * from a previous Non secure Interrupt.
     * Update statistics and handle the S-EL1
     * interrupt before returning to the FSPD.
     * IRQ/FIQs are not enabled since that will
     * complicate the implementation. Execution
     * will be transferred back to the normal world
     * in any case. The handler can return 0
     * if the interrupt was handled or FSP_PREEMPTED
     * if the expected interrupt was preempted
     * by an interrupt that should be handled in EL3
     * e.g. Group 0 interrupt in GICv3. In both
     * the cases switch to EL3 using SMC with id
     * FSP_HANDLED_S_EL1_INTR. Any other return value
     * from the handler will result in panic.
     * ------------------------------------------------
     */
    save_eret_context x2 x3
    bl  fsp_update_sync_sel1_intr_stats
    bl  fsp_common_int_handler
    /* Check if the S-EL1 interrupt has been handled */
    cbnz    x0, fsp_sel1_intr_check_preemption
    b   fsp_sel1_intr_return
fsp_sel1_intr_check_preemption:
    /* Check if the S-EL1 interrupt has been preempted */
    mov_imm x1, FSP_PREEMPTED
    cmp x0, x1
    b.ne    fsp_sel1_int_entry_panic
fsp_sel1_intr_return:
    mov_imm x0, FSP_HANDLED_S_EL1_INTR
    restore_eret_context x2 x3
    smc #0

    /* Should never reach here */
fsp_sel1_int_entry_panic:
    no_ret  fsp_plat_panic_handler
endfunc fsp_sel1_intr_entry

/*---------------------------------------------
 * This entrypoint is used by the FSPD when this
 * cpu resumes execution after an earlier
 * CPU_SUSPEND psci call to ask the FSP to
 * restore its saved context. In the current
 * implementation, the FSPD saves and restores
 * EL1 state so nothing is done here apart from
 * acknowledging the request.
 * ---------------------------------------------
 */
func fsp_cpu_resume_entry
    bl  fsp_cpu_resume_main
    restore_args_call_smc

    /* Should never reach here */
    no_ret  fsp_plat_panic_handler
endfunc fsp_cpu_resume_entry

/*---------------------------------------------
 * This entrypoint is used by the FSPD to ask
 * the FSP to service a fast smc request.
 * ---------------------------------------------
 */
func fsp_fast_smc_entry
    bl  fsp_smc_handler
    restore_args_call_smc

    /* Should never reach here */
    no_ret  fsp_plat_panic_handler
endfunc fsp_fast_smc_entry

/*---------------------------------------------
 * This entrypoint is used by the FSPD to ask
 * the FSP to service a Yielding SMC request.
 * We will enable preemption during execution
 * of fsp_smc_handler.
 * ---------------------------------------------
 */
func fsp_yield_smc_entry
    msr daifclr, #DAIF_FIQ_BIT | DAIF_IRQ_BIT
    bl  fsp_smc_handler
    msr daifset, #DAIF_FIQ_BIT | DAIF_IRQ_BIT
    restore_args_call_smc

    /* Should never reach here */
    no_ret  fsp_plat_panic_handler
endfunc fsp_yield_smc_entry

/*---------------------------------------------------------------------
 * This entrypoint is used by the FSPD to abort a pre-empted Yielding
 * SMC. It could be on behalf of non-secure world or because a CPU
 * suspend/CPU off request needs to abort the preempted SMC.
 * --------------------------------------------------------------------
 */
func fsp_abort_yield_smc_entry

    /*
     * Exceptions masking is already done by the FSPD when entering this
     * hook so there is no need to do it here.
     */

    /* Reset the stack used by the pre-empted SMC */
    bl  fsp_plat_set_my_stack

    /*
     * Allow some cleanup such as releasing locks.
     */
    bl  fsp_abort_smc_handler

    restore_args_call_smc

    /* Should never reach here */
    bl  fsp_plat_panic_handler
endfunc fsp_abort_yield_smc_entry

/* -----------------------------------------------------------------------
 * void zeromem(void *mem, unsigned int length);
 *
 * Initialise a region of device memory to 0. This functions complies with the
 * AAPCS and can be called from C code.
 *
 * NOTE: When data caches and MMU are enabled, zero_normalmem can usually be
 *       used instead for faster zeroing.
 *
 * -----------------------------------------------------------------------
 */
func fsp_zeromem
    /* x2 is the address past the last zeroed address */
    add x2, x0, x1
    /*
     * Uses the fallback path that does not use DC ZVA instruction and
     * therefore does not need enabled MMU
     */
    b   .Lfsp_zeromem_dczva_fallback_entry
endfunc fsp_zeromem

/* -----------------------------------------------------------------------
 * void zeromem_dczva(void *mem, unsigned int length);
 *
 * Fill a region of normal memory of size "length" in bytes with null bytes.
 * MMU must be enabled and the memory be of
 * normal type. This is because this function internally uses the DC ZVA
 * instruction, which generates an Alignment fault if used on any type of
 * Device memory (see section D3.4.9 of the ARMv8 ARM, issue k). When the MMU
 * is disabled, all memory behaves like Device-nGnRnE memory (see section
 * D4.2.8), hence the requirement on the MMU being enabled.
 * NOTE: The code assumes that the block size as defined in DCZID_EL0
 *       register is at least 16 bytes.
 *
 * -----------------------------------------------------------------------
 */
func fsp_zeromem_dczva

    /*
     * The function consists of a series of loops that zero memory one byte
     * at a time, 16 bytes at a time or using the DC ZVA instruction to
     * zero aligned block of bytes, which is assumed to be more than 16.
     * In the case where the DC ZVA instruction cannot be used or if the
     * first 16 bytes loop would overflow, there is fallback path that does
     * not use DC ZVA.
     * Note: The fallback path is also used by the zeromem function that
     *       branches to it directly.
     *
     *              +---------+   zeromem_dczva
     *              |  entry  |
     *              +----+----+
     *                   |
     *                   v
     *              +---------+
     *              | checks  |>o-------+ (If any check fails, fallback)
     *              +----+----+         |
     *                   |              |---------------+
     *                   v              | Fallback path |
     *            +------+------+       |---------------+
     *            | 1 byte loop |       |
     *            +------+------+ .Lzeromem_dczva_initial_1byte_aligned_end
     *                   |              |
     *                   v              |
     *           +-------+-------+      |
     *           | 16 bytes loop |      |
     *           +-------+-------+      |
     *                   |              |
     *                   v              |
     *            +------+------+ .Lzeromem_dczva_blocksize_aligned
     *            | DC ZVA loop |       |
     *            +------+------+       |
     *       +--------+  |              |
     *       |        |  |              |
     *       |        v  v              |
     *       |   +-------+-------+ .Lzeromem_dczva_final_16bytes_aligned
     *       |   | 16 bytes loop |      |
     *       |   +-------+-------+      |
     *       |           |              |
     *       |           v              |
     *       |    +------+------+ .Lzeromem_dczva_final_1byte_aligned
     *       |    | 1 byte loop |       |
     *       |    +-------------+       |
     *       |           |              |
     *       |           v              |
     *       |       +---+--+           |
     *       |       | exit |           |
     *       |       +------+           |
     *       |              |
     *       |           +--------------+    +------------------+ zeromem
     *       |           |  +----------------| zeromem function |
     *       |           |  |                +------------------+
     *       |           v  v
     *       |    +-------------+ .Lzeromem_dczva_fallback_entry
     *       |    | 1 byte loop |
     *       |    +------+------+
     *       |           |
     *       +-----------+
     */

    /*
     * Readable names for registers
     *
     * Registers x0, x1 and x2 are also set by zeromem which
     * branches into the fallback path directly, so cursor, length and
     * stop_address should not be retargeted to other registers.
     */
    cursor       .req x0 /* Start address and then current address */
    length       .req x1 /* Length in bytes of the region to zero out */
    /* Reusing x1 as length is never used after block_mask is set */
    block_mask   .req x1 /* Bitmask of the block size read in DCZID_EL0 */
    stop_address .req x2 /* Address past the last zeroed byte */
    block_size   .req x3 /* Size of a block in bytes as read in DCZID_EL0 */
    tmp1         .req x4
    tmp2         .req x5

//#if ENABLE_ASSERTIONS
//    /*
//     * Check for M bit (MMU enabled) of the current SCTLR_EL(1|3)
//     * register value and panic if the MMU is disabled.
//     */
//#if defined(IMAGE_BL1) || defined(IMAGE_BL31) || (defined(IMAGE_BL2) && BL2_AT_EL3)
//    mrs tmp1, sctlr_el3
//#else
//    mrs tmp1, sctlr_el1
//#endif
//
//    tst tmp1, #SCTLR_M_BIT
//    ASM_ASSERT(ne)
//#endif /* ENABLE_ASSERTIONS */

    /* stop_address is the address past the last to zero */
    add stop_address, cursor, length

    /*
     * Get block_size = (log2(<block size>) >> 2) (see encoding of
     * dczid_el0 reg)
     */
    mrs block_size, dczid_el0

    /*
     * Select the 4 lowest bits and convert the extracted log2(<block size
     * in words>) to <block size in bytes>
     */
    ubfx    block_size, block_size, #0, #4
    mov tmp2, #(1 << 2)
    lsl block_size, tmp2, block_size

//#if ENABLE_ASSERTIONS
//    /*
//     * Assumes block size is at least 16 bytes to avoid manual realignment
//     * of the cursor at the end of the DCZVA loop.
//     */
//    cmp block_size, #16
//    ASM_ASSERT(hs)
//#endif
    /*
     * Not worth doing all the setup for a region less than a block and
     * protects against zeroing a whole block when the area to zero is
     * smaller than that. Also, as it is assumed that the block size is at
     * least 16 bytes, this also protects the initial aligning loops from
     * trying to zero 16 bytes when length is less than 16.
     */
    cmp length, block_size
    b.lo    .Lfsp_zeromem_dczva_fallback_entry

    /*
     * Calculate the bitmask of the block alignment. It will never
     * underflow as the block size is between 4 bytes and 2kB.
     * block_mask = block_size - 1
     */
    sub block_mask, block_size, #1

    /*
     * length alias should not be used after this point unless it is
     * defined as a register other than block_mask's.
     */
     .unreq length

    /*
     * If the start address is already aligned to zero block size, go
     * straight to the cache zeroing loop. This is safe because at this
     * point, the length cannot be smaller than a block size.
     */
    tst cursor, block_mask
    b.eq    .Lfsp_zeromem_dczva_blocksize_aligned

    /*
     * Calculate the first block-size-aligned address. It is assumed that
     * the zero block size is at least 16 bytes. This address is the last
     * address of this initial loop.
     */
    orr tmp1, cursor, block_mask
    add tmp1, tmp1, #1

    /*
     * If the addition overflows, skip the cache zeroing loops. This is
     * quite unlikely however.
     */
    cbz tmp1, .Lfsp_zeromem_dczva_fallback_entry

    /*
     * If the first block-size-aligned address is past the last address,
     * fallback to the simpler code.
     */
    cmp tmp1, stop_address
    b.hi    .Lfsp_zeromem_dczva_fallback_entry

    /*
     * If the start address is already aligned to 16 bytes, skip this loop.
     * It is safe to do this because tmp1 (the stop address of the initial
     * 16 bytes loop) will never be greater than the final stop address.
     */
    tst cursor, #0xf
    b.eq    .Lfsp_zeromem_dczva_initial_1byte_aligned_end

    /* Calculate the next address aligned to 16 bytes */
    orr tmp2, cursor, #0xf
    add tmp2, tmp2, #1
    /* If it overflows, fallback to the simple path (unlikely) */
    cbz tmp2, .Lfsp_zeromem_dczva_fallback_entry
    /*
     * Next aligned address cannot be after the stop address because the
     * length cannot be smaller than 16 at this point.
     */

    /* First loop: zero byte per byte */
1:
    strb    wzr, [cursor], #1
    cmp cursor, tmp2
    b.ne    1b
.Lfsp_zeromem_dczva_initial_1byte_aligned_end:

    /*
     * Second loop: we need to zero 16 bytes at a time from cursor to tmp1
     * before being able to use the code that deals with block-size-aligned
     * addresses.
     */
    cmp cursor, tmp1
    b.hs    2f
1:
    stp xzr, xzr, [cursor], #16
    cmp cursor, tmp1
    b.lo    1b
2:

    /*
     * Third loop: zero a block at a time using DC ZVA cache block zeroing
     * instruction.
     */
.Lfsp_zeromem_dczva_blocksize_aligned:
    /*
     * Calculate the last block-size-aligned address. If the result equals
     * to the start address, the loop will exit immediately.
     */
    bic tmp1, stop_address, block_mask

    cmp cursor, tmp1
    b.hs    2f
1:
    /* Zero the block containing the cursor */
    dc  zva, cursor
    /* Increment the cursor by the size of a block */
    add cursor, cursor, block_size
    cmp cursor, tmp1
    b.lo    1b
2:

    /*
     * Fourth loop: zero 16 bytes at a time and then byte per byte the
     * remaining area
     */
.Lfsp_zeromem_dczva_final_16bytes_aligned:
    /*
     * Calculate the last 16 bytes aligned address. It is assumed that the
     * block size will never be smaller than 16 bytes so that the current
     * cursor is aligned to at least 16 bytes boundary.
     */
    bic tmp1, stop_address, #15

    cmp cursor, tmp1
    b.hs    2f
1:
    stp xzr, xzr, [cursor], #16
    cmp cursor, tmp1
    b.lo    1b
2:

    /* Fifth and final loop: zero byte per byte */
.Lfsp_zeromem_dczva_final_1byte_aligned:
    cmp cursor, stop_address
    b.eq    2f
1:
    strb    wzr, [cursor], #1
    cmp cursor, stop_address
    b.ne    1b
2:
    ret

    /* Fallback for unaligned start addresses */
.Lfsp_zeromem_dczva_fallback_entry:
    /*
     * If the start address is already aligned to 16 bytes, skip this loop.
     */
    tst cursor, #0xf
    b.eq    .Lfsp_zeromem_dczva_final_16bytes_aligned

    /* Calculate the next address aligned to 16 bytes */
    orr tmp1, cursor, #15
    add tmp1, tmp1, #1
    /* If it overflows, fallback to byte per byte zeroing */
    cbz tmp1, .Lfsp_zeromem_dczva_final_1byte_aligned
    /* If the next aligned address is after the stop address, fall back */
    cmp tmp1, stop_address
    b.hs    .Lfsp_zeromem_dczva_final_1byte_aligned

    /* Fallback entry loop: zero byte per byte */
1:
    strb    wzr, [cursor], #1
    cmp cursor, tmp1
    b.ne    1b

    b   .Lfsp_zeromem_dczva_final_16bytes_aligned

    .unreq  cursor
    /*
     * length is already unreq'ed to reuse the register for another
     * variable.
     */
    .unreq  stop_address
    .unreq  block_size
    .unreq  block_mask
    .unreq  tmp1
    .unreq  tmp2
endfunc fsp_zeromem_dczva

/* ------------------------------------------
 * Invalidate from base address till
 * size. 'x0' = addr, 'x1' = size
 * ------------------------------------------
 */
func fsp_inv_dcache_range
    do_dcache_maintenance_by_mva ivac
endfunc fsp_inv_dcache_range

/* -------------------------------------------------------
 * uintptr_t plat_get_my_stack ()
 *
 * For cold-boot BL images, only the primary CPU needs a
 * stack. This function returns the stack pointer for a
 * stack allocated in coherent memory.
 * -------------------------------------------------------
 */
func fsp_plat_get_my_stack
    get_up_stack fsp_platform_coherent_stacks, PLATFORM_STACK_SIZE
    ret
endfunc fsp_plat_get_my_stack

/* -------------------------------------------------------
 * void plat_set_my_stack ()
 *
 * For cold-boot BL images, only the primary CPU needs a
 * stack. This function sets the stack pointer to a stack
 * allocated in coherent memory.
 * -------------------------------------------------------
 */
func fsp_plat_set_my_stack
    get_up_stack fsp_platform_coherent_stacks, PLATFORM_STACK_SIZE
    mov sp, x0
    ret
endfunc fsp_plat_set_my_stack

/* ----------------------------------------------------
 * Single cpu stack in coherent memory.
 * ----------------------------------------------------
 */
declare_stack fsp_platform_coherent_stacks, tzfw_coherent_mem, \
        PLATFORM_STACK_SIZE, 1, CACHE_WRITEBACK_GRANULE
