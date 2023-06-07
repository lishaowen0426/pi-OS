use super::*;
use crate::{errno::*, println, static_vector, type_enum, utils::bitfields::Bitfields};
use core::{
    alloc::{GlobalAlloc, Layout},
    cell::{SyncUnsafeCell, UnsafeCell},
    fmt,
    ops::Deref,
};
use intrusive_collections::linked_list::AtomicLinkOps;
use spin::{mutex::SpinMutex, once::Once, Spin};

const BACKEND_FREE_4K: usize = 16;
const BACKEND_FREE_2M: usize = 8;
const OBJECT_PAGE_PER_SIZE_CLASS: usize = 8;

type AllocationMap = [u64; 8];

#[repr(C)]
struct ObjectPage<const SIZE: usize>
where
    [(); SIZE - core::mem::size_of::<AllocationMap>()]:,
{
    data: [u8; SIZE - core::mem::size_of::<AllocationMap>()],

    allocated: AllocationMap, // 1 means the location is allocated
}

impl<const SIZE: usize> ObjectPage<SIZE>
where
    [(); SIZE - core::mem::size_of::<AllocationMap>()]:,
{
    const METADATA_SIZE: usize = core::mem::size_of::<[u64; 8]>();

    pub fn alloc(&mut self, layout: Layout) -> Option<*mut u8> {
        let offset = self.allocated.first_zero();
        let ptr: usize = self.data.as_ptr() as usize + offset * layout.size();

        // check the bound
        if ptr + layout.size() > (self.data.as_ptr() as usize + self.data.len()) {
            return None;
        }
        self.allocated.set_bit(offset, 1);
        Some(ptr as *mut u8)
    }
    pub fn dealloc(&mut self, ptr: *mut u8, layout: Layout) -> Result<(), ErrorCode> {
        let base = self.data.as_ptr() as usize;
        let diff = ptr as usize - base;
        if diff % layout.size() != 0 {
            Err(EALIGN)
        } else if diff + layout.size() > self.data.len() {
            Err(EOVERFLOW)
        } else {
            let offset = diff / layout.size();
            self.allocated.set_bit(offset, 0);
            Ok(())
        }
    }
}

type ObjectPage4K = ObjectPage<0x1000>;
type ObjectPage2M = ObjectPage<0x200000>;

static_vector!(Free4KVec, VaRange, BACKEND_FREE_4K);
static_vector!(Free2MVec, VaRange, BACKEND_FREE_2M);
static_vector!(ObjectPageVec, VaRange, OBJECT_PAGE_PER_SIZE_CLASS);

struct HeapBackend {
    free_4K: Free4KVec,
    free_2M: Free2MVec,
}
struct HeapFrontend {
    sc_allocator: [SizeClassAllocator; 9],
}

type_enum!(
    enum Log2SizeClass {
        SZ_8 = 3,
        SZ_16 = 4,
        SZ_32 = 5,
        SZ_64 = 6,
        SZ_128 = 7,
        SZ_256 = 8,
        SZ_512 = 9,
        SZ_1024 = 10,
        SZ_2048 = 11,
    }
);
struct SizeClassAllocator {
    pages: ObjectPageVec,
    size_class: Log2SizeClass,
}

impl SizeClassAllocator {
    pub const fn new(sc: Log2SizeClass) -> Self {
        Self {
            pages: ObjectPageVec::new(),
            size_class: sc,
        }
    }

    pub fn refill(&mut self, va_range: VaRange) -> Result<(), ErrorCode> {
        self.pages.push(va_range)
    }
    pub fn alloc(&mut self, layout: Layout) -> Option<*mut u8> {
        println!("sz = {}, {:?}", 1 << (self.size_class as u8), layout);
        for p in self.pages.iter() {
            if let Some(vr) = p.as_ref() {
                let obj_page: &mut ObjectPage4K =
                    unsafe { core::mem::transmute::<usize, &mut ObjectPage4K>(vr.start().value()) };

                if let Some(ptr) = obj_page.alloc(layout) {
                    return Some(ptr);
                }
            }
        }
        None
    }
}

struct UnsafeHeapAllocator {
    backend: HeapBackend,
    frontend: HeapFrontend,
}

impl HeapBackend {
    pub fn new() -> Self {
        Self {
            free_4K: Free4KVec::new(),
            free_2M: Free2MVec::new(),
        }
    }
    fn insert(&mut self, va: VaRange) -> Result<(), ErrorCode> {
        if va.is_4K() {
            self.free_4K.push(va)
        } else if va.is_2M() {
            self.free_2M.push(va)
        } else {
            Err(EPARAM)
        }
    }

    fn request_4K_from_system(&mut self) -> Result<(), ErrorCode> {
        let mapped = super::MMU
            .get()
            .unwrap()
            .kzalloc(BLOCK_4K, RWNORMAL, HIGHER_PAGE)?;

        self.insert(mapped.va)
    }

