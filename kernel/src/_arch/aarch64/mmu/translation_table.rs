use super::{address::*, cache::*, config, frame_allocator::*, translation_entry::*, BlockSize};
use crate::{errno::*, println, unsafe_println};
use aarch64_cpu::{
    asm::barrier,
    registers::{TTBR0_EL1, TTBR1_EL1},
};
use core::ops::Index;
use tock_registers::interfaces::ReadWriteable;

extern "C" {
    static __code_start: u8;
    static __code_end_exclusive: u8;
    static __bss_start: u8;
    static __bss_end_exclusive: u8;
    static __data_start: u8;
    static __data_end_exclusive: u8;
    static __page_table_end_exclusive: u8;
    static l1_lower_page_table: u8;
}
#[repr(transparent)]
pub struct UnsafeTranslationTable<L> {
    base: *mut TranslationTableEntry<L>,
}

unsafe impl<L> Send for UnsafeTranslationTable<L> {}
unsafe impl<L> Sync for UnsafeTranslationTable<L> {}

impl<L: TranslationTableLevel> Index<usize> for UnsafeTranslationTable<L> {
    type Output = TranslationTableEntry<L>;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { self.base.offset(index as isize).as_ref().unwrap() }
    }
}

impl<L: TranslationTableLevel> UnsafeTranslationTable<L> {
    pub fn new(base: *mut TranslationTableEntry<L>) -> Self {
        Self { base }
    }

    pub fn as_mut_ptr(&self) -> *mut TranslationTableEntry<L> {
        self.base
    }

    pub fn as_address(&self) -> usize {
        self.base.addr()
    }

    pub fn set_entry(&self, idx: usize, entry: TranslationTableEntry<L>) -> Result<(), ErrorCode> {
        if idx >= config::ENTRIES_PER_TABLE {
            Err(EBOUND)
        } else {
            unsafe {
                self.base.offset(idx.try_into().unwrap()).write(entry);
            }
            Ok(())
        }
    }
}

impl UnsafeTranslationTable<Level1> {
    pub fn translate(&self, va: VirtualAddress) -> Option<PhysicalAddress> {
        let l1_entry = self[va.level1()].get();
        println!("l2_table_address : {:#018x}", Self::l2_table_address(va));
        println!("l1_entry {}", l1_entry);

        match l1_entry {
            Descriptor::TableEntry(_) => {
                let l2_table = UnsafeTranslationTable::<Level2>::new(
                    Self::l2_table_address(va) as *mut L2Entry
                );
                let l2_entry = l2_table[va.level2()].get();
                match l2_entry {
                    Descriptor::TableEntry(_) => {
                        let l3_table = UnsafeTranslationTable::<Level3>::new(
                            Self::l3_table_address(va) as *mut L3Entry,
                        );
                        let l3_entry = l3_table[va.level3()].get();
                        match l3_entry {
                            Descriptor::PageEntry(_) => {
                                let pa = l3_entry.get_address().unwrap();
                                Some(pa.set_offset(va.offset()))
                            }
                            _ => None,
                        }
                    }
                    Descriptor::L2BlockEntry(_) => {
                        let pa = l2_entry.get_address().unwrap();
                        Some(pa.set_offset(va.offset()))
                    }
                    _ => None,
                }
            }
            Descriptor::L1BlockEntry(_) => {
                let pa = l1_entry.get_address().unwrap();
                Some(pa.set_offset(va.offset()))
            }
            _ => None,
        }
    }
    pub fn map(
        &self,
        va: VirtualAddress,
        pa: PhysicalAddress,
        mt: &MemoryType,
        sz: BlockSize,
        frame_allocator: &mut dyn FrameAllocator,
    ) -> Result<(), ErrorCode> {
        match sz {
            BlockSize::_4K => self.map_4K(va, pa, mt, frame_allocator),
            BlockSize::_2M => self.map_2M(va, pa, mt, frame_allocator),
            BlockSize::_1G => self.map_1G(va, pa, mt, frame_allocator),
        }
    }

