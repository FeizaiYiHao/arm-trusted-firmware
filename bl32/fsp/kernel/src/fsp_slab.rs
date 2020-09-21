//! This is a dynamic memory allocator.
//!
extern crate alloc;

use crate::debug;
use alloc::alloc::{Layout,alloc,dealloc};

const SIZE_QUANT_OBJ: usize = core::mem::size_of::<usize>(); //each obj has to be bigger than a pointer 
const PAGE_SIZE: usize = 4096; //allocate 4kb of memory from fsp_alloc each time (the size of a page)


pub fn get_power_of_two(size:usize)->usize{
    size.wrapping_next_power_of_two()
}

pub struct KmemCache{
    slabs_full:usize,			// list of full slabs
    slabs_partial:usize,		// list of partial slabs 
    slabs_free:usize,			// list of free slabs
    name:[char;8],              // name of the cache, length is fixed for better memory management
    next:usize,                 // next cache
    obj_size:usize,             // size of objects
    obj_num:usize,              // number of objects in one slab
    obj_total:usize,            // total capacity of this cache (can be growing or shrinking)
    obj_active:usize,           // number of active(occupied) object
    obj_free:usize,             // number of free(usable) object
}

struct Slab{
    next_slab: usize,     
    prev_slab: usize,
    free_head:usize,
    inuse:usize,
    free:usize,
    capacity:usize,
    obj_size: usize,
    start_addr: usize,
    end_addr: usize,
}

struct Obj{
    next_obj:usize,
}

impl Obj{
    fn from_addr( addr: usize) -> &'static mut Obj {
        unsafe { &mut *(addr as *mut Obj) }
    }
}

impl KmemCache{
    pub const fn new() -> KmemCache {
        KmemCache {
        slabs_full : 0 ,			
        slabs_partial :0,		
        slabs_free:0,			
        name:['0','0','0','0','0','0','0','0'],              
        next:0,                 
        obj_size:0,             
        obj_num:0,              
        obj_total:0,            
        obj_active:0,           
        obj_free:0,             
        }
    }

    pub fn init(&self, _obj_size:usize , _addr : usize){
        self.as_mut_ref().slabs_full = 0 ;			
        self.as_mut_ref().slabs_partial = 0;		
        self.as_mut_ref().slabs_free = 0;			
        self.as_mut_ref().name = ['0','0','0','0','0','0','0','0'];           
        self.as_mut_ref().next = 0;             
        self.as_mut_ref().obj_size = _obj_size;             
        self.as_mut_ref().obj_num = 0;            
        self.as_mut_ref().obj_total = 0;           
        self.as_mut_ref().obj_active = 0; 
        self.as_mut_ref().obj_free = 0;     
    }


    /*pub fn kmem_check_name(&self, _name:&String)->bool{
        true
    }*/

    fn as_mut_ref(&self) -> &mut KmemCache {
        unsafe { &mut *(self as *const KmemCache as *mut KmemCache) }
    }

