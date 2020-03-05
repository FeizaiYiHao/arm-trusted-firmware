//! Since we're creating a library, this is the main file from Rust's point of view.
//! However, our actual main file is entrypoints.rs.
//!
//! This file mostly contains all asm-facing definitions and functions, as well as Rust
//! initialization.
//!
//! Bad things that should not occur:
//!
//! - Accessing unsafe variables (defined by asm) or making unsafe calls (defined by asm) without
//! observing Rust's borrow checker rules.
//! - Not using correct types (that asm expects) from the Rust side (e.g., primitive types, structs,
//! arrays, etc.)
//! - Wrong type casting when passing references and pointers which is related to the above
//! correct type issue

#![no_std]
#![feature(alloc_error_handler)] // for our own allocator implementation
#![feature(const_fn)] // for mutable references in const fn (unstable)
#![feature(const_in_array_repeat_expressions)] // for initializing an array with a repeated const fn

#[rustfmt::skip] // the log module defines macros used by fsp_allocator, so it has to come first.

mod log;
mod console;
mod entrypoints;
mod fsp_alloc;
mod qemu_constants;

/// SMC function IDs that FSP uses to signal various forms of completions
/// to the secure payload dispatcher.
//#[no_mangle]
//pub static FSP_ENTRY_DONE: u64 = 0xf2000000; // currently only used by asm
#[no_mangle]
pub static FSP_ON_DONE: u64 = 0xf2000001;
#[no_mangle]
pub static FSP_OFF_DONE: u64 = 0xf2000002;
#[no_mangle]
pub static FSP_SUSPEND_DONE: u64 = 0xf2000003;
#[no_mangle]
pub static FSP_RESUME_DONE: u64 = 0xf2000004;
#[no_mangle]
pub static FSP_PREEMPTED: u64 = 0xf2000005;
#[no_mangle]
pub static FSP_ABORT_DONE: u64 = 0xf2000007;
#[no_mangle]
pub static FSP_SYSTEM_OFF_DONE: u64 = 0xf2000008;
#[no_mangle]
pub static FSP_SYSTEM_RESET_DONE: u64 = 0xf2000009;
//#[no_mangle]
//pub static FSP_HANDLED_S_EL1_INTR: u64 = 0xf2000006; // currently only used by asm
//#[no_mangle]
//pub static FSP_HANDLE_SEL1_INTR_AND_RETURN: u64 = 0x2004; // currently only used by asm

/// Definitions to help the assembler access the SMC/ERET args structure
// TODO: These are currently duplicated from fsp_private.h
//pub const FSP_ARGS_SIZE: usize = 0x40; // currently only used by asm
pub const FSP_ARG0: usize = 0x0;
pub const FSP_ARG1: usize = 0x8;
pub const FSP_ARG2: usize = 0x10;
pub const FSP_ARG3: usize = 0x18;
pub const FSP_ARG4: usize = 0x20;
pub const FSP_ARG5: usize = 0x28;
pub const FSP_ARG6: usize = 0x30;
pub const FSP_ARG7: usize = 0x38;
pub const FSP_ARGS_END: usize = 0x40;

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

/// Per cpu data structure to populate parameters for an SMC in C code and use
/// a pointer to this structure in assembler code to populate x0-x7
// TODO: avoid static mut (unstable)
static mut FSP_SMC_ARGS: [FspArgs; crate::qemu_constants::PLATFORM_CORE_COUNT] =
    [FspArgs::new(); crate::qemu_constants::PLATFORM_CORE_COUNT];

// It should be the following, but Rust doesn't allow a const in align yet.
//#[repr(C, align(crate::qemu_constants::CACHE_WRITEBACK_GRANULE))]
#[repr(C, align(64))]
pub struct FspArgs {
    regs: [u64; FSP_ARGS_END >> 3],
}

impl FspArgs {
    const fn new() -> FspArgs {
        FspArgs {
            regs: [0; FSP_ARGS_END >> 3],
        }
    }
}

/// This is the main wrapper function that fsp_entrypoint.S calls.
#[no_mangle]
pub extern "C" fn fsp_main_wrapper() -> *const FspVectors {
    entrypoints::fsp_main();

    unsafe { &fsp_vector_table as *const FspVectors }
}

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
    let linear_id: u32 = unsafe { plat_my_core_pos() };
    let pcpu_smc_args: &mut FspArgs = unsafe { &mut FSP_SMC_ARGS[linear_id as usize] };
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

