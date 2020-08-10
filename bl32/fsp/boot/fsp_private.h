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

#define FSP_ENTRY_DONE 0xf2000000
#define FSP_PREEMPTED 0xf2000005
#define FSP_HANDLED_S_EL1_INTR 0xf2000006
#define FSP_HANDLE_SEL1_INTR_AND_RETURN 0x2004

/*
 * Pulled from include/arch/aarch64/arch.h.
 */
#define SCTLR_M_BIT		(1 << 0)
#define SCTLR_A_BIT		(1 << 1)
#define SCTLR_SA_BIT		(1 << 3)
#define SCTLR_I_BIT		(1 << 12)
#define SCTLR_DSSBS_BIT		(1 << 44)

#define DAIF_FIQ_BIT		(1 << 0)
#define DAIF_IRQ_BIT		(1 << 1)
#define DAIF_ABT_BIT		(1 << 2)
#define DAIF_DBG_BIT		(1 << 3)

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

#endif /* FSP_PRIVATE_H */
