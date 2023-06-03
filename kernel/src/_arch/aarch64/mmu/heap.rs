use super::address::*;
use crate::errno::ErrorCode;
use core::alloc::{GlobalAlloc, Layout};
use spin::mutex::SpinMutex;

const BACKEND_FREE_4K: usize = 16;
const BACKEND_FREE_2M: usize = 8;
#[derive(Default)]
struct HeapBackend {
    free_4k: [VaRange; BACKEND_FREE_4K],
    free_2M: [VaRange; BACKEND_FREE_2M],
}
#[derive(Default)]
struct HeapFrontend {}

#[derive(Default)]
struct UnsafeHeapAllocator {
    backend: Option<HeapBackend>,
    frontend: Option<HeapFrontend>,
}

impl UnsafeHeapAllocator {
    pub const fn new() -> Self {
        Self {
            backend: None,
            frontend: None,
        }
    }

    pub fn init(&mut self) -> Result<(), ErrorCode> {
        self.backend = Some(Default::default());
        self.frontend = Some(Default::default());
        Ok(())
    }

    pub fn fill_backend_with(va: VaRange) {}
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

    pub fn init(&self) -> Result<(), ErrorCode> {
        self.allocator.lock().init()
    }
}

unsafe impl GlobalAlloc for HeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        todo!()
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {}
}

pub fn init() -> Result<(), ErrorCode> {
    HEAP_ALLOCATOR.init()
}

#[global_allocator]
pub static HEAP_ALLOCATOR: HeapAllocator = HeapAllocator::new();
