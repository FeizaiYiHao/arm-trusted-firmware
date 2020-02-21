//! This has a main entry function fsp_main().

#![no_std]
#![feature(alloc_error_handler)] // for our own allocator implementation
#![feature(const_fn)] // for mutable references in const fn (unstable)

#[rustfmt::skip] // the log module defines macros used by fsp_allocator, so it has to come first.

mod log;
mod console;
mod extern_c_defs;
mod fsp_alloc;
mod qemu_constants;

extern crate alloc; // need this due to #![no_std]---for regular Rust, it is by default.

use core::panic::PanicInfo;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    static_debug!("Panic");
    loop {}
}

#[global_allocator]
pub static FSP_ALLOC: fsp_alloc::FspAlloc = fsp_alloc::FspAlloc::new();

#[alloc_error_handler]
fn alloc_error_handler(_layout: alloc::alloc::Layout) -> ! {
    panic!()
}

// TODO: Find a way to avoid static mut
static mut FSP_CONSOLE: console::FspConsole = console::FspConsole::new();

///! This is the initialization function that should be called first before anything else.
fn fsp_init() {
    unsafe {
        FSP_CONSOLE.init();
    }

    // For now, adding the whole available secure memory for dynamic allocation
    let mut base = unsafe { &extern_c_defs::__BL32_END__ as *const u32 as usize };
    let mut size = qemu_constants::SEC_DRAM_SIZE;
    if base > qemu_constants::SEC_DRAM_BASE {
        size = qemu_constants::SEC_DRAM_SIZE - (base - qemu_constants::SEC_DRAM_BASE);
    } else {
        base = qemu_constants::SEC_DRAM_BASE;
    };
    FSP_ALLOC.init(base, size);
}

///! This is the actual main function that extern_c_defs::fsp_main_wrapper() calls.
fn fsp_main() {
    fsp_init();

    debug!("fsp main");

    mem_test();

    debug!(
        "BL32_END: {}",
        &extern_c_defs::__BL32_END__ as *const u32 as usize
    );
    debug!("fsp main done");
}

fn mem_test() {
    use alloc::boxed::Box;
    use alloc::string::String;
    use alloc::string::ToString;
    use alloc::vec::Vec;

    let mut n_lst: Vec<Box<u32>> = Vec::new();
    let mut s_lst: Vec<String> = Vec::new();
    debug!("mem_test inserting");
    for number in 0..10000 {
        let x = Box::<u32>::new(number);
        n_lst.push(x);
        s_lst.push(number.to_string());
    }

    for number in 0..5000 {
        if let Some(n) = n_lst.pop() {
            if let Some(s) = s_lst.pop() {
                assert_eq!(*n, s.parse::<u32>().unwrap());
                debug!("number: {}", *n);
            } else {
                panic!("none");
            }
        } else {
            panic!("none");
        }
        let x = Box::<u32>::new(number);
        n_lst.insert(number as usize, x);
        s_lst.insert(number as usize, number.to_string());
    }

    for number in 0..5000 {
        let x = Box::<u32>::new(number);
        n_lst.push(x);
        s_lst.push(number.to_string());
        let n = n_lst.remove((number * 2) as usize);
        let s = s_lst.remove((number * 2) as usize);
        assert_eq!(*n, s.parse::<u32>().unwrap());
        debug!("number: {}", *n);
    }

    while let Some(n) = n_lst.pop() {
        if let Some(s) = s_lst.pop() {
            assert_eq!(*n, s.parse::<u32>().unwrap());
            debug!("number: {}", *n);
        } else {
            panic!("none");
        }
    }

    debug!("mem_test done");
}
