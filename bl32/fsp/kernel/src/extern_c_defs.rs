///! This is the main wrapper function that fsp_entrypoint.S calls.
#[no_mangle]
pub extern "C" fn fsp_main_wrapper() -> *const FspVectors {
    crate::fsp_main();

    unsafe { &fsp_vector_table as *const FspVectors }
}

// TODO: move this to a proper file later
#[repr(C)]
pub struct FspVectors {
    yield_smc_entry: u32,
    fast_smc_entry: u32,
    cpu_on_entry: u32,
    cpu_off_entry: u32,
    cpu_resume_entry: u32,
    cpu_suspend_entry: u32,
    sel1_intr_entry: u32,
    system_off_entry: u32,
    system_reset_entry: u32,
    abort_yield_smc_entry: u32,
}

extern "C" {
    pub fn console_pl011_register(
        baseaddr: *const u8,
        clock: u32,
        baud: u32,
        console: *const u8,
    ) -> isize;

    pub static fsp_vector_table: FspVectors;

    pub static __BL32_END__: u32; // This is a linker symbol, so its reference is the correct value.

    pub fn strncmp(s1: *const u8, s2: *const u8, n: u32) -> u32;
}

// Rust's libcore calls this function but TF-A's libc doesn't have it.
#[no_mangle]
pub extern "C" fn bcmp(s1: *const u8, s2: *const u8, n: u32) -> u32 {
    unsafe { strncmp(s1, s2, n) }
}

// It should be the following, but Rust doesn't allow a const in align yet.
//#[repr(C, align(crate::qemu_constants::CACHE_WRITEBACK_GRANULE))]
#[repr(C, align(64))]
pub struct FspArgs {
    regs: [u64; FSP_ARGS_END >> 3],
}

/* Definitions to help the assembler access the SMC/ERET args structure */
//pub const FSP_ARGS_SIZE: usize = 0x40;
pub const FSP_ARG0: usize = 0x0;
pub const FSP_ARG1: usize = 0x8;
pub const FSP_ARG2: usize = 0x10;
pub const FSP_ARG3: usize = 0x18;
pub const FSP_ARG4: usize = 0x20;
pub const FSP_ARG5: usize = 0x28;
pub const FSP_ARG6: usize = 0x30;
pub const FSP_ARG7: usize = 0x38;
pub const FSP_ARGS_END: usize = 0x40;

impl FspArgs {
    const fn new() -> FspArgs {
        FspArgs {
            regs: [0; FSP_ARGS_END >> 3],
        }
    }
}

///! Per cpu data structure to populate parameters for an SMC in C code and use
///! a pointer to this structure in assembler code to populate x0-x7
// TODO: avoid static mut (unstable)
static mut FSP_SMC_ARGS: [FspArgs; crate::qemu_constants::PLATFORM_CORE_COUNT] =
    [FspArgs::new(); crate::qemu_constants::PLATFORM_CORE_COUNT];

//#define write_sp_arg(args, offset, val) (((args)->_regs[offset >> 3]) = val)
fn write_sp_arg(args: &mut FspArgs, offset: usize, val: u64) {
    args.regs[offset >> 3] = val;
}

#[no_mangle]
extern "C" fn set_smc_args(
    arg0: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    arg5: u64,
    arg6: u64,
    arg7: u64,
) -> &'static FspArgs {
    let linear_id: usize = 0; // TODO: should call plat_my_core_pos(), i.e., no SMP for now
    let pcpu_smc_args: &mut FspArgs = unsafe { &mut FSP_SMC_ARGS[linear_id] };
    write_sp_arg(pcpu_smc_args, FSP_ARG0, arg0);
    write_sp_arg(pcpu_smc_args, FSP_ARG1, arg1);
    write_sp_arg(pcpu_smc_args, FSP_ARG2, arg2);
    write_sp_arg(pcpu_smc_args, FSP_ARG3, arg3);
    write_sp_arg(pcpu_smc_args, FSP_ARG4, arg4);
    write_sp_arg(pcpu_smc_args, FSP_ARG5, arg5);
    write_sp_arg(pcpu_smc_args, FSP_ARG6, arg6);
    write_sp_arg(pcpu_smc_args, FSP_ARG7, arg7);

    pcpu_smc_args
}
///! SMC function IDs that FSP uses to signal various forms of completions
///! to the secure payload dispatcher.
//pub const FSP_ENTRY_DONE: u64 = 0xf2000000;
pub const FSP_ON_DONE: u64 = 0xf2000001;
pub const FSP_OFF_DONE: u64 = 0xf2000002;
pub const FSP_SUSPEND_DONE: u64 = 0xf2000003;
pub const FSP_RESUME_DONE: u64 = 0xf2000004;
//pub const FSP_PREEMPTED: u64 = 0xf2000005;
pub const FSP_ABORT_DONE: u64 = 0xf2000007;
pub const FSP_SYSTEM_OFF_DONE: u64 = 0xf2000008;
pub const FSP_SYSTEM_RESET_DONE: u64 = 0xf2000009;

/*******************************************************************************
 * This function performs any remaining book keeping in the test secure payload
 * after this cpu's architectural state has been setup in response to an earlier
 * psci cpu_on request.
 ******************************************************************************/
