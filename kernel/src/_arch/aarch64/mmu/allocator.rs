extern crate alloc;

use super::{address::*, heap::*};
use crate::{
    errno::*,
    memory::{BlockSize, MemoryRegion, BLOCK_2M, BLOCK_4K},
    println, BootInfo,
};
use aarch64_cpu::registers::*;
use alloc::boxed::Box;
use core::fmt;
use intrusive_collections::{
    intrusive_adapter, Bound, KeyAdapter, LinkedList, LinkedListLink, RBTree, RBTreeLink,
};
use spin::{mutex::SpinMutex, once::Once};
use tock_registers::interfaces::{Readable, Writeable};

const HUGE_PAGE_RATIO: usize = 40; // in percentage
const LEN_4K: usize = 0x1000;
const LEN_2M: usize = 0x200000;

#[derive(Debug)]
struct AddressRangeNode<T> {
    link: RBTreeLink,
    range: T,
}

impl<T> AddressRangeNode<T>
where
    T: AddressRange + Clone,
{
    fn new(range: T) -> Self {
        Self {
            range,
            link: RBTreeLink::new(),
        }
    }
    fn range(&self) -> &T {
        &self.range
    }
    fn range_copy(&self) -> T {
        self.range.clone()
    }
}

intrusive_adapter!(AddressRangeAdaptor<T> = Box<AddressRangeNode<T>> : AddressRangeNode<T> {link:RBTreeLink} where T:AddressRange+Clone);
impl<'a, T> KeyAdapter<'a> for AddressRangeAdaptor<T>
where
    T: AddressRange + Clone,
{
    type Key = <T as super::address::AddressRange>::Address;
    fn get_key(&self, value: &'a AddressRangeNode<T>) -> Self::Key {
        value.range.end()
    }
}

struct UnsafeFrameAllocator {
    pma: RBTree<AddressRangeAdaptor<PaRange>>,
}

impl fmt::Display for UnsafeFrameAllocator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for v in self.pma.iter() {
            writeln!(f, "{}", v.range())?;
        }
        Ok(())
    }
}

impl UnsafeFrameAllocator {
    pub fn new(pa_range: PaRange) -> Self {
        let mut frame_allocator = Self {
            pma: RBTree::new(AddressRangeAdaptor::new()),
        };
        frame_allocator.init(pa_range);
        frame_allocator
    }

    fn init(&mut self, pa_range: PaRange) {
        self.pma.insert(Box::new(AddressRangeNode::new(pa_range)));
    }
    pub fn allocate_4K(&mut self) -> Result<PaRange, ErrorCode> {
        for n in self.pma.iter() {
            if n.range().len().value() >= LEN_4K {
                let mut range = n.range_copy();
                let end = range.end();
                let bound = Bound::Included(&end);
                let mut cursor = self.pma.upper_bound_mut(bound);
                if *(cursor.get().unwrap().range()) != range {
                    return Err(EFRAME);
                }
                let pa = range.pop_4K_front().unwrap();
                cursor
                    .replace_with(Box::new(AddressRangeNode::new(range)))
                    .unwrap();
                return Ok(pa.to_4K_range());
            }
        }

        Err(EFRAME)
    }
    pub fn allocate_2M(&mut self) -> Result<PaRange, ErrorCode> {
        for n in self.pma.iter() {
            if n.range().len().value() >= LEN_2M {
                let mut range = n.range_copy();
                let end = range.end();
                let bound = Bound::Included(&end);
                let mut cursor = self.pma.upper_bound_mut(bound);
                if *(cursor.get().unwrap().range()) != range {
                    return Err(EFRAME);
                }
                let pa = range.pop_2M_front().unwrap();
                cursor
                    .replace_with(Box::new(AddressRangeNode::new(range)))
                    .unwrap();
                return Ok(pa.to_2M_range());
            }
        }

        Err(EFRAME)
    }

