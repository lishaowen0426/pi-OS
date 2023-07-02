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
    num,
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

#[impl_doubly_linkable]
#[derive(Copy, Clone)]
#[repr(C)]
struct Span {
    start: VirtualAddress,
    num_pages: usize,
    doubly_link: DoublyLink<Self>,
    padding: [u8; allocator::ADDRESS_RANGE_NODE_SIZE
        - core::mem::size_of::<VirtualAddress>()
        - core::mem::size_of::<usize>()
        - core::mem::size_of::<DoublyLink<Self>>()],
}

impl Span {
    fn new(start: VirtualAddress, num_pages: usize) -> Self {
        Self {
            start,
            num_pages,
            doubly_link: Default::default(),
            padding: [0; allocator::ADDRESS_RANGE_NODE_SIZE
                - core::mem::size_of::<VirtualAddress>()
                - core::mem::size_of::<usize>()
                - core::mem::size_of::<DoublyLink<Self>>()],
        }
    }
}

const FREE_BIG_OBJECT_LIST_LENGTH_LIMIT: usize = 16;
// big_objects are pages of 4kb, 8kb, 32kb, 256kb
struct HeapBackend {
    free_4K: DoublyLinkedList<ObjectPage4K>,
    free_2M: DoublyLinkedList<ObjectPage2M>,
    free_big_objects: [DoublyLinkedList<Span>; 4],
    allocated_big_objects: [DoublyLinkedList<Span>; 4],
}
struct HeapFrontend {
    sc_allocator: [Spinlock<SizeClassAllocator>; 9],
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
            let sz_layout =
                Layout::from_size_align(self.size_class.to_bytes(), layout.align()).unwrap();
            if let Some(p) = obj.alloc(sz_layout) {
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
        let sz_layout =
            Layout::from_size_align(self.size_class.to_bytes(), layout.align()).unwrap();
        obj.dealloc(ptr, sz_layout).unwrap();

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
    backend: Spinlock<HeapBackend>,
    frontend: HeapFrontend,
}

type_enum!(
    enum BigObjectSizeClass {
        SZ_4K = 0,
        SZ_8K = 1,
        SZ_32K = 2,
        SZ_256K = 3,
    }
);

impl BigObjectSizeClass {
    fn to_npage(&self) -> usize {
        match *self {
            Self::SZ_4K => 1,
            Self::SZ_8K => 2,
            Self::SZ_32K => 4,
            Self::SZ_256K => 64,
            _ => 0,
        }
    }
}

impl HeapBackend {
    fn pick_big_object_size(sz: usize) -> BigObjectSizeClass {
        match sz {
            0..4097 => BigObjectSizeClass::SZ_4K,
            4097..8193 => BigObjectSizeClass::SZ_8K,
            8193..16385 => BigObjectSizeClass::SZ_32K,
            16385..262145 => BigObjectSizeClass::SZ_256K,
            _ => BigObjectSizeClass::Undefined,
        }
    }

    pub fn new() -> Self {
        Self {
            free_4K: DoublyLinkedList::new(),
            free_2M: DoublyLinkedList::new(),
            free_big_objects: [DoublyLinkedList::new(); 4],
            allocated_big_objects: [DoublyLinkedList::new(); 4],
        }
    }

    fn insert_4K(&mut self, va: VaRange) -> Result<(), ErrorCode> {
        if !va.is_4K_multiple() {
            Err(EPARAM)
        } else {
            let mut va_copy = va;
            while let Some(v) = va_copy.pop_4K_front() {
                let l = Link::some(v.value());
                self.free_4K.push_front(l);
            }
            Ok(())
        }
    }
    fn insert_2M(&mut self, va: VaRange) -> Result<(), ErrorCode> {
        if !va.is_2M_multiple() {
            Err(EPARAM)
        } else {
            let mut va_copy = va;
            while let Some(v) = va_copy.pop_2M_front() {
                let l = Link::some(v.value());
                self.free_2M.push_front(l);
            }
            Ok(())
        }
    }

    fn request_4K_from_system(&mut self) -> Result<(), ErrorCode> {
        let mapped = super::MMU
            .get()
            .unwrap()
            .kzalloc(1, RWNORMAL, HIGHER_PAGE)?;

        self.insert_4K(mapped.va)
    }

    pub fn allocate_4K_free(&mut self) -> Option<usize> {
        self.free_4K.pop_front().map(|l| l.ptr())
    }

