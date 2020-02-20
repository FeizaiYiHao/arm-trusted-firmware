extern "C" {
    pub fn get_bl32_end() -> u32;

    pub fn console_pl011_register(
        baseaddr: *const u8,
        clock: u32,
        baud: u32,
        console: *const u8,
    ) -> isize;
}
