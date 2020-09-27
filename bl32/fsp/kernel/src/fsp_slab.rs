//! This is a dynamic memory allocator.
//!
extern crate alloc;

use crate::debug;
use alloc::alloc::{Layout,alloc,dealloc};

//const SIZE_QUANT_OBJ: usize = core::mem::size_of::<usize>(); //each obj has to be bigger than a pointer 
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
    next_slab: usize,     // next slab in the linked-list, should set to 0 if it's at the tail
    prev_slab: usize,     // prev slab in the linked-list, should set to 0 if it's at the head
    free_head:usize,      // the head of the linked-list for free object, should set to 0 if there is not free object
    inuse:usize,          // number of objects that are in use 
    free:usize,           // number of objects that are free
    capacity:usize,       // total objects in this slab
    obj_size: usize,      // size of each object should be a power of two and bigger than the size of a pointer(usize)
    start_addr: usize,    // the start address of the slab, should be ths address of the slab instance 
    end_addr: usize,      // the end address of the slab
    cache_addr: usize     // the address for its owning kmem_cache
}

#[repr(C)]
struct Obj{
    slab_addr:usize,
    next_obj:usize,       // next free object
}

impl Obj{
    fn from_addr( addr: usize) -> &'static mut Obj {
        unsafe { &mut *(addr as *mut Obj) }
    }
}

impl KmemCache{
    //There are two types of KmemCache. FSP_SLAB is the main cache that holds all other KmemCaches
    //Other KmemCaches have a fixed size of each alloc/dealloc and sometimes a name
    //They allocate memory in two ways
    //1: User can create KmemCache with a name and size of a struct and this KmemCache would only serve alloc/dealloc for this struct
    //2: User can alloc/dealloc memory to the FSP_SLAB, and FSP_SLAB will handle the request based on requested memory size

    //This if for creating the static variable FSP_SLAB
    //wouldn't be used for any other creation for KmemCache
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

