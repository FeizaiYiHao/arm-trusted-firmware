//! This is a dynamic memory allocator.

extern crate alloc; // need this due to #![no_std]---for regular Rust, it is by default.

use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

///! Buffer allocation size quantum: all buffers allocated are a multiple of this size.  This
///! MUST be a power of two.
const SIZE_QUANT: usize = 4;

///! End sentinel: value placed in bsize field of dummy block delimiting
///! end of pool block. The most negative number which will fit in a
///! bufsize, defined in a way that the compiler will accept.
const ESENT: isize = -(((1 << (core::mem::size_of::<isize>() * 8 - 2)) - 1) * 2) - 2;

//struct bhead {
//    bufsize prevfree;               /* Relative link back to previous
//                                       free buffer in memory or 0 if
//                                       previous buffer is allocated.  */
//    bufsize bsize;                  /* Buffer size: positive if free,
//                                       negative if allocated. */
//};
struct BHead {
    prevfree: usize,
    bsize: Option<usize>, // TODO: perhaps this should be isize?
}

impl BHead {
    const fn new() -> BHead {
        BHead {
            prevfree: 0,
            bsize: Some(0),
        }
    }

    fn init(&self, prevfree: usize, bsize: usize) {
        self.set_prevfree(prevfree);
        self.set_bsize(bsize);
    }

    // TODO: should this be 'static?
    //fn new_ref_from_addr(addr: usize) -> &'static BHead {
    //    unsafe { &*(addr as *const BHead) }
    //}

    fn get_addr(&self) -> usize {
        self as *const BHead as usize
    }

    fn new_mut_ref_from_addr(addr: usize) -> &'static mut BHead {
        unsafe { &mut *(addr as *mut BHead) }
    }

    fn as_mut_ref(&self) -> &mut BHead {
        unsafe { &mut *(self as *const BHead as usize as *mut BHead) }
    }

    //fn as_ref(&self) -> &BHead {
    //    unsafe { &*(self as *const BHead) }
    //}

    //fn get_mut_ptr(&self) -> *mut BHead {
    //    self as *const BHead as usize as *mut BHead
    //}

    //fn get_ptr(&self) -> *const BHead {
    //    self as *const BHead as usize as *const BHead
    //}

    fn set_prevfree(&self, prevfree: usize) {
        self.as_mut_ref().prevfree = prevfree;
    }

    fn get_bsize(&self) -> Option<usize> {
        self.bsize
    }

    fn set_bsize(&self, bsize: usize) {
        self.as_mut_ref().bsize = Some(bsize);
    }

    fn set_bsize_none(&self) {
        self.as_mut_ref().bsize = None;
    }
}

//struct qlinks {
//    struct bfhead *flink;           /* Forward link */
//    struct bfhead *blink;           /* Backward link */
//};
struct QLinks {
    flink: usize,
    blink: usize,
}

//struct bfhead {
//    struct bhead bh;                /* Common allocated/free header */
//    struct qlinks ql;               /* Links on free list */
//};
struct BFHead {
    bh: BHead,
    ql: QLinks,
}

impl BFHead {
    // See the comment for FspAlloc::new(). This is another const. init() must be called before
    // using an instance.
    const fn new() -> BFHead {
        BFHead {
            bh: BHead::new(),
            ql: QLinks { flink: 0, blink: 0 },
        }
    }

    // This must be called.
    fn init(&self, prevfree: usize, bsize: usize, flink: &BFHead, blink: &BFHead) -> () {
        self.bh.init(prevfree, bsize);
        self.set_flink(flink);
        self.set_blink(blink);
    }

    //// TODO: should this be 'static?
    //fn new_ref_from_addr(addr: usize) -> &'static BFHead {
    //    unsafe { &*(addr as *const BFHead) }
    //}