#[no_mangle]
pub extern "C" fn cpu_on_main_wrapper() -> &'static FspArgs {
    /* Indicate to the SPD that we have completed turned ourselves on */
    set_smc_args(FSP_ON_DONE, 0, 0, 0, 0, 0, 0, 0)
}

/*******************************************************************************
 * This function performs any remaining book keeping in the test secure payload
 * before this cpu is turned off in response to a psci cpu_off request.
 ******************************************************************************/
#[no_mangle]
pub extern "C" fn cpu_off_main_wrapper(
    _arg0: u64,
    _arg1: u64,
    _arg2: u64,
    _arg3: u64,
    _arg4: u64,
    _arg5: u64,
    _arg6: u64,
    _arg7: u64,
) -> &'static FspArgs {
    /* Indicate to the SPD that we have completed this request */
    set_smc_args(FSP_OFF_DONE, 0, 0, 0, 0, 0, 0, 0)
}

/*******************************************************************************
 * This function performs any book keeping in the test secure payload before
 * this cpu's architectural state is saved in response to an earlier psci
 * cpu_suspend request.
 ******************************************************************************/
#[no_mangle]
pub extern "C" fn cpu_suspend_main_wrapper(
    _arg0: u64,
    _arg1: u64,
    _arg2: u64,
    _arg3: u64,
    _arg4: u64,
    _arg5: u64,
    _arg6: u64,
    _arg7: u64,
) -> &'static FspArgs {
    /* Indicate to the SPD that we have completed this request */
    set_smc_args(FSP_SUSPEND_DONE, 0, 0, 0, 0, 0, 0, 0)
}

/*******************************************************************************
 * This function performs any book keeping in the test secure payload after this
 * cpu's architectural state has been restored after wakeup from an earlier psci
 * cpu_suspend request.
 ******************************************************************************/
#[no_mangle]
pub extern "C" fn cpu_resume_main_wrapper(
    _max_off_pwrlvl: u64,
    _arg1: u64,
    _arg2: u64,
    _arg3: u64,
    _arg4: u64,
    _arg5: u64,
    _arg6: u64,
    _arg7: u64,
) -> &'static FspArgs {
    /* Indicate to the SPD that we have completed this request */
    set_smc_args(FSP_RESUME_DONE, 0, 0, 0, 0, 0, 0, 0)
}

/*******************************************************************************
 * This function performs any remaining bookkeeping in the test secure payload
 * before the system is switched off (in response to a psci SYSTEM_OFF request)
 ******************************************************************************/
#[no_mangle]
pub extern "C" fn system_off_main_wrapper(
    _arg0: u64,
    _arg1: u64,
    _arg2: u64,
    _arg3: u64,
    _arg4: u64,
    _arg5: u64,
    _arg6: u64,
    _arg7: u64,
) -> &'static FspArgs {
    /* Indicate to the SPD that we have completed this request */
    set_smc_args(FSP_SYSTEM_OFF_DONE, 0, 0, 0, 0, 0, 0, 0)
}

/*******************************************************************************
 * This function performs any remaining bookkeeping in the test secure payload
 * before the system is reset (in response to a psci SYSTEM_RESET request)
 ******************************************************************************/
#[no_mangle]
pub extern "C" fn system_reset_main_wrapper(
    _arg0: u64,
    _arg1: u64,
    _arg2: u64,
    _arg3: u64,
    _arg4: u64,
    _arg5: u64,
    _arg6: u64,
    _arg7: u64,
) -> &'static FspArgs {
    /* Indicate to the SPD that we have completed this request */
    set_smc_args(FSP_SYSTEM_RESET_DONE, 0, 0, 0, 0, 0, 0, 0)
}

/*******************************************************************************
 * FSP fast smc handler. The secure monitor jumps to this function by
 * doing the ERET after populating X0-X7 registers. The arguments are received
 * in the function arguments in order. Once the service is rendered, this
 * function returns to Secure Monitor by raising SMC.
 ******************************************************************************/
#[no_mangle]
pub extern "C" fn smc_handler_wrapper(
    func: u64,
    arg1: u64,
    arg2: u64,
    _arg3: u64,
    _arg4: u64,
    _arg5: u64,
    _arg6: u64,
    _arg7: u64,
) -> &'static FspArgs {
    /* Indicate to the SPD that we have completed this request */
    set_smc_args(func, 0, arg1, arg2, 0, 0, 0, 0)
}

/*******************************************************************************
 * FSP smc abort handler. This function is called when aborting a preempted
 * yielding SMC request. It should cleanup all resources owned by the SMC
 * handler such as locks or dynamically allocated memory so following SMC
 * request are executed in a clean environment.
 ******************************************************************************/
#[no_mangle]
pub extern "C" fn abort_smc_handler_wrapper(
    _func: u64,
    _arg1: u64,
    _arg2: u64,
    _arg3: u64,
    _arg4: u64,
    _arg5: u64,
    _arg6: u64,
    _arg7: u64,
) -> &'static FspArgs {
    set_smc_args(FSP_ABORT_DONE, 0, 0, 0, 0, 0, 0, 0)
}

#[no_mangle]
pub extern "C" fn update_sync_sel1_intr_stats_wrapper(_t: u32, _elr_el3: u64) {}

#[no_mangle]
pub extern "C" fn common_int_handler_wrapper() -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn plat_panic_handler_wrapper() -> ! {
    panic!("fsp_plat_panic_handler");
}
