extern crate alloc;
use super::{address::*, heap::*};
use crate::{errno::*, BootInfo};
use alloc::boxed::Box;
use intrusive_collections::{intrusive_adapter, LinkedList, LinkedListLink};
use spin::{mutex::SpinMutex, once::Once};

const HUGE_PAGE_RATIO: usize = 40; // in percentage

extern "C" {
    fn clear_memory_range(start: usize, end_exclusive: usize);
}

struct AddressRangeNode<T: AddressRange> {
    link: LinkedListLink,
    range: T,
}

intrusive_adapter!(RangeListAdaptor<T> = Box<AddressRangeNode<T>> : AddressRangeNode<T> {link: LinkedListLink} where T:AddressRange);

struct UnsafeFrameAllocator {
    free_4k: LinkedList<RangeListAdaptor<PaRange>>,
    free_2m: LinkedList<RangeListAdaptor<PaRange>>, // each PaRange is a multiple of 4k/2m
}

impl UnsafeFrameAllocator {
    pub fn new() -> Self {
        Self {
            free_4k: LinkedList::new(RangeListAdaptor::new()),
            free_2m: LinkedList::new(RangeListAdaptor::new()),
        }
    }

    pub fn fill(&mut self, boot_info: &BootInfo) -> (usize, usize) {
        let (mut huge_range, mut small_range) = boot_info.free_frame.split(HUGE_PAGE_RATIO);
        small_range.align_to_4K();
        huge_range.align_to_2M();
        self.free_4k.push_back(Box::new(AddressRangeNode {
            link: LinkedListLink::new(),
            range: small_range,
        }));
        self.free_2m.push_back(Box::new(AddressRangeNode {
            link: LinkedListLink::new(),
            range: huge_range,
        }));
        (small_range.size_in_bytes(), huge_range.size_in_bytes())
    }
    pub fn allocate_4K(&mut self) -> Option<PhysicalAddress> {
        None
    }
    pub fn allocate_2M(&mut self) -> Option<PhysicalAddress> {
        None
    }
}

pub struct FrameAllocator {
    allocator: SpinMutex<UnsafeFrameAllocator>,
}

impl FrameAllocator {
    pub fn new() -> Self {
        Self {
            allocator: SpinMutex::new(UnsafeFrameAllocator::new()),
        }
    }
    pub fn fill(&self, boot_info: &BootInfo) -> (usize, usize) {
        self.allocator.lock().fill(boot_info)
    }

    pub fn allocate_4K(&self) -> Option<PhysicalAddress> {
        self.allocator.lock().allocate_4K()
    }
    pub fn allocate_2M(&self) -> Option<PhysicalAddress> {
        self.allocator.lock().allocate_2M()
    }
}

pub fn init(boot_info: &BootInfo) -> Result<(), ErrorCode> {
    FRAME_ALLOCATOR.call_once(|| FrameAllocator::new());
    Ok(())
}

pub static FRAME_ALLOCATOR: Once<FrameAllocator> = Once::new();
