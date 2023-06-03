use super::address::*;
use crate::{errno::*, static_vector};
use core::alloc::{GlobalAlloc, Layout};
use spin::mutex::SpinMutex;

const BACKEND_FREE_4K: usize = 16;
const BACKEND_FREE_2M: usize = 8;

static_vector!(Free4KVec, VaRange, BACKEND_FREE_4K);
static_vector!(Free2MVec, VaRange, BACKEND_FREE_2M);

struct HeapBackend {
    free_4K: Free4KVec,
    free_2M: Free2MVec,
}
struct HeapFrontend {}

struct UnsafeHeapAllocator {
    backend: HeapBackend,
    frontend: HeapFrontend,
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
        Ok(())
    }

    pub fn fill_backend_with(&mut self, va: VaRange) -> Result<(), ErrorCode> {
        todo!()
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

unsafe impl GlobalAlloc for HeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        todo!()
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {}
}

pub fn init(va: VaRange) -> Result<(), ErrorCode> {
    HEAP_ALLOCATOR.init(va)
}

#[global_allocator]
pub static HEAP_ALLOCATOR: HeapAllocator = HeapAllocator::new();
