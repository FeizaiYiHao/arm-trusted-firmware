/*
 * Copyright (c) 2013-2019, ARM Limited and Contributors. All rights reserved.
 *
 * SPDX-License-Identifier: BSD-3-Clause
 */

/*
 * Stripped down from TF-A's bl32/tsp/tsp_main.c
 */

#include <common/debug.h>
#include <common/bl_common.h> // for BL32_END
#include "../include/qemu_defs.h"
#include "../include/fsp_private.h"
#include "../include/fsp.h"

void fsp_print_debug_message(void)
{
    NOTICE("BL32: Debug message\n");
}

uint32_t get_bl32_end(void)
{
    return BL32_END;
}

/*
 * Rust's libcore calls this function but TF-A's libc doesn't have it.
 */
int bcmp(const void *s1, const void *s2, size_t n) {
    return strncmp((const char *) s1, (const char *) s2, n);
}

/*******************************************************************************
 * Per cpu data structure to populate parameters for an SMC in C code and use
 * a pointer to this structure in assembler code to populate x0-x7
 ******************************************************************************/
static fsp_args_t fsp_smc_args[PLATFORM_CORE_COUNT];

/*******************************************************************************
 * The FSP memory footprint starts at address BL32_BASE and ends with the
 * linker symbol __BL32_END__. Use these addresses to compute the FSP image
 * size.
 ******************************************************************************/
#define BL32_TOTAL_LIMIT BL32_END
#define BL32_TOTAL_SIZE (BL32_TOTAL_LIMIT - (unsigned long) BL32_BASE)

static fsp_args_t *set_smc_args(uint64_t arg0,
                 uint64_t arg1,
                 uint64_t arg2,
                 uint64_t arg3,
                 uint64_t arg4,
                 uint64_t arg5,
                 uint64_t arg6,
                 uint64_t arg7)
{
    uint32_t linear_id;
    fsp_args_t *pcpu_smc_args;

    /*
     * Return to Secure Monitor by raising an SMC. The results of the
     * service are passed as an arguments to the SMC
     */
    linear_id = 0; //plat_my_core_pos(); // No SMP for now
    pcpu_smc_args = &fsp_smc_args[linear_id];
    write_sp_arg(pcpu_smc_args, FSP_ARG0, arg0);
    write_sp_arg(pcpu_smc_args, FSP_ARG1, arg1);
    write_sp_arg(pcpu_smc_args, FSP_ARG2, arg2);
    write_sp_arg(pcpu_smc_args, FSP_ARG3, arg3);
    write_sp_arg(pcpu_smc_args, FSP_ARG4, arg4);
    write_sp_arg(pcpu_smc_args, FSP_ARG5, arg5);
    write_sp_arg(pcpu_smc_args, FSP_ARG6, arg6);
    write_sp_arg(pcpu_smc_args, FSP_ARG7, arg7);

    return pcpu_smc_args;
}

/*******************************************************************************
 * Setup function for FSP.
 ******************************************************************************/
void fsp_setup(void)
{
    /* Perform early platform-specific setup */
    //fsp_early_platform_setup();

    /* Perform late platform-specific setup */
    //fsp_plat_arch_setup();
}

/*******************************************************************************
 * FSP main entry point where it gets the opportunity to initialize its secure
 * state/applications. Once the state is initialized, it must return to the
 * SPD with a pointer to the 'fsp_vector_table' jump table.
 ******************************************************************************/
uint64_t fsp_c_main(void)
{
    //NOTICE("FSP: %s\n", version_string);
    //NOTICE("FSP: %s\n", build_message);
    //INFO("FSP: Total memory base : 0x%lx\n", (unsigned long) BL32_BASE);
    //INFO("FSP: Total memory size : 0x%lx bytes\n", BL32_TOTAL_SIZE);
	/*
	 * Initialize a console. From plat/arm/common/tsp/arm_tsp_setup.c
	 */
    fsp_qemu_console_init();
    fsp_main();
    return (uint64_t) &fsp_vector_table;
}

/*******************************************************************************
 * This function performs any remaining book keeping in the test secure payload
 * after this cpu's architectural state has been setup in response to an earlier
 * psci cpu_on request.
 ******************************************************************************/
fsp_args_t *fsp_cpu_on_main(void)
{
    /* Indicate to the SPD that we have completed turned ourselves on */
    return set_smc_args(FSP_ON_DONE, 0, 0, 0, 0, 0, 0, 0);
}

/*******************************************************************************
 * This function performs any remaining book keeping in the test secure payload
 * before this cpu is turned off in response to a psci cpu_off request.
 ******************************************************************************/
