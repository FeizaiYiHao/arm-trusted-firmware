//!
//! This is a dynamic memory allocator.
//!
extern crate alloc; // need this due to #![no_std]---for regular Rust, it is by default.

use alloc::alloc::{handle_alloc_error, GlobalAlloc, Layout};

// Copied values from include/qemu_defs.h
// BL32_END is the end address of the BL32 image
// FSP_SEC_MEM_BASE is the base address for the secure DRAM
// FSP_SEC_MEM_SIZE is the size of the secure DRAM
extern "C" {
    pub fn get_bl32_end() -> u32;
}

static FSP_SEC_MEM_BASE: usize = 0x0e100000;
static FSP_SEC_MEM_SIZE: usize = 0x00f00000;

struct FSPAlloc; // a zero-sized struct

unsafe impl GlobalAlloc for FSPAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        debug!("alloc");
        let buf_base = FSP_SEC_MEM_BASE + FSP_SEC_MEM_SIZE - layout.size();
        if buf_base < get_bl32_end() as usize {
            debug!("buf_base smaller than BL32_END");
            handle_alloc_error(layout);
        }
        buf_base as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        debug!("dealloc")
    }
}

#[global_allocator]
static A: FSPAlloc = FSPAlloc;

#[alloc_error_handler]
fn alloc_error_handler(_layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error")
}