    fn l2_table_address(va: VirtualAddress) -> usize {
        let mut res: usize = 0;
        if va.is_higher() {
            res = config::KERNEL_OFFSET;
        }
        let l1_index = va.level1();
        res | (config::RECURSIVE_L1_INDEX << config::L1_INDEX_SHIFT)
            | (config::RECURSIVE_L1_INDEX << config::L2_INDEX_SHIFT)
            | (l1_index << config::L3_INDEX_SHIFT)
    }
    fn l3_table_address(va: VirtualAddress) -> usize {
        let mut res: usize = 0;
        if va.is_higher() {
            res = config::KERNEL_OFFSET;
        }
        let l1_index = va.level1();
        let l2_index = va.level2();
        res | (config::RECURSIVE_L1_INDEX << config::L1_INDEX_SHIFT)
            | (l1_index << config::L2_INDEX_SHIFT)
            | (l2_index << config::L3_INDEX_SHIFT)
    }
    fn map_4K(
        &self,
        va: VirtualAddress,
        pa: PhysicalAddress,
        mt: &MemoryType,
        frame_allocator: &mut dyn FrameAllocator,
    ) -> Result<(), ErrorCode> {
        if !va.is_4K_aligned() || !pa.is_4K_aligned() {
            return Err(EALIGN);
        }

        let mut l1_entry = self[va.level1()].get();
        let l2_table = match l1_entry {
            Descriptor::INVALID => {
                l1_entry = l1_entry.set_table()?;
                l1_entry.set_attributes(TABLE_PAGE)?;
                let allocated_frame_addr: PhysicalAddress =
                    frame_allocator.frame_alloc().ok_or(EFRAME)?.to_address();
                l1_entry.set_address(allocated_frame_addr)?;
                self.set_entry(va.level1(), TranslationTableEntry::from(l1_entry))?;

                Option::Some(UnsafeTranslationTable::<Level2>::new(
                    Self::l2_table_address(va) as *mut L2Entry,
                ))
            }
            Descriptor::TableEntry(_) => Option::Some(UnsafeTranslationTable::<Level2>::new(
                Self::l2_table_address(va) as *mut L2Entry,
            )),
            _ => None,
        }
        .ok_or(ETYPE)?;

        let mut l2_entry = l2_table[va.level2()].get();
        let l3_table = match l2_entry {
            Descriptor::INVALID => {
                l2_entry = l2_entry.set_table()?;
                l2_entry.set_attributes(TABLE_PAGE)?;
                let allocated_frame_addr: PhysicalAddress =
                    frame_allocator.frame_alloc().ok_or(EFRAME)?.to_address();
                l2_entry.set_address(allocated_frame_addr)?;
                l2_table.set_entry(va.level2(), TranslationTableEntry::from(l2_entry))?;

                Option::Some(UnsafeTranslationTable::<Level3>::new(
                    Self::l3_table_address(va) as *mut L3Entry,
                ))
            }
            Descriptor::TableEntry(_) => Option::Some(UnsafeTranslationTable::<Level3>::new(
                Self::l3_table_address(va) as *mut L3Entry,
            )),
            _ => None,
        }
        .ok_or(ETYPE)?;

        let mut l3_entry: Descriptor = l3_table[va.level3()].get();
        match l3_entry {
            Descriptor::INVALID => {
                l3_entry = l3_entry.set_page()?;
                l3_entry.set_attributes(mt)?;
                l3_entry.set_address(pa)?;
                l3_table.set_entry(va.level3(), TranslationTableEntry::from(l3_entry))?;
            }
            _ => return Err(ETYPE),
        };
        Ok(())
    }
    fn map_2M(
        &self,
        va: VirtualAddress,
        pa: PhysicalAddress,
        mt: &MemoryType,
        frame_allocator: &mut dyn FrameAllocator,
    ) -> Result<(), ErrorCode> {
        if !va.is_2M_aligned() || !pa.is_2M_aligned() {
            return Err(EALIGN);
        }

        let mut l1_entry = self[va.level1()].get();
        let l2_table = match l1_entry {
            Descriptor::INVALID => {
                l1_entry = l1_entry.set_table()?;
                l1_entry.set_attributes(TABLE_PAGE)?;
                let allocated_frame_addr: PhysicalAddress =
                    frame_allocator.frame_alloc().ok_or(EFRAME)?.to_address();
                l1_entry.set_address(allocated_frame_addr)?;
                self.set_entry(va.level1(), TranslationTableEntry::from(l1_entry))?;

                Option::Some(UnsafeTranslationTable::<Level2>::new(
                    Self::l2_table_address(va) as *mut L2Entry,
                ))
            }
            Descriptor::TableEntry(_) => Option::Some(UnsafeTranslationTable::<Level2>::new(
                Self::l2_table_address(va) as *mut L2Entry,
            )),
            _ => None,
        }
        .ok_or(ETYPE)?;

        let mut l2_entry = l2_table[va.level2()].get();
        match l2_entry {
            Descriptor::INVALID => {
                l2_entry = l2_entry.set_l2_block()?;
                l2_entry.set_attributes(mt)?;
                l2_entry.set_address(pa)?;
                l2_table.set_entry(va.level2(), TranslationTableEntry::from(l2_entry))?;
            }
            _ => return Err(ETYPE),
        };
        Ok(())
    }
    fn map_1G(
        &self,
        va: VirtualAddress,
        pa: PhysicalAddress,
        mt: &MemoryType,
        _frame_allocator: &mut dyn FrameAllocator,
    ) -> Result<(), ErrorCode> {
        if !va.is_1G_aligned() || !pa.is_1G_aligned() {
            return Err(EALIGN);
        }

        let mut l1_entry = self[va.level1()].get();
        match l1_entry {
            Descriptor::INVALID => {
                l1_entry = l1_entry.set_l1_block()?;
                l1_entry.set_attributes(mt)?;
                l1_entry.set_address(pa)?;
                self.set_entry(va.level1(), TranslationTableEntry::from(l1_entry))?;
            }
            _ => return Err(ETYPE),
        };

        Ok(())
    }
}

fn get_ttbr0() -> usize {
    TTBR0_EL1.get_baddr() as usize
}
pub fn set_ttbr0(pa: PhysicalAddress, asid: u8) {
    unsafe_println!("Set up TTBR0_EL1 with pa {}, ASID = {}", pa, asid);
    TTBR0_EL1.modify(TTBR0_EL1::ASID.val(asid as u64));
    TTBR0_EL1.set_baddr(pa.value() as u64);
    barrier::isb(barrier::SY);
    A64TLB::invalidate_all();
}
pub fn set_ttbr1(pa: PhysicalAddress, asid: u8) {
    unsafe_println!("Set up TTBR1_EL1 with pa {}, ASID = {}", pa, asid);
    TTBR1_EL1.modify(TTBR1_EL1::ASID.val(asid as u64));
    TTBR1_EL1.set_baddr(pa.value() as u64);
    barrier::isb(barrier::SY);
    A64TLB::invalidate_all();
}

#[cfg(test)]
#[allow(dead_code, unused_variables, unused_imports)]
mod tests {
    use super::*;
    use test_macros::kernel_test;

    #[kernel_test]
    fn test_translation_table() {}
}
