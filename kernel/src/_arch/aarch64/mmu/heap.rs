use core::alloc::{GlobalAlloc, Layout};

pub struct HeapAllocator {}

unsafe impl GlobalAlloc for HeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        0x80000 as *mut u8
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {}
}

#[global_allocator]
static HEAP_ALLOCATOR: HeapAllocator = HeapAllocator {};