    fn refill_big(&mut self, mapped: Mapped) -> Result<(), ErrorCode> {
        if !mapped.va.is_4K_multiple() {
            return Err(EALIGN);
        }

        let sc = match mapped.va.size_in_bytes() / 4096 {
            1 => BigObjectSizeClass::SZ_4K,
            2 => BigObjectSizeClass::SZ_8K,
            4 => BigObjectSizeClass::SZ_32K,
            64 => BigObjectSizeClass::SZ_256K,
            _ => BigObjectSizeClass::Undefined,
        } as usize;

        if sc >= self.free_big_objects.len() {
            Err(EINVAL)
        } else {
            let span = Box::new(Span::new(
                mapped.va.start(),
                mapped.va.size_in_bytes() / 4096,
            ));
            let span = Box::leak(span) as *mut Span;
            let link = Link::<Span>::some(span as usize);
            self.free_big_objects[sc].push_front(link);
            Ok(())
        }
    }

    fn alloc_big(&mut self, layout: Layout) -> Option<*mut u8> {
        let sc = Self::pick_big_object_size(layout.size()) as usize;
        if sc >= self.free_big_objects.len() {
            None
        } else if let Some(l) = self.free_big_objects[sc].pop_front() {
            let span = l.resolve();
            self.allocated_big_objects[sc as usize].push_back(l);

            Some(span.start.value() as *mut u8)
        } else {
            None
        }
    }
    fn dealloc_big(&mut self, ptr: *mut u8, layout: Layout) -> Result<(), ErrorCode> {
        let sc = Self::pick_big_object_size(layout.size());
        if sc as usize >= self.allocated_big_objects.len() {
            Err(EINVAL)
        } else {
            let mut link = Link::<Span>::none();
            let layout_npages = sc.to_npage();
            for l in self.allocated_big_objects[sc as usize].iter() {
                let span = l.resolve();

                if span.start.value() == ptr as usize && span.num_pages == layout_npages {
                    link = l;
                    break;
                }
            }

            if link.is_none() {
                Err(EINVAL)
            } else {
                let span = link.resolve();
                unsafe {
                    clear_memory_range(
                        span.start.value(),
                        span.start.value() + sc.to_npage() * 4096,
                    );
                }
                self.allocated_big_objects[sc as usize].remove(link);
                if self.free_big_objects[sc as usize].len() > FREE_BIG_OBJECT_LIST_LENGTH_LIMIT {
                    // should return to the system
                    todo!()
                } else {
                    self.free_big_objects[sc as usize].push_front(link);
                    Ok(())
                }
            }
        }
    }
}

impl HeapFrontend {
    pub const HEAP_FRONTEND_MAX_SIZE_EXCLUSIVE: usize = 2049;
    pub fn pick_size_class(v: usize) -> Log2SizeClass {
        match v {
            0..9 => Log2SizeClass::SZ_8,
            9..17 => Log2SizeClass::SZ_16,
            17..33 => Log2SizeClass::SZ_32,
            33..65 => Log2SizeClass::SZ_64,
            65..129 => Log2SizeClass::SZ_128,
            129..257 => Log2SizeClass::SZ_256,
            257..513 => Log2SizeClass::SZ_512,
            513..1025 => Log2SizeClass::SZ_1024,
            1025..Self::HEAP_FRONTEND_MAX_SIZE_EXCLUSIVE => Log2SizeClass::SZ_2048,
            _ => Log2SizeClass::Undefined,
        }
    }
    pub const fn new() -> Self {
        Self {
            sc_allocator: [
                Spinlock::new(SizeClassAllocator::new(Log2SizeClass::SZ_8)),
                Spinlock::new(SizeClassAllocator::new(Log2SizeClass::SZ_16)),
                Spinlock::new(SizeClassAllocator::new(Log2SizeClass::SZ_32)),
                Spinlock::new(SizeClassAllocator::new(Log2SizeClass::SZ_64)),
                Spinlock::new(SizeClassAllocator::new(Log2SizeClass::SZ_128)),
                Spinlock::new(SizeClassAllocator::new(Log2SizeClass::SZ_256)),
                Spinlock::new(SizeClassAllocator::new(Log2SizeClass::SZ_512)),
                Spinlock::new(SizeClassAllocator::new(Log2SizeClass::SZ_1024)),
                Spinlock::new(SizeClassAllocator::new(Log2SizeClass::SZ_2048)),
            ],
        }
    }