    fn from_addr(addr: usize) -> &'static mut KmemCache {
        unsafe { &mut *(addr as *mut KmemCache) }
    }

    pub unsafe fn main_init(&self){

        self.as_mut_ref().name = ['t','h','e','m','a','i','n','K'];
        self.as_mut_ref().obj_size = get_power_of_two(core::mem::size_of::<KmemCache>());
        
        debug!("main_init objsize{}",self.obj_size);

        // at the begining of the program, allocate one page to hold future KmemCaches
        self.kmem_grow();
        debug!("main_init addr{}",self.slabs_free);
    }

    pub fn main_search_kmem(&self,size:usize) -> Option<usize>{
        
        let mut ptr : usize = self.next;

        while ptr!=0{
            let _kmem_cache: &mut KmemCache = KmemCache::from_addr(ptr as usize);
            if _kmem_cache.obj_size == size{
                return Some(ptr)
            }
            ptr = _kmem_cache.next;
        }

        None
    }

    pub unsafe fn kmem_search_slab(&self)-> Option<*mut u8>{
        if self.obj_free <= 0{
            return None
        }

        let ptr : usize = self.slabs_partial;
        //try partial first
        if ptr!=0 {
            let slab: &mut Slab = Slab::from_addr(ptr as usize);
            if slab.get_free() -1 <= 0 {
                self.kmem_put_partial_into_full();
            }
            return Some(slab.slab_alloc())
        }

        let ptr : usize = self.slabs_free;
        //get one from free list 
        if ptr!=0 {
            let slab: &mut Slab = Slab::from_addr(ptr as usize);


            self.kmem_put_free_into_partial();

            return Some(slab.slab_alloc())
        }
        None
    }
    
    pub unsafe fn kmem_alloc(&self, layout: Layout)-> *mut u8{
        let mut size: usize = layout.size();
        size  = (size + (SIZE_QUANT_OBJ - 1)) & (!(SIZE_QUANT_OBJ - 1));
        size = get_power_of_two(size);
        match self.main_search_kmem(size){
            //find a cache
            Some(addr)=>{
                let cache =  KmemCache::from_addr(addr);
                match cache.kmem_search_slab(){
                    Some(ret)=>{
                        cache.as_mut_ref().obj_free -= 1;
                        ret
                    }
                    _=>{
                        cache.kmem_grow();
                        self.kmem_alloc(layout)
                    }
               }

            }
            _=>{
                self.create_kmem_cache(size);
                self.kmem_alloc(layout)
            }
        }
    }

    pub unsafe fn kmem_dealloc(&self, buf: *mut u8, _layout: Layout) {
        let mut size : usize = _layout.size();
        size  = (size + (SIZE_QUANT_OBJ - 1)) & (!(SIZE_QUANT_OBJ - 1));
        size = get_power_of_two(size);
        match self.main_search_kmem(size){
            Some(_kmem_addr)=>{
                let _kmem_addr = _kmem_addr as usize;
                let (slab_addr,code) = KmemCache::from_addr(_kmem_addr).find_slab(buf as usize);
                Slab::from_addr(slab_addr).slab_dealloc(buf as usize);
                KmemCache::from_addr(_kmem_addr).obj_free += 1;
                if code == 1{
                    KmemCache::from_addr(_kmem_addr).kmem_put_full_into_partial(slab_addr);
                }
                else if code == 2 && Slab::from_addr(slab_addr).inuse == 0 {
                    KmemCache::from_addr(_kmem_addr).kmem_put_partial_into_free(slab_addr);
                }
            }
            _=>{}
        }
    }

    pub fn find_slab(&self, addr:usize)->(usize,usize){
        let mut ptr = self.slabs_full;
        
        while ptr!=0{
            if Slab::from_addr(ptr).check_obj(addr){
                return (ptr,1)
            }
            ptr = Slab::from_addr(ptr).next_slab;
        }

        ptr = self.slabs_partial;

        while ptr!=0{
            if Slab::from_addr(ptr).check_obj(addr){
                return (ptr,2)
            }
            ptr = Slab::from_addr(ptr).next_slab;
        }

        (0,0)
    }

    pub unsafe fn create_kmem_cache(&self, _obj_size:usize) {

        match self.kmem_search_slab(){
            Some(addr)=>{
                let cache = KmemCache::from_addr(addr as usize);
                cache.init(_obj_size,addr as usize);
                cache.next = self.next;
                self.as_mut_ref().next = addr as usize;
                cache.kmem_grow();
            }
            None=>{
                self.kmem_grow();
                self.create_kmem_cache(_obj_size)
            }
        }

    }

    pub unsafe fn kmem_grow(&self){
        let _layout = Layout::from_size_align(PAGE_SIZE, 2);
        match _layout{
            Ok(layout)=>{
                let ptr : *mut u8 = alloc(layout);

                let slab: &mut Slab = Slab::from_addr(ptr as usize);
                slab.init(self.obj_size, ptr.clone() as usize, self.slabs_free);

                if self.slabs_free!=0{
                    Slab::from_addr(self.slabs_free).prev_slab = ptr as usize;
                }

                self.as_mut_ref().slabs_free = ptr as usize;

                self.as_mut_ref().obj_free += slab.get_free();
                self.as_mut_ref().obj_total += slab.get_capacity();
            }
            Err(_)=>{debug!("Slab failed to construct layout");}
        }

    }

    pub unsafe fn kmem_shrink(&self){
        if self.slabs_free!=0{
            let slab = Slab::from_addr(self.slabs_free);
            self.as_mut_ref().slabs_free = slab.next_slab;
            if self.slabs_free != 0{
                Slab::from_addr(self.slabs_free).prev_slab = 0;
            }
            let _layout = Layout::from_size_align(PAGE_SIZE, 2);
            match _layout{
                Ok(layout)=>{
                    self.as_mut_ref().obj_total -= slab.capacity;
                    self.as_mut_ref().obj_free -= slab.free; 
                    dealloc(slab.start_addr as *mut u8,layout);
                }
                _=>{debug!("Slab failed to construct layout");}
            }
        }
    }
    
    // this will always be the first slab in partial list
    pub fn kmem_put_partial_into_full(&self){
        let slab: &mut Slab = Slab::from_addr(self.slabs_partial);
        let tmp = self.slabs_partial;


        if self.slabs_full != 0{
            Slab::from_addr(self.slabs_full).prev_slab = tmp.clone();
        }
         if slab.next_slab!=0 {
            Slab::from_addr(slab.next_slab).prev_slab = 0;
        }
        self.as_mut_ref().slabs_partial = slab.next_slab;
        slab.as_mut_ref().next_slab = self.slabs_full;
        self.as_mut_ref().slabs_full = tmp;
        slab.as_mut_ref().prev_slab = 0;
    }

   // this will always be the first slab in free list
    pub fn kmem_put_free_into_partial(&self){
        let slab: &mut Slab = Slab::from_addr(self.slabs_free);
        let tmp = self.slabs_free;

        if self.slabs_partial != 0{
            Slab::from_addr(self.slabs_partial).prev_slab = tmp.clone();
        }
        if slab.next_slab!=0 {
            Slab::from_addr(slab.next_slab).prev_slab = 0;
        }
        self.as_mut_ref().slabs_free = slab.next_slab;
        slab.as_mut_ref().next_slab = self.slabs_partial;
        self.as_mut_ref().slabs_partial = tmp;
        slab.as_mut_ref().prev_slab = 0;
    }

    pub unsafe fn kmem_put_partial_into_free(&self,addr:usize){
        let slab = Slab::from_addr(addr);

        if self.slabs_free != 0{
            self.kmem_shrink();
        }

        if slab.prev_slab!=0{
            Slab::from_addr(slab.prev_slab).next_slab = slab.next_slab;
        }
        if self.slabs_partial == addr{
            self.as_mut_ref().slabs_partial = slab.next_slab;
            if slab.next_slab != 0 {
                Slab::from_addr(slab.next_slab).prev_slab = 0;
            }
        }

        slab.next_slab = self.slabs_free;
        if self.slabs_free != 0{
            Slab::from_addr(self.slabs_free).prev_slab = addr.clone();
        }
        self.as_mut_ref().slabs_free = addr;

    }
    pub fn kmem_put_full_into_partial(&self,addr:usize){
        let slab = Slab::from_addr(addr);

        if slab.prev_slab!=0{
            Slab::from_addr(slab.prev_slab).next_slab = slab.next_slab;
        }
        if self.slabs_full == addr{
            self.as_mut_ref().slabs_full = slab.next_slab;
            if slab.next_slab != 0 {
                Slab::from_addr(slab.next_slab).prev_slab = 0;
            }
        }

        slab.next_slab = self.slabs_partial;
        if self.slabs_partial != 0{
            Slab::from_addr(self.slabs_partial).prev_slab = addr.clone();
        }
        self.as_mut_ref().slabs_partial = addr;
    }
}

