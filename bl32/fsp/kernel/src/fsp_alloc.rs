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
//const ESENT: isize = (-(((1 << (core::mem::size_of::<isize>() * 8 - 2)) - 1) * 2) - 2);
// TODO: Not sure if this is okay
const ESENT: usize = usize::max_value();

//struct bhead {
//    bufsize prevfree;               /* Relative link back to previous
//                                       free buffer in memory or 0 if
//                                       previous buffer is allocated.  */
//    bufsize bsize;                  /* Buffer size: positive if free,
//                                       negative if allocated. */
//};
struct BHead {
    prevfree: usize,
    bsize: usize,
    allocated: bool,
}

impl BHead {
    const fn new() -> BHead {
        BHead {
            prevfree: 0,
            bsize: 0,
            allocated: false,
        }
    }

    fn init(&self, prevfree: usize, bsize: usize, allocated: bool) {
        self.set_prevfree(prevfree);
        self.set_bsize(bsize);
        self.set_allocated(allocated);
    }

    fn addr(&self) -> usize {
        self as *const BHead as usize
    }

    fn is_allocated(&self) -> bool {
        self.allocated
    }

    fn set_allocated(&self, allocated: bool) {
        self.as_mut_ref().allocated = allocated;
    }

    // TODO: should this be 'static?
    fn from_addr(addr: usize) -> &'static mut BHead {
        unsafe { &mut *(addr as *mut BHead) }
    }

    fn as_mut_ref(&self) -> &mut BHead {
        unsafe { &mut *(self as *const BHead as usize as *mut BHead) }
    }

    fn prevfree(&self) -> usize {
        self.prevfree
    }

    fn set_prevfree(&self, prevfree: usize) {
        self.as_mut_ref().prevfree = prevfree;
    }

    fn bsize(&self) -> usize {
        self.bsize
    }

    fn set_bsize(&self, bsize: usize) {
        self.as_mut_ref().bsize = bsize;
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
        self.bh.init(prevfree, bsize, false);
        self.set_flink(flink);
        self.set_blink(blink);
    }

    //// TODO: should this be 'static?
    //fn new_ref_from_addr(addr: usize) -> &'static BFHead {
    //    unsafe { &*(addr as *const BFHead) }
    //}

    // TODO: should this be 'static?
    fn from_addr(addr: usize) -> &'static mut BFHead {
        unsafe { &mut *(addr as *mut BFHead) }
    }

    fn addr(&self) -> usize {
        self as *const BFHead as usize
    }

    fn as_mut_ref(&self) -> &mut BFHead {
        unsafe { &mut *(self as *const BFHead as usize as *mut BFHead) }
    }

    fn blink_ref(&self) -> &BFHead {
        unsafe { &*(self.ql.blink as *const BFHead) }
    }

    fn blink_mut_ref(&self) -> &mut BFHead {
        unsafe { &mut *(self.ql.blink as *mut BFHead) }
    }

    fn flink_ref(&self) -> &BFHead {
        unsafe { &*(self.ql.flink as *const BFHead) }
    }

    fn flink_mut_ref(&self) -> &mut BFHead {
        unsafe { &mut *(self.ql.flink as *mut BFHead) }
    }

    fn set_blink(&self, blink: &BFHead) {
        self.as_mut_ref().ql.blink = blink.addr();
    }

    fn set_flink(&self, flink: &BFHead) {
        self.as_mut_ref().ql.flink = flink.addr();
    }

    fn prevfree(&self) -> usize {
        self.bh.prevfree()
    }

    fn set_prevfree(&self, prevfree: usize) {
        self.bh.set_prevfree(prevfree);
    }

    fn bsize(&self) -> usize {
        self.bh.bsize()
    }

    fn set_bsize(&self, bsize: usize) {
        self.bh.set_bsize(bsize);
    }

    fn set_allocated(&self, allocated: bool) {
        self.bh.set_allocated(allocated);
    }

    fn is_allocated(&self) -> bool {
        self.bh.is_allocated()
    }

    fn eq(&self, target: &BFHead) -> bool {
        self.addr() == target.addr()
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

        let b: &mut BFHead = BFHead::from_addr(buf);

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
        b.set_blink(self.freelist.blink_ref());
        self.freelist.set_blink(b);
        b.blink_mut_ref().set_flink(b);

        /* Create a dummy allocated buffer at the end of the pool. This dummy
        buffer is seen when a buffer at the end of the pool is released and
        blocks recombination of the last buffer with the dummy buffer at
        the end. The length in the dummy buffer is set to the largest
        negative number to denote the end of the pool for diagnostic
        routines (this specific value is not counted on by the actual
        allocation and release functions). */

        let len = len - core::mem::size_of::<BHead>();
        b.set_bsize(len);
        b.set_allocated(false);

        let bn: &mut BHead = BHead::from_addr(buf + len);
        bn.set_prevfree(len);
        bn.set_bsize(ESENT);
        bn.set_allocated(true);
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

        // converting usize to u32 for layout.size().
        // TODO: check the range before converting.

        let mut size = layout.size();
        if size < self.size_q() {
            size = self.size_q();
        }
        size = (size + (SIZE_QUANT - 1)) & (!(SIZE_QUANT - 1));
        size = size + core::mem::size_of::<BHead>();

        let mut b = self.freelist.flink_mut_ref();

        while !b.eq(&self.freelist) {
            let bsize = b.bsize();
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
                    let ba: &mut BHead = BHead::from_addr(b.addr() + bsize - size);
                    let bn: &mut BHead = BHead::from_addr(ba.addr() + size);
                    /* Subtract size from length of free block. */
                    let bsize = bsize - size;
                    b.set_bsize(bsize);
                    /* Link allocated buffer to the previous free buffer. */
                    ba.set_prevfree(bsize);
                    /* Plug negative size into user buffer. */
                    ba.set_bsize(size);
                    ba.set_allocated(true);
                    /* Mark buffer after this one not preceded by free block. */
                    bn.set_prevfree(0);

                    return (ba.addr() + core::mem::size_of::<BHead>()) as *mut u8;
                } else {
                    /* The buffer isn't big enough to split.  Give  the  whole
                    shebang to the caller and remove it from the free list. */
                    let ba: &mut BHead = BHead::from_addr(b.addr() + b.bsize());

                    b.blink_mut_ref().set_flink(b.flink_ref());
                    b.flink_mut_ref().set_blink(b.blink_ref());
                    /* Negate size to mark buffer allocated. */
                    b.set_bsize(b.bsize());
                    b.set_allocated(true);
                    /* Zero the back pointer in the next buffer in memory
                    to indicate that this buffer is allocated. */
                    ba.set_prevfree(0);
                    return (b.addr() + core::mem::size_of::<BHead>()) as *mut u8;
                }
            }

            b = b.flink_mut_ref(); /* Link to next buffer */
        }

        // BECtl not implemented

        null_mut()
    }

    unsafe fn dealloc(&self, buf: *mut u8, _layout: Layout) {
        debug!("dealloc");

        let mut b: &mut BFHead = BFHead::from_addr((buf as usize) - core::mem::size_of::<BHead>());

        // TODO: need to do something for the following?
        /* Buffer size must be negative, indicating that the buffer is
        allocated. */

        /*

        /* Back pointer in next buffer must be zero, indicating the
        same thing: */

        assert(BH((char *) b - b->bh.bsize)->prevfree == 0);

        */

        /* If the back link is nonzero, the previous buffer is free.  */
        if b.prevfree() != 0 {
            /* The previous buffer is free. Consolidate this buffer with it
            by adding the length of this buffer to the previous free
            buffer. Note that we subtract the size in the buffer being
            released, since it's negative to indicate that the buffer is
            allocated. */
            let size = b.bsize();
            b = BFHead::from_addr(b.addr() - b.prevfree());
            b.set_bsize(b.bsize() + size);
            b.set_allocated(false);
        } else {
            /* The previous buffer isn't allocated. Insert this buffer
            on the free list as an isolated free block. */
            b.set_flink(&self.freelist);
            b.set_blink(&self.freelist.blink_ref());
            self.freelist.set_blink(b);
            b.blink_mut_ref().set_flink(b);
            b.set_bsize(b.bsize());
            b.set_allocated(false);
        }

        /* Now we look at the next buffer in memory, located by advancing from
        the  start  of  this  buffer  by its size, to see if that buffer is
        free.  If it is, we combine  this  buffer  with      the  next  one  in
        memory, dechaining the second buffer from the free list. */
        let mut bn: &mut BFHead = BFHead::from_addr(b.addr() + b.bsize());

        if !bn.is_allocated() {
            /* The buffer is free.  Remove it from the free list and add
            its size to that of our buffer. */
            bn.blink_mut_ref().set_flink(bn.flink_ref());
            bn.flink_mut_ref().set_blink(bn.blink_ref());
            b.set_bsize(b.bsize() + bn.bsize());

            /* Finally,  advance  to   the  buffer  that   follows  the  newly
            consolidated free block.  We must set its  backpointer  to  the
            head  of  the  consolidated free block.  We know the next block
            must be an allocated block because the process of recombination
            guarantees  that  two  free  blocks will never be contiguous in
            memory.  */

            bn = BFHead::from_addr(b.addr() + b.bsize());
        }

        /* The next buffer is allocated.  Set the backpointer in it  to  point
        to this buffer; the previous free buffer in memory. */
        bn.set_prevfree(b.bsize());
        debug!("dealloc done");
    }
}
