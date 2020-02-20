extern "C" {
    pub fn get_bl32_end() -> u32;

    pub fn console_pl011_register(
        baseaddr: *const u8,
        clock: u32,
        baud: u32,
        console: *const u8,
    ) -> isize;

    pub static fsp_vector_table: FspVectors;
}

/*
/* Vector table of jumps */
extern fsp_vectors_t fsp_vector_table;

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
*/

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