/// This function performs any remaining book keeping in the test secure payload
/// after this cpu's architectural state has been setup in response to an earlier
/// psci cpu_on request.
#[no_mangle]
pub extern "C" fn cpu_on_main_wrapper() -> &'static FspArgs {
    /* Indicate to the SPD that we have completed turned ourselves on */
    set_smc_args(FSP_ON_DONE, 0, 0, 0, 0, 0, 0, 0)
}

/// This function performs any remaining book keeping in the test secure payload
/// before this cpu is turned off in response to a psci cpu_off request.
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

/// This function performs any book keeping in the test secure payload before
/// this cpu's architectural state is saved in response to an earlier psci
/// cpu_suspend request.
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

/// This function performs any book keeping in the test secure payload after this
/// cpu's architectural state has been restored after wakeup from an earlier psci
/// cpu_suspend request.
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

/// This function performs any remaining bookkeeping in the test secure payload
/// before the system is switched off (in response to a psci SYSTEM_OFF request)
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

/// This function performs any remaining bookkeeping in the test secure payload
/// before the system is reset (in response to a psci SYSTEM_RESET request)
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

/// FSP fast smc handler. The secure monitor jumps to this function by
/// doing the ERET after populating X0-X7 registers. The arguments are received
/// in the function arguments in order. Once the service is rendered, this
/// function returns to Secure Monitor by raising SMC.
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

/// FSP smc abort handler. This function is called when aborting a preempted
/// yielding SMC request. It should cleanup all resources owned by the SMC
/// handler such as locks or dynamically allocated memory so following SMC
/// request are executed in a clean environment.
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

/// This function updates the FSP statistics for S-EL1 interrupts handled
/// synchronously i.e the ones that have been handed over by the FSPD. It also
/// keeps count of the number of times control was passed back to the FSPD
/// after handling the interrupt. In the future it will be possible that the
/// FSPD hands over an S-EL1 interrupt to the FSP but does not expect it to
/// return execution. This statistic will be useful to distinguish between these
/// two models of synchronous S-EL1 interrupt handling. The 'elr_el3' parameter
/// contains the address of the instruction in normal world where this S-EL1
/// interrupt was generated.
#[no_mangle]
pub extern "C" fn update_sync_sel1_intr_stats_wrapper(_t: u32, _elr_el3: u64) {}

/// This function is invoked when a non S-EL1 interrupt is received and causes
/// the preemption of FSP. This function returns FSP_PREEMPTED and results
/// in the control being handed over to EL3 for handling the interrupt.
#[no_mangle]
pub extern "C" fn handle_preemption() -> i32 {
    return FSP_PREEMPTED as i32;
}

/// common_int_handler is called as a part of both synchronous and
/// asynchronous handling of FSP interrupts. Currently the physical timer
/// interrupt is the only S-EL1 interrupt that this handler expects. It returns
/// 0 upon successfully handling the expected interrupt and all other
/// interrupts are treated as normal world or EL3 interrupts.
#[no_mangle]
pub extern "C" fn common_int_handler_wrapper() -> i32 {
    0
}

/// panic_handler for the assembly
#[no_mangle]
pub extern "C" fn plat_panic_handler_wrapper() -> ! {
    panic!("plat_panic_handler");
}

extern "C" {
    fn console_pl011_register(
        baseaddr: *const u8,
        clock: u32,
        baud: u32,
        console: *const u8,
    ) -> isize;

    static __BL32_END__: u32; // This is a linker symbol, so its reference is the correct value.

    fn strncmp(s1: *const u8, s2: *const u8, n: u32) -> u32;

    static fsp_vector_table: FspVectors;

    fn plat_my_core_pos() -> u32;
}

pub fn bl32_end() -> usize {
    unsafe { &__BL32_END__ as *const u32 as usize }
}

// Rust's libcore calls this function but TF-A's libc doesn't have it.
#[no_mangle]
pub extern "C" fn bcmp(s1: *const u8, s2: *const u8, n: u32) -> u32 {
    unsafe { strncmp(s1, s2, n) }
}
