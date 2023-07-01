use super::*;
use crate::{
    errno::*,
    generics::{DoublyLink, DoublyLinkable, DoublyLinkedList, Link},
    print, println, static_vector, type_enum, type_enum_with_error,
    utils::bitfields::Bitfields,
};
use core::{
    alloc::{GlobalAlloc, Layout},
    fmt,
    iter::Iterator,
    marker::PhantomData,
    ops::{Deref, Drop},
};

use spin::{mutex::SpinMutex, once::Once};
use test_macros::impl_doubly_linkable;

const BACKEND_FREE_4K: usize = 16;
const BACKEND_FREE_2M: usize = 8;
const OBJECT_PAGE_PER_SIZE_CLASS: usize = 8;
const SLABS_LENGTH_LIMIT: usize = 4; // the maximum number of object page a szallocator can keep for allocation

type AllocationMap = [u64; 8];

#[impl_doubly_linkable]
#[repr(C)]
struct ObjectPage<const SIZE: usize>
where
    [(); SIZE - core::mem::size_of::<AllocationMap>() - 3 * core::mem::size_of::<usize>()]:,
{
    // 1 means the location is allocated
    // index 0 starts from the rightmost bit of the last array element
    // note this is different from how we index an array
    // but to keep consistent with a very big integer made up by concatenating multiple u64

    // we at most use 502 bits for the smallest size class 8 bytes
    // (4096 - 64 -8 - 8) / 8 = 502
    // we use the top bit as an indication of whether the page is full
    // since index stars from the last element
    // this is actually the first element
    allocated: AllocationMap,
    count: usize,
    doubly_link: DoublyLink<Self>,

    data: [u8; SIZE - core::mem::size_of::<AllocationMap>() - 3 * core::mem::size_of::<usize>()],
}

