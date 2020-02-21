///! This is the main wrapper function that fsp_entrypoint.S calls.
#[no_mangle]
pub extern "C" fn fsp_main_wrapper() -> *const FspVectors {
    crate::fsp_main();

    unsafe { &fsp_vector_table as *const FspVectors }
}

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

    pub static __BL32_END__: u32;

    pub fn strncmp(s1: *const u8, s2: *const u8, n: u32) -> u32;
}

/*
 * Rust's libcore calls this function but TF-A's libc doesn't have it.
 */
#[no_mangle]
pub extern "C" fn bcmp(s1: *const u8, s2: *const u8, n: u32) -> u32 {
    unsafe { strncmp(s1, s2, n) }
}