    pub fn free_range(&mut self, pa_range: PaRange) {
        let mut to_free = pa_range;
        let end = to_free.end();
        let bound = Bound::Excluded(&end);

        let mut upper_cursor = self.pma.upper_bound_mut(bound);
        if !upper_cursor.is_null() {
            let upper = upper_cursor.get().unwrap().range();
            if to_free.merge(&upper).is_ok() {
                upper_cursor.remove().unwrap();
            }
        }

        let end = to_free.end();
        let bound = Bound::Excluded(&end);

        let mut lower_cursor = self.pma.lower_bound_mut(bound);
        if !lower_cursor.is_null() {
            let lower = lower_cursor.get().unwrap().range();
            if to_free.merge(&lower).is_ok() {
                lower_cursor.remove().unwrap();
            }
        }

        self.pma.insert(Box::new(AddressRangeNode::new(to_free)));
    }
}

pub struct FrameAllocator {
    allocator: SpinMutex<UnsafeFrameAllocator>,
}

impl FrameAllocator {
    pub fn new(boot_info: &BootInfo) -> Self {
        Self {
            allocator: SpinMutex::new(UnsafeFrameAllocator::new(boot_info.free_frame)),
        }
    }

    fn allocate_4K(&self) -> Result<PaRange, ErrorCode> {
        self.allocator.lock().allocate_4K()
    }
    fn allocate_2M(&self) -> Result<PaRange, ErrorCode> {
        self.allocator.lock().allocate_2M()
    }

    pub fn allocate(&self, sz: &BlockSize) -> Result<PaRange, ErrorCode> {
        match *sz {
            BlockSize::_4K => self.allocate_4K(),
            BlockSize::_2M => self.allocate_2M(),
            _ => Err(ESUPPORTED),
        }
    }

    pub fn free_range(&self, pa_range: PaRange) {
        self.allocator.lock().free_range(pa_range)
    }
}

pub static FRAME_ALLOCATOR: Once<FrameAllocator> = Once::new();

struct UnsafePageAllocator {
    lower_vma: RBTree<AddressRangeAdaptor<VaRange>>,

    higher_vma_start: VirtualAddress,
    higher_vma: RBTree<AddressRangeAdaptor<VaRange>>,
}

impl UnsafePageAllocator {
    pub fn new(boot_info: &BootInfo) -> Self {
        let mut page_allocator = Self {
            lower_vma: RBTree::new(AddressRangeAdaptor::new()),
            higher_vma_start: boot_info.higher_free_page.start(),
            higher_vma: RBTree::new(AddressRangeAdaptor::new()),
        };
        page_allocator.init(boot_info);
        page_allocator
    }

