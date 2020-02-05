//!
//! This is a dynamic memory allocator.
//!
extern crate alloc; // need this due to #![no_std]---for regular Rust, it is by default.

use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

struct FSPAllocator; // a zero-sized struct

unsafe impl GlobalAlloc for FSPAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        debug!("alloc");
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        debug!("dealloc")
    }
}

#[global_allocator]
static A: FSPAllocator = FSPAllocator;

#[alloc_error_handler]
fn alloc_error_handler(_layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error")
}
