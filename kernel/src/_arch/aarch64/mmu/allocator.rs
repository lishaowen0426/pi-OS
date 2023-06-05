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
    pub fn new(boot_info: &BootInfo) -> Self {
        let mut frame_allocator = Self {
            free_4k: LinkedList::new(RangeListAdaptor::new()),
            free_2m: LinkedList::new(RangeListAdaptor::new()),
        };
        frame_allocator.init(boot_info);
        frame_allocator
    }

    fn init(&mut self, boot_info: &BootInfo) {
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

pub static FRAME_ALLOCATOR: Once<FrameAllocator> = Once::new();

struct UnsafePageAllocator {
    lower_free_4k: LinkedList<RangeListAdaptor<VaRange>>,
    lower_free_2m: LinkedList<RangeListAdaptor<VaRange>>, // each PaRange is a multiple of 4k/2m
    higher_free_4k: LinkedList<RangeListAdaptor<VaRange>>,
    higher_free_2m: LinkedList<RangeListAdaptor<VaRange>>, // each PaRange is a multiple of 4k/2m
}

impl UnsafePageAllocator {
    pub fn new(boot_info: &BootInfo) -> Self {
        let mut page_allocator = Self {
            lower_free_4k: LinkedList::new(RangeListAdaptor::new()),
            lower_free_2m: LinkedList::new(RangeListAdaptor::new()),
            higher_free_4k: LinkedList::new(RangeListAdaptor::new()),
            higher_free_2m: LinkedList::new(RangeListAdaptor::new()),
        };
        page_allocator.init(boot_info);
        page_allocator
    }

    fn init(&mut self, boot_info: &BootInfo) {
        let (mut lower_huge_range, mut lower_small_range) =
            boot_info.lower_free_page.split(HUGE_PAGE_RATIO);
        lower_small_range.align_to_4K();
        lower_huge_range.align_to_2M();
        self.lower_free_4k.push_back(Box::new(AddressRangeNode {
            link: LinkedListLink::new(),
            range: lower_small_range,
        }));
        self.lower_free_2m.push_back(Box::new(AddressRangeNode {
            link: LinkedListLink::new(),
            range: lower_huge_range,
        }));
        let (mut higher_huge_range, mut higher_small_range) =
            boot_info.higher_free_page.split(HUGE_PAGE_RATIO);
        higher_small_range.align_to_4K();
        higher_huge_range.align_to_2M();
        self.higher_free_4k.push_back(Box::new(AddressRangeNode {
            link: LinkedListLink::new(),
            range: higher_small_range,
        }));
        self.higher_free_2m.push_back(Box::new(AddressRangeNode {
            link: LinkedListLink::new(),
            range: higher_huge_range,
        }));
    }
    pub fn allocate_4K(&mut self) -> Option<VirtualAddress> {
        None
    }
    pub fn allocate_2M(&mut self) -> Option<VirtualAddress> {
        None
    }
}

pub struct PageAllocator {
    allocator: SpinMutex<UnsafePageAllocator>,
}

impl PageAllocator {
    pub fn new(boot_info: &BootInfo) -> Self {
        Self {
            allocator: SpinMutex::new(UnsafePageAllocator::new(boot_info)),
        }
    }

    pub fn allocate_4K(&self) -> Option<VirtualAddress> {
        self.allocator.lock().allocate_4K()
    }
    pub fn allocate_2M(&self) -> Option<VirtualAddress> {
        self.allocator.lock().allocate_2M()
    }
}

pub static PAGE_ALLOCATOR: Once<PageAllocator> = Once::new();

pub fn init(boot_info: &BootInfo) -> Result<(), ErrorCode> {
    FRAME_ALLOCATOR.call_once(|| FrameAllocator::new(boot_info));
    PAGE_ALLOCATOR.call_once(|| PageAllocator::new(boot_info));
    Ok(())
}
