/*
 * Copyright (c) 2013-2018, ARM Limited and Contributors. All rights reserved.
 *
 * SPDX-License-Identifier: BSD-3-Clause
 */

#ifndef FSPD_PRIVATE_H
#define FSPD_PRIVATE_H

#include <platform_def.h>

#include <arch.h>
#include <bl31/interrupt_mgmt.h>
#include <context.h>
#include <lib/psci/psci.h>

/*******************************************************************************
 * Secure Payload PM state information e.g. SP is suspended, uninitialised etc
 * and macros to access the state information in the per-cpu 'state' flags
 ******************************************************************************/
#define FSP_PSTATE_OFF		0
#define FSP_PSTATE_ON		1
#define FSP_PSTATE_SUSPEND	2
#define FSP_PSTATE_SHIFT	0
#define FSP_PSTATE_MASK	0x3
#define get_fsp_pstate(state)	((state >> FSP_PSTATE_SHIFT) & FSP_PSTATE_MASK)
#define clr_fsp_pstate(state)	(state &= ~(FSP_PSTATE_MASK \
					    << FSP_PSTATE_SHIFT))
#define set_fsp_pstate(st, pst)	do {					       \
					clr_fsp_pstate(st);		       \
					st |= (pst & FSP_PSTATE_MASK) <<       \
						FSP_PSTATE_SHIFT;	       \
				} while (0);


/*
 * This flag is used by the FSPD to determine if the FSP is servicing a yielding
 * SMC request prior to programming the next entry into the FSP e.g. if FSP
 * execution is preempted by a non-secure interrupt and handed control to the
 * normal world. If another request which is distinct from what the FSP was
 * previously doing arrives, then this flag will be help the FSPD to either
 * reject the new request or service it while ensuring that the previous context
 * is not corrupted.
 */
#define YIELD_SMC_ACTIVE_FLAG_SHIFT	2
#define YIELD_SMC_ACTIVE_FLAG_MASK	1
#define get_yield_smc_active_flag(state)				\
				((state >> YIELD_SMC_ACTIVE_FLAG_SHIFT) \
				& YIELD_SMC_ACTIVE_FLAG_MASK)
#define set_yield_smc_active_flag(state)	(state |=		\
					1 << YIELD_SMC_ACTIVE_FLAG_SHIFT)
#define clr_yield_smc_active_flag(state)	(state &=		\
					~(YIELD_SMC_ACTIVE_FLAG_MASK	\
					<< YIELD_SMC_ACTIVE_FLAG_SHIFT))

/*******************************************************************************
 * Secure Payload execution state information i.e. aarch32 or aarch64
 ******************************************************************************/
#define FSP_AARCH32		MODE_RW_32
#define FSP_AARCH64		MODE_RW_64

/*******************************************************************************
 * The SPD should know the type of Secure Payload.
 ******************************************************************************/
#define FSP_TYPE_UP		PSCI_TOS_NOT_UP_MIG_CAP
#define FSP_TYPE_UPM		PSCI_TOS_UP_MIG_CAP
#define FSP_TYPE_MP		PSCI_TOS_NOT_PRESENT_MP

/*******************************************************************************
 * Secure Payload migrate type information as known to the SPD. We assume that
 * the SPD is dealing with an MP Secure Payload.
 ******************************************************************************/
#define FSP_MIGRATE_INFO		FSP_TYPE_MP

/*******************************************************************************
 * Number of cpus that the present on this platform. TODO: Rely on a topology
 * tree to determine this in the future to avoid assumptions about mpidr
 * allocation
 ******************************************************************************/
#define FSPD_CORE_COUNT		PLATFORM_CORE_COUNT

/*******************************************************************************
 * Constants that allow assembler code to preserve callee-saved registers of the
 * C runtime context while performing a security state switch.
 ******************************************************************************/
#define FSPD_C_RT_CTX_X19		0x0
#define FSPD_C_RT_CTX_X20		0x8
#define FSPD_C_RT_CTX_X21		0x10
#define FSPD_C_RT_CTX_X22		0x18
#define FSPD_C_RT_CTX_X23		0x20
#define FSPD_C_RT_CTX_X24		0x28
#define FSPD_C_RT_CTX_X25		0x30
#define FSPD_C_RT_CTX_X26		0x38
#define FSPD_C_RT_CTX_X27		0x40
#define FSPD_C_RT_CTX_X28		0x48
#define FSPD_C_RT_CTX_X29		0x50
#define FSPD_C_RT_CTX_X30		0x58
#define FSPD_C_RT_CTX_SIZE		0x60
#define FSPD_C_RT_CTX_ENTRIES		(FSPD_C_RT_CTX_SIZE >> DWORD_SHIFT)

/*******************************************************************************
 * Constants that allow assembler code to preserve caller-saved registers of the
 * SP context while performing a FSP preemption.
 * Note: These offsets have to match with the offsets for the corresponding
 * registers in cpu_context as we are using memcpy to copy the values from
 * cpu_context to sp_ctx.
 ******************************************************************************/
#define FSPD_SP_CTX_X0		0x0
#define FSPD_SP_CTX_X1		0x8
#define FSPD_SP_CTX_X2		0x10
#define FSPD_SP_CTX_X3		0x18
#define FSPD_SP_CTX_X4		0x20
#define FSPD_SP_CTX_X5		0x28
#define FSPD_SP_CTX_X6		0x30
#define FSPD_SP_CTX_X7		0x38
#define FSPD_SP_CTX_X8		0x40
#define FSPD_SP_CTX_X9		0x48
#define FSPD_SP_CTX_X10		0x50
#define FSPD_SP_CTX_X11		0x58
#define FSPD_SP_CTX_X12		0x60
#define FSPD_SP_CTX_X13		0x68
#define FSPD_SP_CTX_X14		0x70
#define FSPD_SP_CTX_X15		0x78
#define FSPD_SP_CTX_X16		0x80
#define FSPD_SP_CTX_X17		0x88
#define FSPD_SP_CTX_SIZE	0x90
#define FSPD_SP_CTX_ENTRIES		(FSPD_SP_CTX_SIZE >> DWORD_SHIFT)

