#![no_std]

use core::panic::PanicInfo;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

extern "C" {
    fn fsp_print_debug_message();
}

#[no_mangle]
pub extern "C" fn fsp_main() {
    unsafe {
        fsp_print_debug_message();
    }
}

//#[no_mangle]
//pub extern "C" fn fsp_common_int_handler() -> i32 {
//    panic!("fsp_common_int_handler() called");
//}
//
//#[no_mangle]
//pub extern "C" fn plat_panic_handler() -> i32 {
//    panic!("fsp_common_int_handler() called");
//}