impl Slab{
    pub fn from_addr(addr: usize) -> &'static mut Slab {
        unsafe { &mut *(addr as *mut Slab) }
    }

    fn as_mut_ref(&self) -> &mut Slab {
        unsafe { &mut *(self as *const Slab as *mut Slab) }
    }

    /*fn set_next(&self, next: usize) {
        self.as_mut_ref().next_slab = next;
    }*/

    fn get_free(&self)-> usize{
        self.free
    }

    fn get_capacity(&self)-> usize{
        self.capacity
    }

    pub unsafe fn init(&self, _obj_size:usize, addr:usize, _next_slab:usize){

        debug!("Slab init addr{} next{}", addr, _next_slab);

        self.as_mut_ref().next_slab = _next_slab;
        self.as_mut_ref().prev_slab = 0;

        let _obj_size = get_power_of_two((_obj_size + (SIZE_QUANT_OBJ - 1)) & (!(SIZE_QUANT_OBJ - 1)));
        self.as_mut_ref().obj_size = _obj_size;
        let _obj_num = PAGE_SIZE/_obj_size;
        let self_space = get_power_of_two((core::mem::size_of::<Slab>() + (SIZE_QUANT_OBJ - 1)) & (!(SIZE_QUANT_OBJ - 1)));
        self.as_mut_ref().capacity = _obj_num - self_space/_obj_size;
        self.as_mut_ref().free = self.capacity;

        self.as_mut_ref().free_head = addr + self_space;

        let mut ptr:usize = self.free_head.clone();
        for _i in 0..self.capacity{
            Obj::from_addr(ptr).next_obj = ptr + _obj_size;
            ptr = ptr + _obj_size;
        }

        self.as_mut_ref().start_addr = addr.clone();   
        self.as_mut_ref().end_addr = addr.clone()+PAGE_SIZE;
        
    }

    fn check_obj(&self, addr: usize)->bool{
        self.start_addr <= addr && self.end_addr >= addr
    }

    pub unsafe fn slab_alloc(&self) -> *mut u8{
        let ret: *mut u8 = self.free_head as *mut u8;
        self.as_mut_ref().free_head = Obj::from_addr(self.free_head).next_obj;
        self.as_mut_ref().inuse += 1;
        self.as_mut_ref().free -= 1;
        ret
    }

    pub unsafe fn slab_dealloc(&self,addr:usize) {
        let tmp = self.free_head;
        self.as_mut_ref().free_head = addr;
        Obj::from_addr(self.free_head).next_obj = tmp;
        self.as_mut_ref().inuse -= 1;
        self.as_mut_ref().free += 1;
    }
}