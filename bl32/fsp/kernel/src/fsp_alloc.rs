//! This is a dynamic memory allocator.

extern crate alloc; // need this due to #![no_std]---for regular Rust, it is by default.

use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

// Copied values from include/qemu_defs.h
// BL32_END is the end address of the BL32 image
// FSP_SEC_MEM_BASE is the base address for the secure DRAM
// FSP_SEC_MEM_SIZE is the size of the secure DRAM
//extern "C" {
//    pub fn get_bl32_end() -> u32;
//}

///! Buffer allocation size quantum: all buffers allocated are a multiple of this size.  This
///! MUST be a power of two.
const SIZE_QUANT: usize = 4;

///! End sentinel: value placed in bsize field of dummy block delimiting
///! end of pool block. The most negative number which will fit in a
///! bufsize, defined in a way that the compiler will accept.
const ESENT: isize = -(((1 << (core::mem::size_of::<isize>() * 8 - 2)) - 1) * 2) - 2;

//struct bhead {
//    bufsize prevfree;		      /* Relative link back to previous
//					 free buffer in memory or 0 if
//					 previous buffer is allocated.	*/
//    bufsize bsize;		      /* Buffer size: positive if free,
//					 negative if allocated. */
//};
struct BHead {
    prevfree: usize,
    bsize: usize, // TODO: perhaps this should be isize?
}

impl BHead {
    const fn new() -> BHead {
        BHead {
            prevfree: 0,
            bsize: 0,
        }
    }
}

//struct qlinks {
//    struct bfhead *flink;	      /* Forward link */
//    struct bfhead *blink;	      /* Backward link */
//};
struct QLinks {
    flink: Option<&'static mut BFHead>,
    blink: Option<&'static mut BFHead>,
}

impl QLinks {
    const fn new() -> QLinks {
        QLinks {
            flink: None,
            blink: None,
        }
    }
}

//struct bfhead {
//    struct bhead bh;		      /* Common allocated/free header */
//    struct qlinks ql;		      /* Links on free list */
//};
struct BFHead {
    bh: BHead,
    ql: QLinks,
}

impl BFHead {
    const fn new() -> BFHead {
        BFHead {
            bh: BHead::new(),
            ql: QLinks::new(),
        }
    }
}

//static struct bfhead freelist = {     /* List of free buffers */
//    {0, 0},
//    {&freelist, &freelist}
//};
pub struct FspAlloc {
    freelist: BFHead,
}

impl FspAlloc {
    // const fn is necessary because we're creating a static variable for FspAlloc. Note that const
    // fn is interpreted at compile time, so everything in it should be interpretable statically.
    pub const fn new() -> FspAlloc {
        FspAlloc {
            freelist: BFHead::new(),
        }
    }

    // This is bpool(), but renamed to init(). Unlike the original bpool(), we're assuming that
    // this is only called once in the beginning.
    pub fn init(&self, buf: *mut u8, len: usize) -> () {
        let len = len & !(SIZE_QUANT - 1);

        // Need a check like the following:
        // assert(len - sizeof(struct bhead) <= -((bufsize) ESent + 1));

        // Need checks like the following:
        // assert(freelist.ql.blink->ql.flink == &freelist);
        // assert(freelist.ql.flink->ql.blink == &freelist);

        // dereferencing raw pointers is unsafe.
        unsafe {
            let b: &mut BFHead = &mut *(buf as *mut BFHead);

            /* Clear the backpointer at the start of the block to indicate that
            there is no free block prior to this one. That blocks
            recombination when the first block in memory is released. */
            b.bh.prevfree = 0;

            /* Chain the new block to the free list. */
            //b->ql.flink = &freelist;
            //b->ql.blink = freelist.ql.blink;
            //freelist.ql.blink = b;
            //b->ql.blink->ql.flink = b;
            let freelist_ptr = &self.freelist as *const BFHead as usize as *mut BFHead;
            //b.ql.flink = Some(&mut *freelist_ptr);
            b.ql.flink = None; // just for testing

            //b.ql.blink = (*freelist_ptr).ql.blink.take();
            b.ql.blink = Some(&mut *freelist_ptr); // this is different from the original

            //(*freelist_ptr).ql.blink = Some(&mut *(buf as *mut BFHead));
            (*freelist_ptr).ql.blink = None; // just for testing
            (*freelist_ptr).ql.flink = Some(&mut *(buf as *mut BFHead)); // this is different from the original

            /* Create a dummy allocated buffer at the end of the pool. This dummy
            buffer is seen when a buffer at the end of the pool is released and
            blocks recombination of the last buffer with the dummy buffer at
            the end. The length in the dummy buffer is set to the largest
            negative number to denote the end of the pool for diagnostic
            routines (this specific value is not counted on by the actual
            allocation and release functions). */

            let len = len - core::mem::size_of::<BHead>();
            b.bh.bsize = len;
            let bn: &mut BHead = &mut *((buf as usize + len) as *mut BHead);
            bn.prevfree = len;

            bn.bsize = ESENT as usize;
        }
    }

    fn ql_size(&self) -> usize {
        core::mem::size_of::<QLinks>()
    }

    fn size_q(&self) -> usize {
        let qls = self.ql_size();
        if SIZE_QUANT > qls {
            SIZE_QUANT
        } else {
            qls
        }
    }
}

unsafe impl GlobalAlloc for FspAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        debug!("alloc");

        let mut size: usize = layout.size();
        if size < self.size_q() {
            size = self.size_q();
        }
        size = (size + (SIZE_QUANT - 1)) & (!(SIZE_QUANT - 1));
        size = size + core::mem::size_of::<BHead>();

        let mut current_ref = &mut *(&self.freelist as *const BFHead as usize as *mut BFHead);

        while let Some(ref mut b) = current_ref.ql.flink {
            debug!("while");
            if b.bh.bsize >= size {
                debug!("first if");
                // Buffer  is big enough to satisfy  the request.  Allocate it
                // to the caller.  We must decide whether the buffer is  large
                // enough  to  split  into  the part given to the caller and a
                // free buffer that remains on the free list, or  whether  the
                // entire  buffer  should  be  removed	from the free list and
                // given to the caller in its entirety.   We  only  split  the
                // buffer if enough room remains for a header plus the minimum
                // quantum of allocation.
                if (b.bh.bsize - size) > (self.size_q() + core::mem::size_of::<BHead>()) {
                    debug!("second if");
                    let ba: &mut BHead = &mut *(((*b as *const BFHead as usize)
                        + (b.bh.bsize - size))
                        as *mut BHead);
                    let bn: &mut BHead = &mut *(((ba as *mut BHead as usize) + size) as *mut BHead);
                    // Subtract size from length of free block.
                    b.bh.bsize = b.bh.bsize - size;
                    // Link allocated buffer to the previous free buffer.
                    ba.prevfree = b.bh.bsize;
                    // Plug negative size into user buffer.
                    // TODO
                    //ba.bsize = -(size);
                    // Mark buffer after this one not preceded by free block.
                    bn.prevfree = 0;
                    //buf = (void *) ((((char *) ba) + sizeof(struct bhead)));
                    return ((ba as *mut BHead as usize) + core::mem::size_of::<BHead>())
                        as *mut u8;
                } else {
                }
            }
            // TODO: handle None
            current_ref = current_ref.ql.flink.as_mut().unwrap();
            debug!("while end");
        }
        debug!("while done");

        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        debug!("dealloc")
    }
}