fsp_args_t *fsp_cpu_off_main(uint64_t arg0,
               uint64_t arg1,
               uint64_t arg2,
               uint64_t arg3,
               uint64_t arg4,
               uint64_t arg5,
               uint64_t arg6,
               uint64_t arg7)
{
    /* Indicate to the SPD that we have completed this request */
    return set_smc_args(FSP_OFF_DONE, 0, 0, 0, 0, 0, 0, 0);
}

/*******************************************************************************
 * This function performs any book keeping in the test secure payload before
 * this cpu's architectural state is saved in response to an earlier psci
 * cpu_suspend request.
 ******************************************************************************/
fsp_args_t *fsp_cpu_suspend_main(uint64_t arg0,
                   uint64_t arg1,
                   uint64_t arg2,
                   uint64_t arg3,
                   uint64_t arg4,
                   uint64_t arg5,
                   uint64_t arg6,
                   uint64_t arg7)
{
    /* Indicate to the SPD that we have completed this request */
    return set_smc_args(FSP_SUSPEND_DONE, 0, 0, 0, 0, 0, 0, 0);
}

/*******************************************************************************
 * This function performs any book keeping in the test secure payload after this
 * cpu's architectural state has been restored after wakeup from an earlier psci
 * cpu_suspend request.
 ******************************************************************************/
fsp_args_t *fsp_cpu_resume_main(uint64_t max_off_pwrlvl,
                  uint64_t arg1,
                  uint64_t arg2,
                  uint64_t arg3,
                  uint64_t arg4,
                  uint64_t arg5,
                  uint64_t arg6,
                  uint64_t arg7)
{
    /* Indicate to the SPD that we have completed this request */
    return set_smc_args(FSP_RESUME_DONE, 0, 0, 0, 0, 0, 0, 0);
}

/*******************************************************************************
 * This function performs any remaining bookkeeping in the test secure payload
 * before the system is switched off (in response to a psci SYSTEM_OFF request)
 ******************************************************************************/
fsp_args_t *fsp_system_off_main(uint64_t arg0,
                uint64_t arg1,
                uint64_t arg2,
                uint64_t arg3,
                uint64_t arg4,
                uint64_t arg5,
                uint64_t arg6,
                uint64_t arg7)
{
    /* Indicate to the SPD that we have completed this request */
    return set_smc_args(FSP_SYSTEM_OFF_DONE, 0, 0, 0, 0, 0, 0, 0);
}

/*******************************************************************************
 * This function performs any remaining bookkeeping in the test secure payload
 * before the system is reset (in response to a psci SYSTEM_RESET request)
 ******************************************************************************/
fsp_args_t *fsp_system_reset_main(uint64_t arg0,
                uint64_t arg1,
                uint64_t arg2,
                uint64_t arg3,
                uint64_t arg4,
                uint64_t arg5,
                uint64_t arg6,
                uint64_t arg7)
{
    /* Indicate to the SPD that we have completed this request */
    return set_smc_args(FSP_SYSTEM_RESET_DONE, 0, 0, 0, 0, 0, 0, 0);
}

/*******************************************************************************
 * FSP fast smc handler. The secure monitor jumps to this function by
 * doing the ERET after populating X0-X7 registers. The arguments are received
 * in the function arguments in order. Once the service is rendered, this
 * function returns to Secure Monitor by raising SMC.
 ******************************************************************************/
fsp_args_t *fsp_smc_handler(uint64_t func,
                   uint64_t arg1,
                   uint64_t arg2,
                   uint64_t arg3,
                   uint64_t arg4,
                   uint64_t arg5,
                   uint64_t arg6,
                   uint64_t arg7)
{
    uint64_t results[2];
    results[0] = arg1;
    results[1] = arg2;

    return set_smc_args(func, 0,
                results[0],
                results[1],
                0, 0, 0, 0);
}

/*******************************************************************************
 * FSP smc abort handler. This function is called when aborting a preempted
 * yielding SMC request. It should cleanup all resources owned by the SMC
 * handler such as locks or dynamically allocated memory so following SMC
 * request are executed in a clean environment.
 ******************************************************************************/
fsp_args_t *fsp_abort_smc_handler(uint64_t func,
                  uint64_t arg1,
                  uint64_t arg2,
                  uint64_t arg3,
                  uint64_t arg4,
                  uint64_t arg5,
                  uint64_t arg6,
                  uint64_t arg7)
{
    return set_smc_args(FSP_ABORT_DONE, 0, 0, 0, 0, 0, 0, 0);
}

void fsp_update_sync_sel1_intr_stats(uint32_t type, uint64_t elr_el3) {
    return;
}

int32_t fsp_common_int_handler(void) {
    return 0;
}

int32_t fsp_plat_panic_handler(void) {
    return 0;
}