    // TODO: should this be 'static?
    fn new_mut_ref_from_addr(addr: usize) -> &'static mut BFHead {
        unsafe { &mut *(addr as *mut BFHead) }
    }

    fn get_addr(&self) -> usize {
        self as *const BFHead as usize
    }

    //fn as_ref(&self) -> &BFHead {
    //    unsafe { &*(self as *const BFHead) }
    //}

    fn as_mut_ref(&self) -> &mut BFHead {
        unsafe { &mut *(self as *const BFHead as usize as *mut BFHead) }
    }

    //fn get_ptr(&self) -> *const BFHead {
    //    self as *const BFHead
    //}

    //fn get_mut_ptr(&self) -> *mut BFHead {
    //    self as *const BFHead as usize as *mut BFHead
    //}

    fn get_blink_ref(&self) -> &BFHead {
        unsafe { &*(self.ql.blink as *const BFHead) }
    }

    fn get_blink_mut_ref(&self) -> &mut BFHead {
        unsafe { &mut *(self.ql.blink as *mut BFHead) }
    }

    fn get_flink_ref(&self) -> &BFHead {
        unsafe { &*(self.ql.flink as *const BFHead) }
    }

    fn get_flink_mut_ref(&self) -> &mut BFHead {
        unsafe { &mut *(self.ql.flink as *mut BFHead) }
    }

    fn set_blink(&self, blink: &BFHead) {
        self.as_mut_ref().ql.blink = blink.get_addr();
    }

    fn set_flink(&self, flink: &BFHead) {
        self.as_mut_ref().ql.flink = flink.get_addr();
    }

    fn set_prevfree(&self, prevfree: usize) {
        self.bh.set_prevfree(prevfree);
    }

    fn get_bsize(&self) -> Option<usize> {
        self.bh.get_bsize()
    }

    fn set_bsize(&self, bsize: usize) {
        self.bh.set_bsize(bsize);
    }

    fn set_bsize_none(&self) {
        self.bh.set_bsize_none();
    }

    fn eq(&self, target: &BFHead) -> bool {
        self.get_addr() == target.get_addr()
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
    //
    // Because this is const, it just creates a place holder. init() must be called before using
    // the instance.
    pub const fn new() -> FspAlloc {
        FspAlloc {
            freelist: BFHead::new(),
        }
    }

    // This is bpool(), but renamed to init(). Unlike the original bpool(), we're assuming that
    // this is only called once in the beginning. This must be called.
    pub fn init(&self, buf: usize, len: usize) {
        self.freelist.init(0, 0, &self.freelist, &self.freelist);
        let len = len & !(SIZE_QUANT - 1);

        // Need a check like the following:
        // assert(len - sizeof(struct bhead) <= -((bufsize) ESent + 1));

        // Need checks like the following:
        // assert(freelist.ql.blink->ql.flink == &freelist);
        // assert(freelist.ql.flink->ql.blink == &freelist);

        let b: &mut BFHead = BFHead::new_mut_ref_from_addr(buf);

        /* Clear the backpointer at the start of the block to indicate that
        there is no free block prior to this one. That blocks
        recombination when the first block in memory is released. */
        b.set_prevfree(0);

        /* Chain the new block to the free list. */
        //b->ql.flink = &freelist;
        //b->ql.blink = freelist.ql.blink;
        //freelist.ql.blink = b;
        //b->ql.blink->ql.flink = b;
        b.set_flink(&self.freelist);
        b.set_blink(self.freelist.get_blink_ref());
        self.freelist.set_blink(b);
        b.get_blink_mut_ref().set_flink(b);

        /* Create a dummy allocated buffer at the end of the pool. This dummy
        buffer is seen when a buffer at the end of the pool is released and
        blocks recombination of the last buffer with the dummy buffer at
        the end. The length in the dummy buffer is set to the largest
        negative number to denote the end of the pool for diagnostic
        routines (this specific value is not counted on by the actual
        allocation and release functions). */

        let len = len - core::mem::size_of::<BHead>();
        b.set_bsize(len);
        let bn: &mut BHead = BHead::new_mut_ref_from_addr(buf + len);
        debug!("calling");
        bn.set_prevfree(len);
        debug!("done");
        bn.set_bsize(ESENT as usize);
        debug!("FspAlloc init done");
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

        let mut b = self.freelist.get_flink_mut_ref();

        while !b.eq(&self.freelist) {
            debug!("while");
            if let Some(bsize) = b.get_bsize() {
                if bsize >= size {
                    /* Buffer  is big enough to satisfy  the request.  Allocate it
                    to the caller.  We must decide whether the buffer is  large
                    enough  to  split  into  the part given to the caller and a
                    free buffer that remains on the free list, or  whether  the
                    entire  buffer  should  be  removed from the free list and
                    given to the caller in its entirety.   We  only  split  the
                    buffer if enough room remains for a header plus the minimum
                    quantum of allocation. */
                    if (bsize - size) > (self.size_q() + core::mem::size_of::<BFHead>()) {
                        let ba: &mut BHead =
                            BHead::new_mut_ref_from_addr(b.get_addr() + bsize - size);
                        let bn: &mut BHead = BHead::new_mut_ref_from_addr(ba.get_addr() + size);
                        /* Subtract size from length of free block. */
                        let bsize = bsize - size;
                        b.set_bsize(bsize);
                        /* Link allocated buffer to the previous free buffer. */
                        ba.set_prevfree(bsize);
                        /* Plug negative size into user buffer. */
                        ba.set_bsize_none();
                        /* Mark buffer after this one not preceded by free block. */
                        bn.set_prevfree(0);

                        return (ba.get_addr() + core::mem::size_of::<BHead>()) as *mut u8;
                    } else {
                        /* The buffer isn't big enough to split.  Give  the  whole
                        shebang to the caller and remove it from the free list. */
                        let ba: &mut BHead =
                            BHead::new_mut_ref_from_addr(b.get_addr() + b.get_bsize().unwrap());

                        b.get_blink_mut_ref().set_flink(b.get_flink_ref());
                        b.get_flink_mut_ref().set_blink(b.get_blink_ref());
                        /* Negate size to mark buffer allocated. */
                        b.set_bsize_none();
                        /* Zero the back pointer in the next buffer in memory
                        to indicate that this buffer is allocated. */
                        ba.set_prevfree(0);
                        return (b.get_addr() + core::mem::size_of::<BHead>()) as *mut u8;
                    }
                }
            }

            b = b.get_flink_mut_ref(); /* Link to next buffer */
        }

        // BECtl not implemented

        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        debug!("dealloc")
    }
}
