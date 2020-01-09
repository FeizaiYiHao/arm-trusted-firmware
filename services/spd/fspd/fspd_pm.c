/*
 * Copyright (c) 2013-2016, ARM Limited and Contributors. All rights reserved.
 *
 * SPDX-License-Identifier: BSD-3-Clause
 */

#include <assert.h>

#include <arch_helpers.h>
#include <common/bl_common.h>
#include <common/debug.h>
#include <lib/el3_runtime/context_mgmt.h>
#include <plat/common/platform.h>

#include "fsp.h"
#include "fspd_private.h"

/*******************************************************************************
 * The target cpu is being turned on. Allow the FSPD/FSP to perform any actions
 * needed. Nothing at the moment.
 ******************************************************************************/
static void fspd_cpu_on_handler(u_register_t target_cpu)
{
}

/*******************************************************************************
 * This cpu is being turned off. Allow the FSPD/FSP to perform any actions
 * needed
 ******************************************************************************/
static int32_t fspd_cpu_off_handler(u_register_t unused)
{
	int32_t rc = 0;
	uint32_t linear_id = plat_my_core_pos();
	fsp_context_t *fsp_ctx = &fspd_sp_context[linear_id];

	assert(fsp_vectors);
	assert(get_fsp_pstate(fsp_ctx->state) == FSP_PSTATE_ON);

	/*
	 * Abort any preempted SMC request before overwriting the SECURE
	 * context.
	 */
	fspd_abort_preempted_smc(fsp_ctx);

	/* Program the entry point and enter the FSP */
	cm_set_elr_el3(SECURE, (uint64_t) &fsp_vectors->cpu_off_entry);
	rc = fspd_synchronous_sp_entry(fsp_ctx);

	/*
	 * Read the response from the FSP. A non-zero return means that
	 * something went wrong while communicating with the FSP.
	 */
	if (rc != 0)
		panic();

	/*
	 * Reset FSP's context for a fresh start when this cpu is turned on
	 * subsequently.
	 */
	set_fsp_pstate(fsp_ctx->state, FSP_PSTATE_OFF);

	return 0;
}

/*******************************************************************************
 * This cpu is being suspended. S-EL1 state must have been saved in the
 * resident cpu (mpidr format) if it is a UP/UP migratable FSP.
 ******************************************************************************/
static void fspd_cpu_suspend_handler(u_register_t max_off_pwrlvl)
{
	int32_t rc = 0;
	uint32_t linear_id = plat_my_core_pos();
	fsp_context_t *fsp_ctx = &fspd_sp_context[linear_id];

	assert(fsp_vectors);
	assert(get_fsp_pstate(fsp_ctx->state) == FSP_PSTATE_ON);

	/*
	 * Abort any preempted SMC request before overwriting the SECURE
	 * context.
	 */
	fspd_abort_preempted_smc(fsp_ctx);

	/* Program the entry point and enter the FSP */
	cm_set_elr_el3(SECURE, (uint64_t) &fsp_vectors->cpu_suspend_entry);
	rc = fspd_synchronous_sp_entry(fsp_ctx);

	/*
	 * Read the response from the FSP. A non-zero return means that
	 * something went wrong while communicating with the FSP.
	 */
	if (rc)
		panic();

	/* Update its context to reflect the state the FSP is in */
	set_fsp_pstate(fsp_ctx->state, FSP_PSTATE_SUSPEND);
}

/*******************************************************************************
 * This cpu has been turned on. Enter the FSP to initialise S-EL1 and other bits
 * before passing control back to the Secure Monitor. Entry in S-EL1 is done
 * after initialising minimal architectural state that guarantees safe
 * execution.
 ******************************************************************************/
static void fspd_cpu_on_finish_handler(u_register_t unused)
{
	int32_t rc = 0;
	uint32_t linear_id = plat_my_core_pos();
	fsp_context_t *fsp_ctx = &fspd_sp_context[linear_id];
	entry_point_info_t fsp_on_entrypoint;

	assert(fsp_vectors);
	assert(get_fsp_pstate(fsp_ctx->state) == FSP_PSTATE_OFF);

	fspd_init_fsp_ep_state(&fsp_on_entrypoint,
				FSP_AARCH64,
				(uint64_t) &fsp_vectors->cpu_on_entry,
				fsp_ctx);

	/* Initialise this cpu's secure context */
	cm_init_my_context(&fsp_on_entrypoint);

#if FSP_NS_INTR_ASYNC_PREEMPT
	/*
	 * Disable the NS interrupt locally since it will be enabled globally
	 * within cm_init_my_context.
	 */
	disable_intr_rm_local(INTR_TYPE_NS, SECURE);
#endif

	/* Enter the FSP */
	rc = fspd_synchronous_sp_entry(fsp_ctx);

	/*
	 * Read the response from the FSP. A non-zero return means that
	 * something went wrong while communicating with the SP.
	 */
	if (rc != 0)
		panic();

	/* Update its context to reflect the state the SP is in */
	set_fsp_pstate(fsp_ctx->state, FSP_PSTATE_ON);
}