    fn init(&mut self, boot_info: &BootInfo) {
        self.lower_vma
            .insert(Box::new(AddressRangeNode::new(boot_info.lower_free_page)));
        self.higher_vma
            .insert(Box::new(AddressRangeNode::new(boot_info.higher_free_page)));
    }
    fn _allocate_4K(vma: &mut RBTree<AddressRangeAdaptor<VaRange>>) -> Result<VaRange, ErrorCode> {
        for n in vma.iter() {
            if n.range().len().value() >= LEN_4K {
                let mut range = n.range_copy();
                let end = range.end();
                let bound = Bound::Included(&end);
                let mut cursor = vma.upper_bound_mut(bound);
                if *(cursor.get().unwrap().range()) != range {
                    return Err(EPAGE);
                }
                let va = range.pop_4K_front().unwrap();
                cursor
                    .replace_with(Box::new(AddressRangeNode::new(range)))
                    .unwrap();
                return Ok(va.to_4K_range());
            }
        }

        Err(EPAGE)
    }
    fn _allocate_2M(vma: &mut RBTree<AddressRangeAdaptor<VaRange>>) -> Result<VaRange, ErrorCode> {
        for n in vma.iter() {
            if n.range().len().value() >= LEN_2M {
                let mut range = n.range_copy();
                let end = range.end();
                let bound = Bound::Included(&end);
                let mut cursor = vma.upper_bound_mut(bound);
                if *(cursor.get().unwrap().range()) != range {
                    return Err(EPAGE);
                }
                let va = range.pop_2M_front().unwrap();
                cursor
                    .replace_with(Box::new(AddressRangeNode::new(range)))
                    .unwrap();
                return Ok(va.to_2M_range());
            }
        }

        Err(EPAGE)
    }
    fn _free_range(vma: &mut RBTree<AddressRangeAdaptor<VaRange>>, va_range: VaRange) {
        let mut to_free = va_range;
        let end = to_free.end();
        let bound = Bound::Excluded(&end);

        let mut upper_cursor = vma.upper_bound_mut(bound);
        if !upper_cursor.is_null() {
            let upper = upper_cursor.get().unwrap().range();
            if to_free.merge(&upper).is_ok() {
                upper_cursor.remove().unwrap();
            }
        }

        let end = to_free.end();
        let bound = Bound::Excluded(&end);

        let mut lower_cursor = vma.lower_bound_mut(bound);
        if !lower_cursor.is_null() {
            let lower = lower_cursor.get().unwrap().range();
            if to_free.merge(&lower).is_ok() {
                lower_cursor.remove().unwrap();
            }
        }

        vma.insert(Box::new(AddressRangeNode::new(to_free)));
    }
    fn allocate_4K(&mut self, region: &MemoryRegion) -> Result<VaRange, ErrorCode> {
        match *region {
            MemoryRegion::Lower => Self::_allocate_4K(&mut self.lower_vma),
            MemoryRegion::Higher => Self::_allocate_4K(&mut self.higher_vma),
        }
    }
    fn allocate_2M(&mut self, region: &MemoryRegion) -> Result<VaRange, ErrorCode> {
        match *region {
            MemoryRegion::Lower => Self::_allocate_2M(&mut self.lower_vma),
            MemoryRegion::Higher => Self::_allocate_2M(&mut self.higher_vma),
        }
    }
    pub fn free_range(&mut self, va_range: VaRange) {
        if va_range.start() >= self.higher_vma_start {
            Self::_free_range(&mut self.higher_vma, va_range)
        } else {
            Self::_free_range(&mut self.lower_vma, va_range)
        }
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

    fn allocate_4K(&self, region: &MemoryRegion) -> Result<VaRange, ErrorCode> {
        self.allocator.lock().allocate_4K(region)
    }
    fn allocate_2M(&self, region: &MemoryRegion) -> Result<VaRange, ErrorCode> {
        self.allocator.lock().allocate_2M(region)
    }

    pub fn allocate(&self, sz: &BlockSize, region: &MemoryRegion) -> Result<VaRange, ErrorCode> {
        match *sz {
            BlockSize::_4K => self.allocate_4K(region),
            BlockSize::_2M => self.allocate_2M(region),
            _ => Err(ESUPPORTED),
        }
    }
    pub fn free_range(&self, va_range: VaRange) {
        self.allocator.lock().free_range(va_range)
    }
}

pub static PAGE_ALLOCATOR: Once<PageAllocator> = Once::new();

pub fn init(boot_info: &BootInfo) -> Result<(), ErrorCode> {
    FRAME_ALLOCATOR.call_once(|| FrameAllocator::new(boot_info));
    PAGE_ALLOCATOR.call_once(|| PageAllocator::new(boot_info));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use intrusive_collections::PointerOps;
    use test_macros::kernel_test;

    #[derive(Debug)]
    struct VMA {
        start: usize,
        end: usize,
    }
    #[derive(Debug)]
    struct Node {
        vma: VMA,
        link: RBTreeLink,
    }

    intrusive_adapter!(RBAdaptor = Box<Node> :Node {link: RBTreeLink});

    impl VMA {
        pub fn new(start: usize, end: usize) -> Self {
            Self { start, end }
        }
    }
    impl Node {
        pub fn new(start: usize, end: usize) -> Self {
            Self {
                vma: VMA::new(start, end),
                link: RBTreeLink::new(),
            }
        }
    }

    impl<'a> KeyAdapter<'a> for RBAdaptor {
        type Key = usize;
        fn get_key(&self, value: &'a Node) -> Self::Key {
            value.vma.end
        }
    }
    #[kernel_test]
    fn test_rbtree() {
        {}
    }
}
