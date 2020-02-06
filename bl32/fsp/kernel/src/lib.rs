//!
//! This has a main entry function fsp_main().
//!
#![no_std]
#![feature(alloc_error_handler)] // for our own allocator implementation

#[rustfmt::skip] // the log module defines macros used by fsp_allocator, so it has to come first.
mod log;
mod fsp_alloc;

extern crate alloc; // need this due to #![no_std]---for regular Rust, it is by default.

use core::panic::PanicInfo;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    debug!("Panic");
    loop {}
}

#[no_mangle]
pub extern "C" fn fsp_main() {
    debug!("fsp main");

    use alloc::boxed::Box;
    let x = Box::new(10);
    let val: u8 = *x;
    if val == 10 {
        debug!("val is 10");
    } else {
        debug!("val is not 10");
    }
    debug!("Used Box");
}