impl<const SIZE: usize> ObjectPage<SIZE>
where
    [(); SIZE - core::mem::size_of::<AllocationMap>() - 3 * core::mem::size_of::<usize>()]:,
{
    const FULL_BIT: usize = 63;

    fn init(&mut self, sc: Log2SizeClass) -> Result<(), ErrorCode> {
        // check if the page is empty
        for u in self.allocated.iter() {
            if *u != 0 {
                return Err(EINVAL);
            }
        }

        Ok(())
    }
    fn mark_full(&mut self) {
        self.allocated[0].set_bit(Self::FULL_BIT, 1)
    }
    fn clear_full(&mut self) {
        self.allocated[0].set_bit(Self::FULL_BIT, 0)
    }
    fn is_marked_full(&self) -> bool {
        self.allocated[0].get_bit(Self::FULL_BIT) == 1
    }

    fn count(&self) -> usize {
        self.count
    }

    fn alloc(&mut self, layout: Layout) -> Option<*mut u8> {
        let offset = self.allocated.first_zero();
        let ptr: usize = self.data.as_ptr() as usize + offset * layout.size();

        // check the bound
        if ptr + layout.size() > (self.data.as_ptr() as usize + self.data.len()) {
            return None;
        }
        if self.allocated.get_bit(offset) == 1 {
            return None;
        }

        let allocated = ptr as *mut u8;
        let alignment = layout.align();
        if !allocated.is_aligned_to(alignment) {
            return None;
        }

        self.allocated.set_bit(offset, 1);
        self.count = self.count + 1;
        Some(allocated)
    }
    fn dealloc(&mut self, ptr: *mut u8, layout: Layout) -> Result<(), ErrorCode> {
        let base = self.data.as_ptr() as usize;

        let diff = ptr as usize - base;
        if diff % layout.size() != 0 {
            Err(EALIGN)
        } else if diff + layout.size() > self.data.len() {
            Err(EOVERFLOW)
        } else {
            unsafe {
                ptr.write_bytes(0, layout.size());
            }

            let offset = diff / layout.size();
            self.allocated.set_bit(offset, 0);
            self.count = self.count - 1;
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
    free_4K: DoublyLinkedList<ObjectPage4K>,
    free_2M: DoublyLinkedList<ObjectPage2M>,
}
struct HeapFrontend {
    sc_allocator: [SizeClassAllocator; 10],
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
        SZ_LARGE = 12,
    }
);

impl Log2SizeClass {
    fn to_bytes(&self) -> usize {
        match *self {
            Log2SizeClass::Undefined => 0,
            _ => 1 << *self as u8,
        }
    }
}

struct SizeClassAllocator {
    slabs: DoublyLinkedList<ObjectPage4K>,
    full_slabs: DoublyLinkedList<ObjectPage4K>,
    size_class: Log2SizeClass,
}

impl SizeClassAllocator {
    pub const fn new(sc: Log2SizeClass) -> Self {
        Self {
            slabs: DoublyLinkedList::new(),
            full_slabs: DoublyLinkedList::new(),
            size_class: sc,
        }
    }

    pub fn refill(&mut self, start: usize) -> Result<(), ErrorCode> {
        self.slabs.push_front(Link::some(start));
        Ok(())
    }
    pub fn alloc(&mut self, layout: Layout) -> Option<*mut u8> {
        for l in self.slabs.iter() {
            let obj: &mut ObjectPage4K = l.resolve_mut();
            if let Some(p) = obj.alloc(layout) {
                return Some(p);
            } else {
                // Check DoublyLinkedlistIteratror
                // when next() returns, the iter is already pointing to the next
                obj.mark_full();
                self.slabs.remove(l);
                self.full_slabs.push_front(l);
            }
        }
        None
    }

    // ptr is always within the 4K page from the base of the ObjectPage4K
    //
    fn dealloc(&mut self, ptr: *mut u8, layout: Layout) -> Option<VirtualAddress> {
        let va = VirtualAddress::from(ptr as usize);
        let obj_base = va.align_to_4K_up();
        let link = Link::some(obj_base.value());
        let obj: &mut ObjectPage4K = link.resolve_mut();
        obj.dealloc(ptr, layout).unwrap();

        if obj.is_marked_full() {
            obj.clear_full();
            self.full_slabs.remove(link);
            self.slabs.push_back(link);
        }

        if obj.count() == 0 {
            if self.slabs.len() > SLABS_LENGTH_LIMIT {
                // return the empty free page to the backend
                self.slabs.remove(link);
                Some(obj_base)
            } else {
                None
            }
        } else {
            None
        }
    }
}

struct UnsafeHeapAllocator {
    backend: HeapBackend,
    frontend: HeapFrontend,
}

impl HeapBackend {
    pub fn new() -> Self {
        Self {
            free_4K: DoublyLinkedList::new(),
            free_2M: DoublyLinkedList::new(),
        }
    }
    fn insert(&mut self, va: VaRange) -> Result<(), ErrorCode> {
        if va.is_4K() {
            let l = Link::some(va.start().value());
            self.free_4K.push_front(l);
            Ok(())
        } else if va.is_2M() {
            let l = Link::some(va.start().value());
            self.free_2M.push_front(l);
            Ok(())
        } else {
            Err(EPARAM)
        }
    }

    fn request_4K_from_system(&mut self) -> Result<(), ErrorCode> {
        let mapped = super::MMU
            .get()
            .unwrap()
            .kzalloc(1, RWNORMAL, HIGHER_PAGE)?;

        self.insert(mapped.va)
    }

    pub fn allocate_4K_free(&mut self) -> Option<usize> {
        self.free_4K.pop_front().map(|l| l.ptr())
    }
}

impl HeapFrontend {
    // pub fn pick_size_class(v: u64) -> Log2SizeClass {
    // if v <= 8 {
    // Log2SizeClass::SZ_8
    // } else {
    // let mut n = v - 1;
    // n |= n >> 1; // Divide by 2^k for consecutive doublings of k up to 32,
    // n |= n >> 2; // and then or the results.
    // n |= n >> 4;
    // n |= n >> 8;
    // n |= n >> 16;
    // n |= n >> 32;
    // n = n + 1;
    // Log2SizeClass::from(n.ilog2())
    // }
    // }
    //
    pub fn pick_size_class(v: u64) -> Log2SizeClass {
        match v {
            0..9 => Log2SizeClass::SZ_8,
            9..17 => Log2SizeClass::SZ_16,
            17..33 => Log2SizeClass::SZ_32,
            33..65 => Log2SizeClass::SZ_64,
            65..129 => Log2SizeClass::SZ_128,
            129..257 => Log2SizeClass::SZ_256,
            257..513 => Log2SizeClass::SZ_512,
            513..1025 => Log2SizeClass::SZ_1024,
            1025..2049 => Log2SizeClass::SZ_2048,
            2049..4000 => Log2SizeClass::SZ_LARGE,
            _ => Log2SizeClass::Undefined,
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
                SizeClassAllocator::new(Log2SizeClass::SZ_LARGE),
            ],
        }
    }

    pub fn refill(&mut self, start: usize, layout: Layout) -> Result<(), ErrorCode> {
        let sc = Self::pick_size_class(layout.size() as u64);
        println!("layout {:?}, sc = {}", layout, sc);
        if sc == Log2SizeClass::Undefined {
            todo!()
        }
        self.sc_allocator[sc as usize - 3].refill(start)
    }

    pub fn alloc(&mut self, layout: Layout) -> Option<*mut u8> {
        let sz = Self::pick_size_class(layout.size() as u64) as usize;
        if (sz - 3) >= self.sc_allocator.len() {
            return None;
        }
        self.sc_allocator[sz - 3].alloc(layout)
    }
    fn dealloc(
        &mut self,
        ptr: *mut u8,
        layout: Layout,
    ) -> Result<Option<VirtualAddress>, ErrorCode> {
        let sz = Self::pick_size_class(layout.size() as u64) as usize;
        if (sz - 3) >= self.sc_allocator.len() {
            return Err(ESUPPORTED);
        }
        Ok(self.sc_allocator[sz - 3].dealloc(ptr, layout))
    }
}

