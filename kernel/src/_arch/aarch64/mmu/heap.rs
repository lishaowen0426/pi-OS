use super::address::*;
use crate::{errno::*, println, static_vector};
use core::alloc::{GlobalAlloc, Layout};
use spin::{mutex::SpinMutex, once::Once};

const BACKEND_FREE_4K: usize = 16;
const BACKEND_FREE_2M: usize = 8;

#[repr(C)]
struct ObjectPage4K {
    data: [u8; ObjectPage4K::SIZE - ObjectPage4K::METADATA_SIZE],

    allocated: [u64; 8], // 1 means the location is allocated
}

impl ObjectPage4K {
    const SIZE: usize = 4096;
    const METADATA_SIZE: usize = core::mem::size_of::<[u64; 8]>();
}

static_vector!(Free4KVec, VirtualAddress, BACKEND_FREE_4K);
static_vector!(Free2MVec, VirtualAddress, BACKEND_FREE_2M);

struct HeapBackend {
    free_4K: Free4KVec,
    free_2M: Free2MVec,
}
struct HeapFrontend {}

struct UnsafeHeapAllocator {
    backend: HeapBackend,
    frontend: HeapFrontend,
}

impl HeapBackend {
    fn insert(&mut self, va: VaRange) -> Result<(), ErrorCode> {
        if va.is_4K() {
            self.free_4K.push(va.start())
        } else if va.is_2M() {
            self.free_2M.push(va.start())
        } else {
            Err(EPARAM)
        }
    }
}

impl UnsafeHeapAllocator {
    pub const fn new() -> Self {
        Self {
            backend: HeapBackend {
                free_4K: Free4KVec::new(),
                free_2M: Free2MVec::new(),
            },
            frontend: HeapFrontend {},
        }
    }

    pub fn init(&mut self, va: VaRange) -> Result<(), ErrorCode> {
        self.backend.insert(va)
    }

    pub fn alloc(&mut self, layout: Layout) -> *mut u8 {
        core::ptr::null_mut()
    }
}

pub struct HeapAllocator {
    allocator: SpinMutex<UnsafeHeapAllocator>,
}

impl HeapAllocator {
    pub const fn new() -> Self {
        Self {
            allocator: SpinMutex::new(UnsafeHeapAllocator::new()),
        }
    }

    pub fn init(&self, va: VaRange) -> Result<(), ErrorCode> {
        self.allocator.lock().init(va)
    }
}

pub fn init(va: VaRange) -> Result<(), ErrorCode> {
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
        core::ptr::null_mut()
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {}
}
