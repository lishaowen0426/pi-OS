extern crate alloc;
use super::address::*;
use crate::{errno::*, BootInfo};
use alloc::boxed::Box;
use intrusive_collections::{intrusive_adapter, LinkedList, LinkedListLink};
use spin::{mutex::SpinMutex, once::Once};

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
    free_2m: LinkedList<RangeListAdaptor<PaRange>>,
}

impl UnsafeFrameAllocator {
    pub fn new(boot_info: &BootInfo) -> Self {
        Self {
            free_4k: LinkedList::new(RangeListAdaptor::new()),
            free_2m: LinkedList::new(RangeListAdaptor::new()),
        }
    }
    pub fn allocate_4K(&self) -> Option<PhysicalAddress> {
        None
    }
    pub fn allocate_2M(&self) -> Option<PhysicalAddress> {
        None
    }
}

pub struct FrameAllocator {
    allocator: SpinMutex<UnsafeFrameAllocator>,
}

impl FrameAllocator {
    pub fn new(boot_info: &BootInfo) -> Self {
        Self {
            allocator: SpinMutex::new(UnsafeFrameAllocator::new(boot_info)),
        }
    }

    pub fn allocate_4K(&self) -> Option<PhysicalAddress> {
        self.allocator.lock().allocate_4K()
    }
    pub fn allocate_2M(&self) -> Option<PhysicalAddress> {
        self.allocator.lock().allocate_2M()
    }
}

pub fn init(boot_info: &BootInfo) -> Result<(), ErrorCode> {
    FRAME_ALLOCATOR.call_once(|| FrameAllocator::new(boot_info));
    Ok(())
}

pub static FRAME_ALLOCATOR: Once<FrameAllocator> = Once::new();