/*******************************************************************************
 * This cpu has resumed from suspend. The SPD saved the FSP context when it
 * completed the preceding suspend call. Use that context to program an entry
 * into the FSP to allow it to do any remaining book keeping
 ******************************************************************************/
static void fspd_cpu_suspend_finish_handler(u_register_t max_off_pwrlvl)
{
	int32_t rc = 0;
	uint32_t linear_id = plat_my_core_pos();
	fsp_context_t *fsp_ctx = &fspd_sp_context[linear_id];

	assert(fsp_vectors);
	assert(get_fsp_pstate(fsp_ctx->state) == FSP_PSTATE_SUSPEND);

	/* Program the entry point, max_off_pwrlvl and enter the SP */
	write_ctx_reg(get_gpregs_ctx(&fsp_ctx->cpu_ctx),
		      CTX_GPREG_X0,
		      max_off_pwrlvl);
	cm_set_elr_el3(SECURE, (uint64_t) &fsp_vectors->cpu_resume_entry);
	rc = fspd_synchronous_sp_entry(fsp_ctx);

	/*
	 * Read the response from the FSP. A non-zero return means that
	 * something went wrong while communicating with the FSP.
	 */
	if (rc != 0)
		panic();

	/* Update its context to reflect the state the SP is in */
	set_fsp_pstate(fsp_ctx->state, FSP_PSTATE_ON);
}

/*******************************************************************************
 * Return the type of FSP the FSPD is dealing with. Report the current resident
 * cpu (mpidr format) if it is a UP/UP migratable FSP.
 ******************************************************************************/
static int32_t fspd_cpu_migrate_info(u_register_t *resident_cpu)
{
	return FSP_MIGRATE_INFO;
}

/*******************************************************************************
 * System is about to be switched off. Allow the FSPD/FSP to perform
 * any actions needed.
 ******************************************************************************/
static void fspd_system_off(void)
{
	uint32_t linear_id = plat_my_core_pos();
	fsp_context_t *fsp_ctx = &fspd_sp_context[linear_id];

	assert(fsp_vectors);
	assert(get_fsp_pstate(fsp_ctx->state) == FSP_PSTATE_ON);

	/*
	 * Abort any preempted SMC request before overwriting the SECURE
	 * context.
	 */
	fspd_abort_preempted_smc(fsp_ctx);

	/* Program the entry point */
	cm_set_elr_el3(SECURE, (uint64_t) &fsp_vectors->system_off_entry);

	/* Enter the FSP. We do not care about the return value because we
	 * must continue the shutdown anyway */
	fspd_synchronous_sp_entry(fsp_ctx);
}

/*******************************************************************************
 * System is about to be reset. Allow the FSPD/FSP to perform
 * any actions needed.
 ******************************************************************************/
static void fspd_system_reset(void)
{
	uint32_t linear_id = plat_my_core_pos();
	fsp_context_t *fsp_ctx = &fspd_sp_context[linear_id];

	assert(fsp_vectors);
	assert(get_fsp_pstate(fsp_ctx->state) == FSP_PSTATE_ON);

	/*
	 * Abort any preempted SMC request before overwriting the SECURE
	 * context.
	 */
	fspd_abort_preempted_smc(fsp_ctx);

	/* Program the entry point */
	cm_set_elr_el3(SECURE, (uint64_t) &fsp_vectors->system_reset_entry);

	/*
	 * Enter the FSP. We do not care about the return value because we
	 * must continue the reset anyway
	 */
	fspd_synchronous_sp_entry(fsp_ctx);
}

/*******************************************************************************
 * Structure populated by the FSP Dispatcher to be given a chance to perform any
 * FSP bookkeeping before PSCI executes a power mgmt.  operation.
 ******************************************************************************/
const spd_pm_ops_t fspd_pm = {
	.svc_on = fspd_cpu_on_handler,
	.svc_off = fspd_cpu_off_handler,
	.svc_suspend = fspd_cpu_suspend_handler,
	.svc_on_finish = fspd_cpu_on_finish_handler,
	.svc_suspend_finish = fspd_cpu_suspend_finish_handler,
	.svc_migrate = NULL,
	.svc_migrate_info = fspd_cpu_migrate_info,
	.svc_system_off = fspd_system_off,
	.svc_system_reset = fspd_system_reset
};
