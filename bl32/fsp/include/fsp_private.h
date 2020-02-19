/*
 * Copyright (c) 2014-2018, ARM Limited and Contributors. All rights reserved.
 *
 * SPDX-License-Identifier: BSD-3-Clause
 */

/*
 * Pulled mainlly from TF-A's bl32/tsp/tsp_private.h but other header files too.
 */

#ifndef FSP_PRIVATE_H
#define FSP_PRIVATE_H

/*
 * For those constants to be shared between C and other sources, apply a 'U',
 * 'UL', 'ULL', 'L' or 'LL' suffix to the argument only in C, to avoid
 * undefined or unintended behaviour.
 *
 * The GNU assembler and linker do not support these suffixes (it causes the
 * build process to fail) therefore the suffix is omitted when used in linker
 * scripts and assembler files.
*/

/*
 * Pulled from include/export/lib/utils_def_exp.h.
 */
#if defined(__ASSEMBLER__)
# define   U(_x)    (_x)
# define  UL(_x)    (_x)
# define ULL(_x)    (_x)
# define   L(_x)    (_x)
# define  LL(_x)    (_x)
#else
# define   U(_x)    (_x##U)
# define  UL(_x)    (_x##UL)
# define ULL(_x)    (_x##ULL)
# define   L(_x)    (_x##L)
# define  LL(_x)    (_x##LL)
#endif

/*
 * Pulled from include/arch/aarch64/arch.h.
 */
#define SCTLR_M_BIT		(ULL(1) << 0)
#define SCTLR_A_BIT		(ULL(1) << 1)
#define SCTLR_SA_BIT		(ULL(1) << 3)
#define SCTLR_I_BIT		(ULL(1) << 12)
#define SCTLR_DSSBS_BIT		(ULL(1) << 44)

#define MPIDR_CPU_MASK      MPIDR_AFFLVL_MASK
#define MPIDR_CLUSTER_MASK  (MPIDR_AFFLVL_MASK << MPIDR_AFFINITY_BITS)
#define MPIDR_AFFINITY_BITS U(8)
#define MPIDR_AFFLVL_MASK   ULL(0xff)

#define DAIF_FIQ_BIT		(U(1) << 0)
#define DAIF_IRQ_BIT		(U(1) << 1)
#define DAIF_ABT_BIT		(U(1) << 2)
#define DAIF_DBG_BIT		(U(1) << 3)

/* Definitions to help the assembler access the SMC/ERET args structure */
#define FSP_ARGS_SIZE       0x40
#define FSP_ARG0            0x0
#define FSP_ARG1            0x8
#define FSP_ARG2            0x10
#define FSP_ARG3            0x18
#define FSP_ARG4            0x20
#define FSP_ARG5            0x28
#define FSP_ARG6            0x30
#define FSP_ARG7            0x38
#define FSP_ARGS_END        0x40


#ifndef __ASSEMBLER__

#include <stdint.h>
#include <string.h> // for using strncmp()

#include "qemu_defs.h" /* For CACHE_WRITEBACK_GRANULE */

#include "fsp.h"

#define __aligned(x)    __attribute__((__aligned__(x)))

typedef struct fsp_args {
    uint64_t _regs[FSP_ARGS_END >> 3];
} __aligned(CACHE_WRITEBACK_GRANULE) fsp_args_t;

/* Macros to access members of the above structure using their offsets */
#define write_sp_arg(args, offset, val) (((args)->_regs[offset >> 3])   \
                     = val)

fsp_args_t *fsp_cpu_resume_main(uint64_t max_off_pwrlvl,
                
                uint64_t arg1,
                uint64_t arg2,
                uint64_t arg3,
                uint64_t arg4,
                uint64_t arg5,
                uint64_t arg6,
                uint64_t arg7);
fsp_args_t *fsp_cpu_suspend_main(uint64_t arg0,
                uint64_t arg1,
                uint64_t arg2,
                uint64_t arg3,
                uint64_t arg4,
                uint64_t arg5,
                uint64_t arg6,
                uint64_t arg7);
fsp_args_t *fsp_cpu_on_main(void);
fsp_args_t *fsp_cpu_off_main(uint64_t arg0,
                uint64_t arg1,
                uint64_t arg2,
                uint64_t arg3,
                uint64_t arg4,
                uint64_t arg5,
                uint64_t arg6,
                uint64_t arg7);

/* S-EL1 interrupt management functions */
void fsp_update_sync_sel1_intr_stats(uint32_t type, uint64_t elr_el3);

/* Vector table of jumps */
extern fsp_vectors_t fsp_vector_table;

/* functions */
int32_t fsp_common_int_handler(void);
int32_t fsp_plat_panic_handler(void);

fsp_args_t *fsp_abort_smc_handler(uint64_t func,
                    uint64_t arg1,
                    uint64_t arg2,
                    uint64_t arg3,
                    uint64_t arg4,
                    uint64_t arg5,
                    uint64_t arg6,
                    uint64_t arg7);

fsp_args_t *fsp_smc_handler(uint64_t func,
                    uint64_t arg1,
                    uint64_t arg2,
                    uint64_t arg3,
                    uint64_t arg4,
                    uint64_t arg5,
                    uint64_t arg6,
                    uint64_t arg7);

fsp_args_t *fsp_system_reset_main(uint64_t arg0,
                    uint64_t arg1,
                    uint64_t arg2,
                    uint64_t arg3,
                    uint64_t arg4,
                    uint64_t arg5,
                    uint64_t arg6,
                    uint64_t arg7);

fsp_args_t *fsp_system_off_main(uint64_t arg0,
                    uint64_t arg1,
                    uint64_t arg2,
                    uint64_t arg3,
                    uint64_t arg4,
                    uint64_t arg5,
                    uint64_t arg6,
                    uint64_t arg7);

uint64_t fsp_c_main(void);
void fsp_init(void);
void fsp_main(void);
void fsp_print_debug_loop_message(void);
void fsp_qemu_console_init(void);
int bcmp(const void *s1, const void *s2, size_t n);

#endif /* __ASSEMBLER__ */

#endif /* FSP_PRIVATE_H */
