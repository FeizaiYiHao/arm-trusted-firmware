//! This is the actual main file. This contains all entrypoint functions.

extern crate alloc; // need this due to #![no_std]---for regular Rust, it is by default.

use crate::console;
use crate::debug;
use crate::fsp_alloc;
use crate::fsp_slab;
use crate::qemu_constants;

/// Custom global allocator
#[global_allocator]
pub static FSP_ALLOC: fsp_alloc::FspAlloc = fsp_alloc::FspAlloc::new();

pub static mut FSP_SLAB : fsp_slab::KmemCache = fsp_slab::KmemCache::new();
/// Global console
// TODO: Find a way to avoid static mut
pub static mut FSP_CONSOLE: console::FspConsole = console::FspConsole::new();

/// This is the initialization function that should be called first before anything else.
fn fsp_init() {
    unsafe {
        FSP_CONSOLE.init();
    }

    // For now, adding the whole available secure memory for dynamic allocation
    let mut base = crate::bl32_end();
    let mut size = qemu_constants::BL32_MEM_SIZE;
    if base > qemu_constants::BL32_MEM_BASE {
        size = qemu_constants::BL32_MEM_SIZE - (base - qemu_constants::BL32_MEM_BASE);
    } else {
        base = qemu_constants::BL32_MEM_BASE;
    };
    FSP_ALLOC.init(base, size);

    unsafe{
        FSP_SLAB.main_init();    
    }
}

/// This is the actual main function that extern_c_defs::fsp_main_wrapper() calls.
pub fn fsp_main() {
    fsp_init();

    debug!("fsp main");
   
    mem_test();

    unsafe{
        slab_test();
    }

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

 unsafe fn  slab_test (){
     //test for speed
    use alloc::vec::Vec;
    use alloc::alloc::{Layout};
    let mut a: Vec<usize> = Vec::new();
    for _ii in 1..100{
        for _i in 1..100{
            let ptr =  FSP_SLAB.kmem_alloc(Layout::from_size_align_unchecked(1, 2));
            *(ptr as *mut usize) = _i + (_ii*100) as usize;
            a.push(ptr as usize);
            debug!("Slab alloc addr = ,{}",ptr as usize);
        }
        for _i in 1..100{
            let ptr = a.pop();
            match ptr {
                Some(a) => {
                    debug!("Slab dealloc value = {} , addr = {}", *(a as *mut usize) , a);
                    FSP_SLAB.kmem_dealloc(a as *mut u8,Layout::from_size_align_unchecked(1, 2));
                }
                _=>{}
            }
        }
    }
}

/// This function is called on panic.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    debug!(info.payload().downcast_ref::<&str>().unwrap());
    loop {}
}

#[alloc_error_handler]
fn alloc_error_handler(_layout: alloc::alloc::Layout) -> ! {
    panic!("alloc error")
}