    //This must be called for each KmemCache creation.
    //TO DO: support name initialization 
    fn init(&self, _obj_size:usize , _addr : usize){
        //debug!("KmemCache init addr{} size{}", _addr, _obj_size);
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

    // this is for inintializing FSP_SLAB. this must be called
    pub unsafe fn main_init(&self){

        //as the FSP_SLAB holds all other KmemCaches, we should set its size to the size of KmemCache
        self.as_mut_ref().name = ['t','h','e','m','a','i','n','K'];
        self.as_mut_ref().obj_size = get_power_of_two(core::mem::size_of::<KmemCache>() + core::mem::size_of::<usize>());
        
        //debug!("main_init objsize{}",self.obj_size);

        // at the begining of the program, allocate one slab to hold future KmemCaches
        self.kmem_grow();
       // debug!("main_init addr{}",self.slabs_free);
    }

    //Search all caches in FSP_SLAB
    //Return the address of the KmemCache which size is equal to the input size
    fn main_search_kmem(&self,size:usize) -> Option<usize>{
        
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

    //Seach for a slab that has free object in a given cache and return a pointer to the allocated memory
    unsafe fn kmem_search_slab(&self)-> Option<*mut u8>{
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
    
    //Allcoate a piece of memory for a given Layout
    //If the cache with the size of the Layout is full, FSP_SLAB will call kmem_grow() to extend its capacity by one slab
    //If not cache has the same size, FSP_SLAB will call create_kmem_cache() to create one with the given size;
    //
    //This could only be called from FSP_SLAB, calling this function from any other KmemCache will probably casue infinity loop.
    pub unsafe fn kmem_alloc(&self, layout: Layout)-> *mut u8{
        let mut size: usize = layout.size();
        size = get_power_of_two(size + core::mem::size_of::<usize>());
        match self.main_search_kmem(size){
            //found a cache
            Some(addr)=>{
                let cache =  KmemCache::from_addr(addr);
                match cache.kmem_search_slab(){
                    //found a slab with free obj
                    Some(ret)=>{
                        cache.as_mut_ref().obj_free -= 1;
                        ret
                    }
                    //No aviable slab, allocate another slab to this cache
                    _=>{
                        cache.kmem_grow();
                        self.kmem_alloc(layout)
                    }
               }

            }
            //No such cache in FSP_SLAB, Create one
            _=>{
                self.create_kmem_cache(size);
                self.kmem_alloc(layout)
            }
        }
    }

    //
    pub unsafe fn kmem_dealloc(&self, buf: *mut u8, _layout: Layout) {
        let obj_addr = buf as usize - core::mem::size_of::<usize>();
        let slab_ptr = Obj::from_addr(obj_addr).slab_addr;
        let slab = Slab::from_addr(slab_ptr);
        let (cache_ptr,ret_info) = slab.slab_dealloc(obj_addr);
        let cache = KmemCache::from_addr(cache_ptr);
        if ret_info == 0{
            cache.kmem_put_full_into_partial(slab_ptr);
        }
        if ret_info == 2{
            cache.kmem_put_partial_into_free(slab_ptr);
        }
        cache.as_mut_ref().obj_free += 1;
    }

    //Calling to FSP_SLAB, Create a KmemCache with a given size.
    //There should not have two Kmem_caches that share the same size
    unsafe fn create_kmem_cache(&self, _obj_size:usize) {
        //try allocate memory from FSP_SLAB's slabs
        match self.kmem_search_slab(){
            //found memory
            Some(addr)=>{
                debug!("create_kmem_cache at {}", addr as usize);
                let cache = KmemCache::from_addr(addr as usize);
                cache.init(_obj_size,addr as usize);
                cache.next = self.next;
                self.as_mut_ref().next = addr as usize;
                cache.kmem_grow();
            }
            //All FSP_SLAB's slabs are full, which wouldn't normally happen. 
            //
            //The first slab we created for FSP_SLAB at the beginning has capacity around 30
            //we normally don't have that many different types of structs that noffseteed SLAB
            None=>{
                self.kmem_grow();
                self.create_kmem_cache(_obj_size)
            }
        }

    }

    //Allocate 4kb of memory from FSP_ALLOC and create a new slab for the cache
    unsafe fn kmem_grow(&self){
        debug!("kmem_grow free_obj{} freehead{} partialhead{} fullhead{}", self.obj_free,self.slabs_free,self.slabs_partial,self.slabs_full);
        let _layout = Layout::from_size_align(PAGE_SIZE, 2);
        match _layout{
            Ok(layout)=>{
                let ptr : *mut u8 = alloc(layout);

                let slab: &mut Slab = Slab::from_addr(ptr as usize);
                slab.init(self.obj_size, ptr.clone() as usize, self as *const KmemCache as usize );

                if self.slabs_free!=0{
                    Slab::from_addr(self.slabs_free).set_prev(ptr as usize);
                    slab.set_next(self.slabs_free);
                }

                self.as_mut_ref().slabs_free = ptr as usize;

                self.as_mut_ref().obj_free += slab.get_free();
                self.as_mut_ref().obj_total += slab.get_capacity();
            }
            Err(_)=>{debug!("Slab failed to construct layout");}
        }

    }

    //Free the first slab in the free list
    //For now, it is only called by kmem_put_partial_into_free(), whenever we have more than one free slab, we will free til we have only one.
    unsafe fn kmem_shrink(&self){
        if self.slabs_free!=0{
            let slab = Slab::from_addr(self.slabs_free);
            self.as_mut_ref().slabs_free = slab.next_slab;
            if self.slabs_free != 0{
                Slab::from_addr(self.slabs_free).set_prev(0);
            }
            let _layout = Layout::from_size_align(PAGE_SIZE, 2);
            match _layout{
                Ok(layout)=>{
                    self.as_mut_ref().obj_total -= slab.capacity;
                    self.as_mut_ref().obj_free -= slab.free; 
                    debug!("kmem_shrink freeing slab at {} next{}", slab.start_addr,slab.next_slab);
                    dealloc(slab.start_addr as *mut u8,layout);
                }
                _=>{debug!("Slab failed to construct layout");}
            }
        }
    }
    
    // this will always be the first slab in partial list
    fn kmem_put_partial_into_full(&self){
        let slab: &mut Slab = Slab::from_addr(self.slabs_partial);
        let tmp = self.slabs_partial;
        debug!("kmem_put_partial_into_full addr{} slab.next{} slab.prev{} self.partial{} self.full{}", self.slabs_partial, slab.next_slab,slab.prev_slab, self.slabs_partial, self.slabs_full);


        if self.slabs_full != 0{
            Slab::from_addr(self.slabs_full).set_prev(tmp.clone());
        }
         if slab.next_slab!=0 {
            Slab::from_addr(slab.next_slab).set_prev( 0 );
        }
        self.as_mut_ref().slabs_partial = slab.next_slab;
        slab.set_next( self.slabs_full);
        self.as_mut_ref().slabs_full = tmp;
        slab.as_mut_ref().set_prev(0);
    }

   // this will always be the first slab in free list
    fn kmem_put_free_into_partial(&self){
        let slab: &mut Slab = Slab::from_addr(self.slabs_free);
        //debug!("kmem_put_free_into_partial addr{} slab.next{} slab.prev{} self.free{} self.partial{}", self.slabs_free, slab.next_slab,slab.prev_slab, self.slabs_free, self.slabs_partial);
        let tmp = self.slabs_free;

        if self.slabs_partial != 0{
            Slab::from_addr(self.slabs_partial).set_prev( tmp.clone() );
        }
        if slab.next_slab!=0 {
            Slab::from_addr(slab.next_slab).set_prev( 0 );
        }
        self.as_mut_ref().slabs_free = slab.next_slab;
        slab.set_next(self.slabs_partial);
        self.as_mut_ref().slabs_partial = tmp;
        slab.as_mut_ref().set_prev(0);
    }

    //put slab at the address into free list 
    unsafe fn kmem_put_partial_into_free(&self,addr:usize){
        let slab = Slab::from_addr(addr);
        //debug!("kmem_put_partial_into_free addr{} slab.next{} slab.prev{} self.partial{} self.free{}", addr, slab.next_slab,slab.prev_slab, self.slabs_partial, self.slabs_free);
        if self.slabs_free != 0{
            self.kmem_shrink();
        }

        if slab.prev_slab!=0{
            Slab::from_addr(slab.prev_slab).set_next(slab.next_slab);
        }
        if self.slabs_partial == addr{
            self.as_mut_ref().slabs_partial = slab.next_slab;
            if slab.next_slab != 0 {
                Slab::from_addr(slab.next_slab).set_prev(0);
            }
        }

        slab.set_next(self.slabs_free);
        if self.slabs_free != 0{
            Slab::from_addr(self.slabs_free).set_prev(addr.clone());
        }
        self.as_mut_ref().slabs_free = addr;
       // debug!("kmem_put_partial_into_free after addr{} slab.next{} slab.prev{} self.partial{} self.free{}", addr, slab.next_slab,slab.prev_slab, self.slabs_partial, self.slabs_free);
    }

    //put slab at the address into partial list
    fn kmem_put_full_into_partial(&self,addr:usize){
        let slab = Slab::from_addr(addr);
        //debug!("kmem_put_full_into_partial addr{} slab.next{} slab.prev{} self.full{} self.partial{}", addr, slab.next_slab,slab.prev_slab, self.slabs_full, self.slabs_partial);
        if slab.prev_slab!=0{
            Slab::from_addr(slab.prev_slab).set_next(slab.next_slab);
        }
        if self.slabs_full == addr{
            self.as_mut_ref().slabs_full = slab.next_slab;
            if slab.next_slab != 0 {
                Slab::from_addr(slab.next_slab).set_prev(0);
            }
        }

        slab.set_next(self.slabs_partial);
        if self.slabs_partial != 0{
            Slab::from_addr(self.slabs_partial).set_prev(addr.clone());
        }
        self.as_mut_ref().slabs_partial = addr;
        //debug!("kmem_put_full_into_partial after addr{} slab.next{} slab.prev{} self.full{} self.partial{}", addr, slab.next_slab,slab.prev_slab, self.slabs_full, self.slabs_partial);
    }
}

impl Slab{
    //The basic unit of Slab allocation
    //One slab is 4kb and size of each object is a power of two, and no smaller than a pointer(usize)
    //The instance of Slab is put into the beginning of slab, and it takes one or more complete objects
    //
    //At the beginning, all objects are free and since objects are no smaller than a pointer, we put a pointer at the beginning of each object
    //which pointing to the address of next free objects
    //doing this, the process of allocating is a one-step thing--return the head of free list and move the head to the next
    //
    //update: Now we also put a offset right before the "next_object" pointer
    //        unlike the points whose space will be used when the object is allocated,
    //        the offset will remain unchanged all the time to keep track of the address of the slab to achive a better dealloc speed.
    fn from_addr(addr: usize) -> &'static mut Slab {
        unsafe { &mut *(addr as *mut Slab) }
    }

    fn as_mut_ref(&self) -> &mut Slab {
        unsafe { &mut *(self as *const Slab as *mut Slab) }
    }

    fn set_next(&self, next: usize) {
        self.as_mut_ref().next_slab = next;
    }

    fn set_prev(&self, prev: usize){
        self.as_mut_ref().prev_slab = prev;
    }

    fn get_free(&self)-> usize{
        self.free
    }

    fn get_capacity(&self)-> usize{
        self.capacity
    }

    //This must be called when creating a new slab for a KmemCache
    pub unsafe fn init(&self, _obj_size:usize, addr:usize,  _cache_addre:usize){

        debug!("Slab init addr{} next{} size{}", addr, 0, _obj_size);

        self.as_mut_ref().inuse = 0;

        self.as_mut_ref().start_addr = addr.clone();   
        self.as_mut_ref().end_addr = addr.clone()+PAGE_SIZE;
        
        self.as_mut_ref().cache_addr = _cache_addre;

        self.as_mut_ref().next_slab = 0;
        self.as_mut_ref().prev_slab = 0;

        self.as_mut_ref().obj_size = _obj_size;
        let _obj_num = PAGE_SIZE/_obj_size;
        let self_space = (core::mem::size_of::<Slab>() + (_obj_size - 1)) & (!(_obj_size - 1));
        self.as_mut_ref().capacity = _obj_num - self_space/_obj_size;
        self.as_mut_ref().free = self.capacity;

        self.as_mut_ref().free_head = addr + self_space;

        let mut ptr:usize = addr + self_space;
        for _i in 1..self.capacity{
            Obj::from_addr(ptr).next_obj = ptr + _obj_size;
            Obj::from_addr(ptr).slab_addr =  self as *const Slab as usize ;
            ptr = ptr + _obj_size;
        }
        //last object, set its next to 0
        Obj::from_addr(ptr).next_obj = 0;
        Obj::from_addr(ptr).slab_addr = self as *const Slab as usize ;
        
    }
    

    pub unsafe fn slab_alloc(&self) -> *mut u8{
        let ret: *mut u8 = (self.free_head + core::mem::size_of::<usize>()) as *mut u8;
        self.as_mut_ref().free_head = Obj::from_addr(self.free_head).next_obj;
        self.as_mut_ref().inuse += 1;
        self.as_mut_ref().free -= 1;
        //debug!("alloc addr{} inuse{} free{} capa{}", self as *const Slab as usize, self.inuse,self.free,self.capacity);
        ret
    }

    //sencond felid of return value: return info
    //0->from full to partial, 1-> keep in partial, 2-> partial to free
    pub unsafe fn slab_dealloc(&self,addr:usize)->(usize,usize) {
        let tmp = self.free_head;
        self.as_mut_ref().free_head = addr;
        Obj::from_addr(self.free_head).next_obj = tmp;
        self.as_mut_ref().inuse -= 1;
        self.as_mut_ref().free += 1;
        //debug!("dealloc addr{} inuse{} free{} capa{}", self as *const Slab as usize, self.inuse,self.free,self.capacity);
        if self.capacity -1 == self.inuse{
            return (self.cache_addr,0)
        }
        if self.free == self.capacity{
            return (self.cache_addr,2)
        }
        return (self.cache_addr,1)
    }
}