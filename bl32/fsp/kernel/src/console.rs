extern "C" {
    pub fn fsp_qemu_console_init();
}

pub fn fsp_console_init() {
    unsafe { fsp_qemu_console_init() }
}
