//! This has a main entry function fsp_main().

#![no_std]
#![feature(alloc_error_handler)] // for our own allocator implementation
#![feature(const_fn)] // for mutable references in const fn (unstable)

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

#[global_allocator]
pub static FSP_ALLOC: fsp_alloc::FspAlloc = fsp_alloc::FspAlloc::new();

#[alloc_error_handler]
fn alloc_error_handler(_layout: alloc::alloc::Layout) -> ! {
    panic!()
}

// Copied values from include/qemu_defs.h
// BL32_END is the end address of the BL32 image
// FSP_SEC_MEM_BASE is the base address for the secure DRAM
// FSP_SEC_MEM_SIZE is the size of the secure DRAM
extern "C" {
    pub fn get_bl32_end() -> u32;
}
const FSP_SEC_MEM_BASE: usize = 0x0e100000;
const FSP_SEC_MEM_SIZE: usize = 0x00f00000;

#[no_mangle]
pub extern "C" fn fsp_main() {
    debug!("fsp main");

    // For now, adding the whole available secure memory for dynamic allocation
    let mut base = unsafe { get_bl32_end() as usize };
    let mut size = FSP_SEC_MEM_SIZE;
    if base > FSP_SEC_MEM_BASE {
        size = FSP_SEC_MEM_SIZE - (base - FSP_SEC_MEM_BASE);
    } else {
        base = FSP_SEC_MEM_BASE;
    };
    FSP_ALLOC.init(base, size);
    use alloc::boxed::Box;
    let x = Box::new(10);
    let y = Box::new(1000);
    let val_x: u32 = *x;
    let val_y: u32 = *y;
    if val_x == 10 {
        debug!("val_x is 10");
    } else {
        debug!("val_x is not 10");
    }
    if val_y == 1000 {
        debug!("val_y is 1000");
    } else {
        debug!("val_y is not 1000");
    }
    debug!("fsp main done");
}