impl UnsafeHeapAllocator {
    fn new() -> Self {
        Self {
            backend: HeapBackend::new(),
            frontend: HeapFrontend::new(),
        }
    }

    fn init(&mut self, va: VaRange) -> Result<(), ErrorCode> {
        let mut va_copy = va;
        while let Some(v) = va_copy.pop_4K_front() {
            self.backend
                .insert(VaRange::new(v, v + VirtualAddress::_4K))?;
        }
        Ok(())
    }

    fn alloc(&mut self, layout: Layout) -> Option<*mut u8> {
        self.frontend.alloc(layout)
    }
    fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        if let Some(v) = self.frontend.dealloc(ptr, layout).unwrap() {
            // first try to return to the backend
            // it its full, let MMU unmap it.
            self.backend
                .insert(VaRange::new(v, v + VirtualAddress::_4K))
                .or_else(|_| MMU.get().unwrap().unmap(v))
                .unwrap();
        }
    }
}

pub struct BumpBuffer {
    start: usize,
    next: usize,
    end: usize, // exclusive
}

impl Iterator for BumpBuffer {
    type Item = *mut u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.next >= self.end {
            None
        } else {
            self.next = self.next + 1;
            Some((self.next - 1) as *mut u8)
        }
    }
}

impl BumpBuffer {
    pub fn construct<T>(&mut self) -> Result<*mut T, ErrorCode> {
        let layout = Layout::new::<T>();
        let align_offset = (self.next as *mut u8).align_offset(layout.align());

        if self.next + align_offset + layout.size() > self.end {
            Err(EOVERFLOW)
        } else {
            self.next = self.next + align_offset;
            let ptr: *mut T = self.next as *mut T;
            self.next = self.next + layout.size();
            Ok(ptr)
        }
    }
    pub fn construct_n<T>(&mut self, len: usize) -> Result<*mut T, ErrorCode> {
        let layout = Layout::new::<T>();
        let align_offset = (self.next as *mut u8).align_offset(layout.align());

        if self.next + align_offset + len * layout.size() > self.end {
            Err(EOVERFLOW)
        } else {
            self.next = self.next + align_offset;
            let ptr: *mut T = self.next as *mut T;
            self.next = self.next + len * layout.size();
            Ok(ptr)
        }
    }