#ifndef __ASSEMBLER__

#include <stdint.h>

#include <lib/cassert.h>

typedef uint32_t fsp_vector_isn_t;

typedef struct fsp_vectors {
	fsp_vector_isn_t yield_smc_entry;
	fsp_vector_isn_t fast_smc_entry;
	fsp_vector_isn_t cpu_on_entry;
	fsp_vector_isn_t cpu_off_entry;
	fsp_vector_isn_t cpu_resume_entry;
	fsp_vector_isn_t cpu_suspend_entry;
	fsp_vector_isn_t sel1_intr_entry;
	fsp_vector_isn_t system_off_entry;
	fsp_vector_isn_t system_reset_entry;
	fsp_vector_isn_t abort_yield_smc_entry;
} fsp_vectors_t;

/*
 * The number of arguments to save during a SMC call for FSP.
 * Currently only x1 and x2 are used by FSP.
 */
#define FSP_NUM_ARGS	0x2

/* AArch64 callee saved general purpose register context structure. */
DEFINE_REG_STRUCT(c_rt_regs, FSPD_C_RT_CTX_ENTRIES);

/*
 * Compile time assertion to ensure that both the compiler and linker
 * have the same double word aligned view of the size of the C runtime
 * register context.
 */
CASSERT(FSPD_C_RT_CTX_SIZE == sizeof(c_rt_regs_t),	\
	assert_spd_c_rt_regs_size_mismatch);

/* SEL1 Secure payload (SP) caller saved register context structure. */
DEFINE_REG_STRUCT(sp_ctx_regs, FSPD_SP_CTX_ENTRIES);

/*
 * Compile time assertion to ensure that both the compiler and linker
 * have the same double word aligned view of the size of the C runtime
 * register context.
 */
CASSERT(FSPD_SP_CTX_SIZE == sizeof(sp_ctx_regs_t),	\
	assert_spd_sp_regs_size_mismatch);

/*******************************************************************************
 * Structure which helps the SPD to maintain the per-cpu state of the SP.
 * 'saved_spsr_el3' - temporary copy to allow S-EL1 interrupt handling when
 *                    the FSP has been preempted.
 * 'saved_elr_el3'  - temporary copy to allow S-EL1 interrupt handling when
 *                    the FSP has been preempted.
 * 'state'          - collection of flags to track SP state e.g. on/off
 * 'mpidr'          - mpidr to associate a context with a cpu
 * 'c_rt_ctx'       - stack address to restore C runtime context from after
 *                    returning from a synchronous entry into the SP.
 * 'cpu_ctx'        - space to maintain SP architectural state
 * 'saved_fsp_args' - space to store arguments for FSP arithmetic operations
 *                    which will queried using the FSP_GET_ARGS SMC by FSP.
 * 'sp_ctx'         - space to save the SEL1 Secure Payload(SP) caller saved
 *                    register context after it has been preempted by an EL3
 *                    routed NS interrupt and when a Secure Interrupt is taken
 *                    to SP.
 ******************************************************************************/
typedef struct fsp_context {
	uint64_t saved_elr_el3;
	uint32_t saved_spsr_el3;
	uint32_t state;
	uint64_t mpidr;
	uint64_t c_rt_ctx;
	cpu_context_t cpu_ctx;
	uint64_t saved_fsp_args[FSP_NUM_ARGS];
#if FSP_NS_INTR_ASYNC_PREEMPT
	sp_ctx_regs_t sp_ctx;
#endif
} fsp_context_t;

/* Helper macros to store and retrieve fsp args from fsp_context */
#define store_fsp_args(_fsp_ctx, _x1, _x2)		do {\
				_fsp_ctx->saved_fsp_args[0] = _x1;\
				_fsp_ctx->saved_fsp_args[1] = _x2;\
			} while (0)

#define get_fsp_args(_fsp_ctx, _x1, _x2)	do {\
				_x1 = _fsp_ctx->saved_fsp_args[0];\
				_x2 = _fsp_ctx->saved_fsp_args[1];\
			} while (0)

/* FSPD power management handlers */
extern const spd_pm_ops_t fspd_pm;

/*******************************************************************************
 * Forward declarations
 ******************************************************************************/
//typedef struct fsp_vectors fsp_vectors_t;
//struct fsp_vectors;

/*******************************************************************************
 * Function & Data prototypes
 ******************************************************************************/
uint64_t fspd_enter_sp(uint64_t *c_rt_ctx);
void __dead2 fspd_exit_sp(uint64_t c_rt_ctx, uint64_t ret);
uint64_t fspd_synchronous_sp_entry(fsp_context_t *fsp_ctx);
void __dead2 fspd_synchronous_sp_exit(fsp_context_t *fsp_ctx, uint64_t ret);
void fspd_init_fsp_ep_state(struct entry_point_info *fsp_entry_point,
				uint32_t rw,
				uint64_t pc,
				fsp_context_t *fsp_ctx);
int fspd_abort_preempted_smc(fsp_context_t *fsp_ctx);

uint64_t fspd_handle_sp_preemption(void *handle);

extern fsp_context_t fspd_sp_context[FSPD_CORE_COUNT];
extern fsp_vectors_t *fsp_vectors;
#endif /*__ASSEMBLER__*/

#endif /* FSPD_PRIVATE_H */