    pub fn allocate_4K_free(&mut self) -> Result<VaRange, ErrorCode> {
        if let Some(v) = self.free_4K.pop() {
            Ok(v)
        } else {
            self.request_4K_from_system()?;
            if let Some(vv) = self.free_4K.pop() {
                Ok(vv)
            } else {
                Err(EUNKNOWN)
            }
        }
    }
}

impl HeapFrontend {
    pub fn pick_size_class(v: u64) -> Log2SizeClass {
        if v <= 8 {
            Log2SizeClass::SZ_8
        } else {
            let mut n = v - 1;
            n |= n >> 1; // Divide by 2^k for consecutive doublings of k up to 32,
            n |= n >> 2; // and then or the results.
            n |= n >> 4;
            n |= n >> 8;
            n |= n >> 16;
            n |= n >> 32;
            n = n + 1;
            Log2SizeClass::from(n.ilog2())
        }
    }
    pub const fn new() -> Self {
        Self {
            sc_allocator: [
                SizeClassAllocator::new(Log2SizeClass::SZ_8),
                SizeClassAllocator::new(Log2SizeClass::SZ_16),
                SizeClassAllocator::new(Log2SizeClass::SZ_32),
                SizeClassAllocator::new(Log2SizeClass::SZ_64),
                SizeClassAllocator::new(Log2SizeClass::SZ_128),
                SizeClassAllocator::new(Log2SizeClass::SZ_256),
                SizeClassAllocator::new(Log2SizeClass::SZ_512),
                SizeClassAllocator::new(Log2SizeClass::SZ_1024),
                SizeClassAllocator::new(Log2SizeClass::SZ_2048),
            ],
        }
    }

    pub fn refill(&mut self, va_range: VaRange, layout: Layout) -> Result<(), ErrorCode> {
        let sc = Self::pick_size_class(layout.size() as u64) as usize;
        self.sc_allocator[sc - 3].refill(va_range)
    }

    pub fn alloc(&mut self, layout: Layout) -> Option<*mut u8> {
        let sz = Self::pick_size_class(layout.size() as u64) as usize;
        self.sc_allocator[sz - 3].alloc(layout)
    }
}

impl UnsafeHeapAllocator {
    pub fn new() -> Self {
        Self {
            backend: HeapBackend::new(),
            frontend: HeapFrontend::new(),
        }
    }

    pub fn init(&mut self, va: VaRange) -> Result<(), ErrorCode> {
        self.backend.insert(va)
    }

    pub fn alloc(&mut self, layout: Layout) -> Option<*mut u8> {
        if let Some(ptr) = self.frontend.alloc(layout) {
            Some(ptr)
        } else {
            let va_range_4k = self.backend.allocate_4K_free().ok()?;
            self.frontend.refill(va_range_4k, layout).ok()?;

            self.frontend.alloc(layout)
        }
    }
}

pub struct HeapAllocator {
    allocator: SpinMutex<UnsafeHeapAllocator>,
}

impl HeapAllocator {
    pub fn new() -> Self {
        Self {
            allocator: SpinMutex::new(UnsafeHeapAllocator::new()),
        }
    }

    pub fn init(&self, va: VaRange) -> Result<(), ErrorCode> {
        self.allocator.lock().init(va)
    }
    fn alloc(&self, layout: Layout) -> *mut u8 {
        if let Some(ptr) = self.allocator.lock().alloc(layout) {
            ptr
        } else {
            core::ptr::null_mut()
        }
    }
}

#[no_mangle]
#[inline(never)]
pub fn heap_init(va: VaRange) -> Result<(), ErrorCode> {
    HEAP_ALLOCATOR.call_once(|| HeapAllocator::new());
    HEAP_ALLOCATOR.get().unwrap().init(va)
}
pub static HEAP_ALLOCATOR: Once<HeapAllocator> = Once::new();

pub struct Heap {}

impl Heap {
    pub const fn new() -> Self {
        Self {}
    }
}

#[global_allocator]
pub static HEAP: Heap = Heap::new();

unsafe impl GlobalAlloc for Heap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        HEAP_ALLOCATOR.get().unwrap().alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::println;
    use test_macros::kernel_test;

    #[kernel_test]
    fn test_heap() {
        assert_eq!(HeapFrontend::pick_size_class(0), Log2SizeClass::SZ_8);
        assert_eq!(HeapFrontend::pick_size_class(8), Log2SizeClass::SZ_8);
        assert_eq!(HeapFrontend::pick_size_class(15), Log2SizeClass::SZ_16);
        assert_eq!(HeapFrontend::pick_size_class(1020), Log2SizeClass::SZ_1024);
        assert_eq!(HeapFrontend::pick_size_class(2044), Log2SizeClass::SZ_2048);
        assert_eq!(
            HeapFrontend::pick_size_class(4000),
            Log2SizeClass::Undefined
        );
    }
}