    pub fn alloc_n(&mut self, len: usize) -> Result<*mut u8, ErrorCode> {
        let ptr = self.next as *mut u8;
        if let Ok(_) = self.advance_by(len) {
            Ok(ptr)
        } else {
            Err(EOVERFLOW)
        }
    }
}

impl BumpBuffer {
    pub fn new(range: VaRange) -> Self {
        Self {
            start: range.start().value(),
            next: range.start().value(),
            end: range.end().value(),
        }
    }
}

impl Drop for BumpBuffer {
    fn drop(&mut self) {
        MMU.get()
            .unwrap()
            .unmap(VirtualAddress::from(self.start))
            .unwrap();
    }
}

pub struct HeapAllocator {
    allocator: SpinMutex<UnsafeHeapAllocator>,
}

impl HeapAllocator {
    fn new() -> Self {
        Self {
            allocator: SpinMutex::new(UnsafeHeapAllocator::new()),
        }
    }

    fn init(&self, va: VaRange) -> Result<(), ErrorCode> {
        self.allocator.lock().init(va)
    }

    fn alloc(&self, layout: Layout) -> *mut u8 {
        if let Some(ptr) = self.allocator.lock().alloc(layout) {
            return ptr;
        }

        if layout.size() > 4096 {
            todo!()
        }

        let mut free_4k_page = if let Some(p) = self.allocator.lock().backend.allocate_4K_free() {
            p
        } else {
            0
        };
        if free_4k_page == 0 {
            const ADDRESS_NODE_SIZE: usize = Layout::new::<AddressRangeNode<VaRange>>().size();

            if layout.size() == ADDRESS_NODE_SIZE {
                // since in kzalloc, page/frame_allocator will call Box<AddressRangeNode<*aRange>>
                // this requires special handl
                todo!()
            }

            free_4k_page =
                if let Ok(mapped) = super::MMU.get().unwrap().kzalloc(1, RWNORMAL, HIGHER_PAGE) {
                    mapped.va.start().value()
                } else {
                    0
                };
        }

        if free_4k_page == 0 {
            return core::ptr::null::<u8>() as *mut u8;
        }

        if self
            .allocator
            .lock()
            .frontend
            .refill(free_4k_page, layout)
            .is_err()
        {
            return core::ptr::null::<u8>() as *mut u8;
        }

        if let Some(p) = self.allocator.lock().frontend.alloc(layout) {
            p
        } else {
            core::ptr::null::<u8>() as *mut u8
        }
    }

    fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.allocator.lock().dealloc(ptr, layout)
    }

    pub fn alloc_bump_buffer(&self, npage: usize) -> Result<BumpBuffer, ErrorCode> {
        let mapped = MMU.get().unwrap().kzalloc(npage, RWNORMAL, HIGHER_PAGE)?;
        Ok(BumpBuffer::new(mapped.va))
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
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        HEAP_ALLOCATOR.get().unwrap().dealloc(ptr, layout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_macros::kernel_test;
    #[derive(Debug)]
    #[repr(C)]
    struct T {
        a: [u8; 16],
    }

    impl T {
        fn new() -> Self {
            Self { a: [1; 16] }
        }
    }

    #[kernel_test]
    fn test_heap() {
        {
            let mut buffer = HEAP_ALLOCATOR.get().unwrap().alloc_bump_buffer(1).unwrap();
            let mut i = 0;
            for i in 0..4096 / core::mem::size_of::<T>() {
                buffer.construct::<T>().unwrap();
            }
            assert!(buffer.construct::<T>().is_err());
        }
        {
            let mut buffer = HEAP_ALLOCATOR.get().unwrap().alloc_bump_buffer(1).unwrap();
            let arr = buffer
                .construct_n::<T>(4096 / core::mem::size_of::<T>())
                .unwrap();

            assert!(buffer.construct::<T>().is_err());
        }
    }
}