    pub fn refill(&self, start: usize, layout: Layout) -> Result<(), ErrorCode> {
        let sc = Self::pick_size_class(layout.size());
        if sc == Log2SizeClass::Undefined {
            todo!()
        }
        self.sc_allocator[sc as usize - 3].lock().refill(start)
    }

    pub fn alloc(&self, layout: Layout) -> Option<*mut u8> {
        let sc = Self::pick_size_class(layout.size());
        if (sc as usize - 3) >= self.sc_allocator.len() {
            return None;
        }
        self.sc_allocator[sc as usize - 3].lock().alloc(layout)
    }
    fn dealloc(&self, ptr: *mut u8, layout: Layout) -> Result<Option<VirtualAddress>, ErrorCode> {
        let sc = Self::pick_size_class(layout.size()) as usize;
        if (sc - 3) >= self.sc_allocator.len() {
            return Err(ESUPPORTED);
        }
        Ok(self.sc_allocator[sc - 3].lock().dealloc(ptr, layout))
    }
}

impl UnsafeHeapAllocator {
    fn new() -> Self {
        Self {
            backend: Spinlock::new(HeapBackend::new()),
            frontend: HeapFrontend::new(),
        }
    }

    fn init(&self, va: VaRange) -> Result<(), ErrorCode> {
        let mut va_copy = va;
        self.backend.lock().insert_4K(va)
    }

    fn alloc(&self, layout: Layout) -> Option<*mut u8> {
        if layout.size() >= HeapFrontend::HEAP_FRONTEND_MAX_SIZE_EXCLUSIVE {
            if let Some(p) = self.backend.lock().alloc_big(layout) {
                return Some(p);
            }

            let npages = HeapBackend::pick_big_object_size(layout.size()).to_npage();

            let mapped = super::MMU
                .get()
                .unwrap()
                .kzalloc(npages, RWNORMAL, HIGHER_PAGE)
                .unwrap();

            self.backend.lock().refill_big(mapped).unwrap();
            self.backend.lock().alloc_big(layout)
        } else {
            if let Some(p) = self.frontend.alloc(layout) {
                return Some(p);
            }

            let refilled = if let Some(p) = self.backend.lock().allocate_4K_free() {
                self.frontend.refill(p, layout).unwrap();
                true
            } else {
                false
            };
            if !refilled {
                if layout.size() == ADDRESS_RANGE_NODE_SIZE {
                    todo!()
                }

                let mapped = super::MMU
                    .get()
                    .unwrap()
                    .kzalloc(10, RWNORMAL, HIGHER_PAGE)
                    .unwrap();

                self.backend.lock().insert_4K(mapped.va).unwrap();
                let p = self.backend.lock().allocate_4K_free().unwrap();

                self.frontend.refill(p, layout).unwrap();
            }
            self.frontend.alloc(layout)
        }
    }
    fn dealloc(&self, ptr: *mut u8, layout: Layout) -> Result<(), ErrorCode> {
        if layout.size() >= HeapFrontend::HEAP_FRONTEND_MAX_SIZE_EXCLUSIVE {
            return self.backend.lock().dealloc_big(ptr, layout);
        }

        let v = self.frontend.dealloc(ptr, layout)?;

        if let Some(va) = v {
            // first try to return to the backend
            // it its full, let MMU unmap it.
            self.backend
                .lock()
                .insert_4K(VaRange::new(va, va + VirtualAddress::_4K))
                .or_else(|_| MMU.get().unwrap().unmap(va))
                .unwrap();
        }
        Ok(())
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
    allocator: UnsafeHeapAllocator,
}

impl HeapAllocator {
    fn new() -> Self {
        Self {
            allocator: UnsafeHeapAllocator::new(),
        }
    }

    fn init(&self, va: VaRange) -> Result<(), ErrorCode> {
        self.allocator.init(va)
    }

    fn alloc(&self, layout: Layout) -> *mut u8 {
        if let Some(p) = self.allocator.alloc(layout) {
            p
        } else {
            core::ptr::null::<u8>() as *mut u8
        }
    }

    fn dealloc(&self, ptr: *mut u8, layout: Layout) -> Result<(), ErrorCode> {
        self.allocator.dealloc(ptr, layout)
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
        HEAP_ALLOCATOR.get().unwrap().dealloc(ptr, layout).unwrap();
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
        let span_size = core::mem::size_of::<Span>();

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
