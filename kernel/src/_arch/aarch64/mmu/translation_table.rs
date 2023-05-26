use super::{address::*, cache::*, config, frame_allocator::*, translation_entry::*, BlockSize};
use crate::{bsp::mmio, errno::*, println, unsafe_print, unsafe_println};
use aarch64_cpu::{
    asm::barrier,
    registers::{TTBR0_EL1, TTBR1_EL1},
};
use core::ops::Index;
use tock_registers::interfaces::ReadWriteable;

extern "C" {
    static __boot_core_stack_end_exclusive: u8;
    static __code_start: u8;
    static __code_end_exclusive: u8;
    static __bss_start: u8;
    static __bss_end_exclusive: u8;
    static __data_start: u8;
    static __data_end_exclusive: u8;
    static __l1_page_table_start: u8;
    static __page_table_end_exclusive: u8;
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
        println!("l2_table_address : {:#066b}", Self::l2_table_address(va));
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
        let l1_index = va.level1();
        (config::RECURSIVE_L1_INDEX << config::L1_INDEX_SHIFT)
            | (config::RECURSIVE_L1_INDEX << config::L2_INDEX_SHIFT)
            | (l1_index << config::L3_INDEX_SHIFT)
    }
    fn l3_table_address(va: VirtualAddress) -> usize {
        let l1_index = va.level1();
        let l2_index = va.level2();
        (config::RECURSIVE_L1_INDEX << config::L1_INDEX_SHIFT)
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

#[allow(unused_variables)]
pub fn set_up_init_mapping() -> Result<(), ErrorCode> {
    let boot_core_stack_end_exclusive_addr: usize;
    let code_start_addr: usize;
    let code_end_addr: usize;
    let data_start_addr: usize;
    let data_end_addr: usize;
    let bss_start_addr: usize;
    let bss_end_addr: usize;
    let l1_page_table_start_addr: usize;
    let l1_page_table_end_addr: usize;
    unsafe {
        boot_core_stack_end_exclusive_addr = &__boot_core_stack_end_exclusive as *const _ as usize;
        code_start_addr = &__code_start as *const _ as usize;
        code_end_addr = &__code_end_exclusive as *const _ as usize;
        data_start_addr = &__data_start as *const _ as usize;
        data_end_addr = &__data_end_exclusive as *const _ as usize;
        bss_start_addr = &__bss_start as *const _ as usize;
        bss_end_addr = &__bss_end_exclusive as *const _ as usize;
        l1_page_table_start_addr = &__l1_page_table_start as *const _ as usize;
        l1_page_table_end_addr = &__page_table_end_exclusive as *const _ as usize;
    }

    let l1_table = UnsafeTranslationTable::<Level1>::new(l1_page_table_start_addr as *mut L1Entry);

    // 1. recursive L1
    {
        // let mut recursive_table_entry = l1_table[config::RECURSIVE_L1_INDEX]
        // .get()
        // .set_table()
        // .unwrap();
        // recursive_table_entry.set_table_attributes()?;
        //
        // unsafe {
        // let mut recursive_page_entry = recursive_table_entry.table_to_page().unwrap();
        //
        // recursive_page_entry.set_RW_normal().unwrap();
        //
        // recursive_page_entry
        // .set_address(PhysicalAddress::try_from(l1_page_table_start_addr).unwrap())
        // .unwrap();
        // l1_table.set_entry(
        // config::RECURSIVE_L1_INDEX,
        // TranslationTableEntry::from(recursive_page_entry),
        // )?;
        // }
        // unsafe_println!(
        // "L1 table is recursively mapped to VA = {:#018x}",
        // config::L1_VIRTUAL_ADDRESS
        // );
    }

    // 2. Identity map before where MMIO starts using 4kb blocks
    // On pi3, L1[MMIO_START] = 0
    // On pi4, L1[MMIO_START] = 3
    {
        let boot_stack_end = VirtualAddress::try_from(boot_core_stack_end_exclusive_addr).unwrap();
        let code_start = VirtualAddress::try_from(code_start_addr).unwrap();
        let code_end = VirtualAddress::try_from(code_end_addr).unwrap();
        let data_start = VirtualAddress::try_from(data_start_addr).unwrap();
        let data_end = VirtualAddress::try_from(data_end_addr).unwrap();
        let bss_start = VirtualAddress::try_from(bss_start_addr).unwrap();
        let bss_end = VirtualAddress::try_from(bss_end_addr).unwrap();
        let _l1_page_table_start = VirtualAddress::try_from(l1_page_table_start_addr).unwrap();
        let peripheral_start = VirtualAddress::try_from(mmio::PERIPHERAL_START).unwrap();
        let memory_end = VirtualAddress::try_from(config::PHYSICAL_MEMORY_END_EXCLUSIVE).unwrap();

        let free_frame = PhysicalAddress::try_from(l1_page_table_end_addr)
            .unwrap()
            .to_frame();
        unsafe_println!("free frame from {:?}", free_frame);
        let mut linear_allocator = LinearFrameAllocator::new(free_frame);
        let table_walk_and_identity_map_4K = |va: VirtualAddress,
                                              mt: &MemoryType,
                                              frame_allocator: &mut dyn FrameAllocator|
         -> Result<(), ErrorCode> {
            if !va.is_4K_aligned() {
                return Err(EALIGN);
            }
            let mapped_to = PhysicalAddress::try_from(va.value()).unwrap();

            let mut l1_entry = l1_table[va.level1()].get();
            let l2_table = match l1_entry {
                Descriptor::INVALID => {
                    l1_entry = l1_entry.set_table()?;
                    l1_entry.set_attributes(TABLE_PAGE)?;
                    let allocated_frame_addr: PhysicalAddress =
                        frame_allocator.frame_alloc().ok_or(EFRAME)?.to_address();
                    l1_entry.set_address(allocated_frame_addr)?;
                    l1_table.set_entry(va.level1(), TranslationTableEntry::from(l1_entry))?;

                    Option::Some(UnsafeTranslationTable::<Level2>::new(
                        allocated_frame_addr.value() as *mut L2Entry,
                    ))
                }
                Descriptor::TableEntry(_) => Option::Some(UnsafeTranslationTable::<Level2>::new(
                    l1_entry.get_address().unwrap().value() as *mut L2Entry,
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
                        allocated_frame_addr.value() as *mut L3Entry,
                    ))
                }
                Descriptor::TableEntry(_) => Option::Some(UnsafeTranslationTable::<Level3>::new(
                    l2_entry.get_address().unwrap().value() as *mut L3Entry,
                )),
                _ => None,
            }
            .ok_or(ETYPE)?;

            let mut l3_entry: Descriptor = l3_table[va.level3()].get();
            match l3_entry {
                Descriptor::INVALID => {
                    l3_entry = l3_entry.set_page()?;
                    l3_entry.set_attributes(mt)?;
                    l3_entry.set_address(mapped_to)?;
                    l3_table.set_entry(va.level3(), TranslationTableEntry::from(l3_entry))?;
                }
                _ => return Err(ETYPE),
            };

            Ok(())
        };
        let table_walk_and_identity_map_2M = |va: VirtualAddress,
                                              mt: &MemoryType,
                                              frame_allocator: &mut dyn FrameAllocator|
         -> Result<(), ErrorCode> {
            if !va.is_2M_aligned() {
                return Err(EALIGN);
            }
            let mapped_to = PhysicalAddress::try_from(va.value()).unwrap();

            let mut l1_entry = l1_table[va.level1()].get();
            let l2_table = match l1_entry {
                Descriptor::INVALID => {
                    l1_entry = l1_entry.set_table()?;
                    l1_entry.set_attributes(TABLE_PAGE)?;
                    let allocated_frame_addr: PhysicalAddress =
                        frame_allocator.frame_alloc().ok_or(EFRAME)?.to_address();
                    l1_entry.set_address(allocated_frame_addr)?;
                    l1_table.set_entry(va.level1(), TranslationTableEntry::from(l1_entry))?;

                    Option::Some(UnsafeTranslationTable::<Level2>::new(
                        allocated_frame_addr.value() as *mut L2Entry,
                    ))
                }
                Descriptor::TableEntry(_) => Option::Some(UnsafeTranslationTable::<Level2>::new(
                    l1_entry.get_address().unwrap().value() as *mut L2Entry,
                )),
                _ => None,
            }
            .ok_or(ETYPE)?;

            let mut l2_entry = l2_table[va.level2()].get();
            match l2_entry {
                Descriptor::INVALID => {
                    l2_entry = l2_entry.set_l2_block()?;
                    l2_entry.set_attributes(mt)?;
                    l2_entry.set_address(mapped_to)?;
                    l2_table.set_entry(va.level2(), TranslationTableEntry::from(l2_entry))?;
                }
                _ => return Err(ETYPE),
            };

            Ok(())
        };

        let table_translate = |va: VirtualAddress| {
            let l1_entry = l1_table[va.level1()].get();
            unsafe_print!("va = {:?}, l1 = {}", va, l1_entry);
            match l1_entry {
                Descriptor::TableEntry(e) => {
                    let l2_table = UnsafeTranslationTable::<Level2>::new(
                        l1_entry.get_address().unwrap().value() as *mut L2Entry,
                    );
                    let l2_entry = l2_table[va.level2()].get();
                    unsafe_print!(", l2= {}", l2_entry);
                    match l2_entry {
                        Descriptor::TableEntry(e) => {
                            let l3_table = UnsafeTranslationTable::<Level3>::new(
                                l2_entry.get_address().unwrap().value() as *mut L3Entry,
                            );
                            let l3_entry = l3_table[va.level3()].get();
                            unsafe_print!(", l3= {}", l3_entry);
                            match l3_entry {
                                Descriptor::PageEntry(_) => {
                                    let pa = l3_entry.get_address().unwrap();
                                    unsafe_println!(", pa= {}", pa);
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
        };

        let va_start = VirtualAddress::try_from(0usize).unwrap();
        // table_walk_and_identity_map_2M(va_start, RWXNORMAL, &mut linear_allocator).unwrap();
        unsafe_println!("boot stack pages: {:?} -> {:?}", va_start, boot_stack_end);
        va_start.iter_4K_to(boot_stack_end).unwrap().for_each(|va| {
            // table_walk_and_identity_map_4K(va, RWNORMAL, &mut linear_allocator).unwrap();
            // table_translate(va);
        });
        unsafe_println!("code pages: {:?} -> {:?}", code_start, code_end);
        code_start.iter_4K_to(code_end).unwrap().for_each(|va| {
            // table_walk_and_identity_map_4K(va, XNORMAL, &mut linear_allocator).unwrap();
            // table_translate(va);
        });
        unsafe_println!("rodata pages: {:?} -> {:?}", data_start, data_end);
        data_start.iter_4K_to(data_end).unwrap().for_each(|va| {
            // table_walk_and_identity_map_4K(va, RONORMAL, &mut linear_allocator).unwrap();
            // table_translate(va);
        });
        unsafe_println!("bss pages: {:?} -> {:?}", bss_start, bss_end);
        bss_start.iter_4K_to(bss_end).unwrap().for_each(|va| {
            // table_walk_and_identity_map_4K(va, RWNORMAL, &mut linear_allocator).unwrap();
            // table_translate(va);
        });
        unsafe_println!("mmio pages: {:?} -> {:?}", peripheral_start, memory_end);
        peripheral_start
            .iter_2M_to(memory_end)
            .unwrap()
            .for_each(|va| {
                table_walk_and_identity_map_2M(va, RWDEVICE, &mut linear_allocator).unwrap();
                // table_translate(va);
            });
        unsafe_println!("The next free frame = {:?}", linear_allocator.peek());
        let l1_pa = PhysicalAddress::try_from(l1_page_table_start_addr).unwrap();
        set_ttbr0(l1_pa, 0);
    }

    Ok(())
}

#[cfg(test)]
#[allow(dead_code, unused_variables, unused_imports)]
mod tests {
    use super::*;
    use test_macros::kernel_test;

    #[kernel_test]
    fn test_translation_table() {}
}
