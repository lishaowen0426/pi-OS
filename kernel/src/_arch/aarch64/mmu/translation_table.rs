use super::{address::*, config, frame_allocator::*, translation_entry::*};
use crate::{bsp::mmio, errno::*, println};
use aarch64_cpu::registers::TTBR0_EL1;
use core::{
    ops::{Index, IndexMut},
    ptr::NonNull,
};

extern "C" {
    static __boot_core_stack_end_exclusive: u8;
    static __code_start: u8;
    static __code_end_exclusive: u8;
    static __bss_start: u8;
    static __bss_end_exclusive: u8;
}

pub struct TranslationTable<L> {
    base: NonNull<TranslationTableEntry<L>>,
    num_elems: usize,
}

impl<L: TranslationTableLevel> Index<usize> for TranslationTable<L> {
    type Output = TranslationTableEntry<L>;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { self.base.as_ptr().offset(index as isize).as_ref().unwrap() }
    }
}
impl<L: TranslationTableLevel> IndexMut<usize> for TranslationTable<L> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe { self.base.as_ptr().offset(index as isize).as_mut().unwrap() }
    }
}

impl<L: TranslationTableLevel> TranslationTable<L> {
    pub fn new(ptr: *mut TranslationTableEntry<L>, num_elems: usize) -> Self {
        Self {
            base: NonNull::new(ptr).unwrap(),
            num_elems,
        }
    }

    pub fn address(&self) -> VirtualAddress {
        VirtualAddress::try_from(self.base.addr().get()).unwrap()
    }
}

pub fn get_ttbr0() -> usize {
    TTBR0_EL1.get_baddr() as usize
}

type L1TranslationTable = TranslationTable<Level1>;
type L2TranslationTable = TranslationTable<Level2>;
type L3TranslationTable = TranslationTable<Level3>;

impl L1TranslationTable {
    pub fn translate(&self, va: VirtualAddress) -> Option<PhysicalAddress> {
        None
    }
    pub fn map(
        &mut self,
        va: VirtualAddress,
        pa: PhysicalAddress,
        frame_allocator: &mut dyn FrameAllocator,
    ) -> Result<(), ErrorCode> {
        Ok(())
    }

    pub fn set_up_init_mapping(&mut self) -> Result<(), ErrorCode> {
        let mut boot_core_stack_end_exclusive_addr: usize = 0;
        let mut code_start_addr: usize = 0;
        let mut code_end_addr: usize = 0;
        let mut bss_start_addr: usize = 0;
        let mut bss_end_addr: usize = 0;
        let l1_page_start_addr: usize = self.base.addr().get();
        let l1_page_end_addr: usize = l1_page_start_addr + config::PAGE_SIZE;
        unsafe {
            boot_core_stack_end_exclusive_addr =
                &__boot_core_stack_end_exclusive as *const _ as usize;
            code_start_addr = &__code_start as *const _ as usize;
            code_end_addr = &__code_end_exclusive as *const _ as usize;
            bss_start_addr = &__bss_start as *const _ as usize;
            bss_end_addr = &__bss_end_exclusive as *const _ as usize;
        }
        // 1. recursive L1
        {
            let mut recursive_table_entry =
                self[config::RECURSIVE_L1_INDEX].get().set_table().unwrap();
            recursive_table_entry.set_table_attributes();

            println!("recursive {}", recursive_table_entry);
            unsafe {
                let l1_page_table_start = PhysicalAddress::try_from(l1_page_start_addr).unwrap();
                println!("l1 page table: {:?}", l1_page_table_start);
                let mut recursive_page_entry = recursive_table_entry.table_to_page().unwrap();

                recursive_page_entry.set_RW_normal().unwrap();

                recursive_page_entry
                    .set_address(l1_page_table_start)
                    .unwrap();
                println!("recursive {}", recursive_page_entry);
                self[config::RECURSIVE_L1_INDEX] =
                    TranslationTableEntry::from(recursive_page_entry);
            }

            println!("entry {}", self[config::RECURSIVE_L1_INDEX]);
        }

        // 2. Identity map before where MMIO starts using l1 1GB blocks
        // On pi3, L1[MMIO_START] = 0
        // On pi4, L1[MMIO_START] = 3
        {
            let boot_stack_end =
                VirtualAddress::try_from(boot_core_stack_end_exclusive_addr).unwrap();
            let code_start = VirtualAddress::try_from(code_start_addr).unwrap();
            let code_end = VirtualAddress::try_from(code_end_addr).unwrap();
            let bss_start = VirtualAddress::try_from(bss_start_addr).unwrap();
            let bss_end = VirtualAddress::try_from(bss_end_addr).unwrap();
            let peripheral_start = VirtualAddress::try_from(0xFE00_0000usize).unwrap();
            let l1_page_table_start = VirtualAddress::try_from(l1_page_start_addr).unwrap();
            println!("boot stack top: {:?}", boot_stack_end);
            println!("code start: {:?}", code_start);
            println!("code end: {:?}", code_end);
            println!("bss start: {:?}", bss_start);
            println!("bss end: {:?}", bss_end);
            println!("l1 page table: {:?}", l1_page_table_start);
            println!("MMIO start: {:?}", peripheral_start);

            let mut free_frame = PhysicalAddress::try_from(l1_page_end_addr)
                .unwrap()
                .to_frame();
            println!("free frame from {:?}", free_frame);
            let mut linear_allocator = LinearFrameAllocator::new(free_frame);

            let mut table_walk_and_identity_map_4k = |va: VirtualAddress,
                                                      mt: &MemoryType,
                                                      frame_allocator: &mut dyn FrameAllocator|
             -> Result<(), ErrorCode> {
                let mapped_to = PhysicalAddress::try_from(va.value()).unwrap();

                let mut l1_entry = self[va.level1()].get();
                let mut l2_table: L2TranslationTable = match l1_entry {
                    Descriptor::INVALID => {
                        l1_entry = l1_entry.set_table()?;
                        l1_entry.set_attributes(TABLE_PAGE);
                        let allocated_frame_addr: PhysicalAddress =
                            frame_allocator.frame_alloc().ok_or(EFRAME)?.to_address();
                        l1_entry.set_address(allocated_frame_addr);
                        self[va.level1()] = TranslationTableEntry::from(l1_entry);

                        Option::Some(TranslationTable::<Level2>::new(
                            allocated_frame_addr.value() as *mut L2Entry,
                            config::ENTRIES_PER_TABLE,
                        ))
                    }
                    Descriptor::TableEntry(e) => Option::Some(TranslationTable::<Level2>::new(
                        l1_entry.get_address().unwrap().value() as *mut L2Entry,
                        config::ENTRIES_PER_TABLE,
                    )),
                    _ => None,
                }
                .ok_or(ETYPE)?;
                println!("l1[{}] = {}", va.level1(), self[va.level1()].get());

                let mut l2_entry = l2_table[va.level2()].get();
                let mut l3_table: L3TranslationTable = match l2_entry {
                    Descriptor::INVALID => {
                        l2_entry = l2_entry.set_table()?;
                        l2_entry.set_attributes(TABLE_PAGE);
                        let allocated_frame_addr: PhysicalAddress =
                            frame_allocator.frame_alloc().ok_or(EFRAME)?.to_address();
                        l2_entry.set_address(allocated_frame_addr);
                        l2_table[va.level2()] = TranslationTableEntry::from(l2_entry);
                        Option::Some(TranslationTable::<Level3>::new(
                            allocated_frame_addr.value() as *mut L3Entry,
                            config::ENTRIES_PER_TABLE,
                        ))
                    }
                    Descriptor::TableEntry(e) => Option::Some(TranslationTable::<Level3>::new(
                        l2_entry.get_address().unwrap().value() as *mut L3Entry,
                        config::ENTRIES_PER_TABLE,
                    )),
                    _ => None,
                }
                .ok_or(ETYPE)?;
                println!("l2[{}] = {}", va.level2(), l2_table[va.level2()].get());

                let mut l3_entry: Descriptor = l3_table[va.level3()].get();
                match l3_entry {
                    Descriptor::INVALID => {
                        l3_entry = l3_entry.set_page()?;
                        l3_entry.set_attributes(mt)?;
                        l3_entry.set_address(PhysicalAddress::try_from(va.value()).unwrap())?;
                        l3_table[va.level3()] = TranslationTableEntry::from(l3_entry);
                    }
                    _ => return Err(ETYPE),
                };

                println!("l3[{}] = {}", va.level3(), l3_table[va.level3()].get());

                Ok(())
            };

            let va_start = VirtualAddress::try_from(0usize).unwrap();
            // let peripheral_start = VirtualAddress::try_from(mmio::PERIPHERAL_START).unwrap();
            println!("boot stack pages");
            va_start.iter_4K_to(boot_stack_end).unwrap().for_each(|va| {
                table_walk_and_identity_map_4k(va, RWNORMAL, &mut linear_allocator).unwrap();
            });
            println!("code pages");
            code_start.iter_4K_to(code_end).unwrap().for_each(|va| {
                table_walk_and_identity_map_4k(va, RONORMAL, &mut linear_allocator).unwrap();
            });
            println!("bss pages");
            bss_start.iter_4K_to(bss_end).unwrap().for_each(|va| {
                table_walk_and_identity_map_4k(va, RWNORMAL, &mut linear_allocator).unwrap();
            });

            println!("The next free frame = {:?}", linear_allocator.peek());
            println!("Have you mapped page table frames and mmio?");
        }

        Ok(())
    }
}

#[cfg(test)]
#[allow(dead_code, unused_variables, unused_imports)]
mod tests {
    use super::*;
    use test_macros::kernel_test;

    #[kernel_test]
    fn test_translation_table() {
        {
            let va_start = VirtualAddress::default();
            let va_peripheral = VirtualAddress::try_from(mmio::PERIPHERAL_START).unwrap();
            let va_end = VirtualAddress::try_from(0xFFFF_FFFFusize).unwrap();
            let pa_start = PhysicalAddress::default();
            let pa_end = PhysicalAddress::try_from(0xFFFF_FFFFusize).unwrap();
            let l1_virt = VirtualAddress::try_from(config::L1_VIRTUAL_ADDRESS).unwrap();
            println!("Identity mapping from {} to {}", va_start, va_end);
            println!(
                "l1 entries are {} to {}",
                va_start.level1(),
                va_end.level1()
            );

            println!(
                "Peripheral starts at {}, it is at l1[{}], l2[{}], l3[{}]",
                va_peripheral,
                va_peripheral.level1(),
                va_peripheral.level2(),
                va_peripheral.level3()
            );

            println!(
                "L1 table virtual address {}, it is at l1[{}], l2[{}], l3[{}]",
                l1_virt,
                l1_virt.level1(),
                l1_virt.level2(),
                l1_virt.level3()
            );
        }
        {
            println!("ttbr0: {:#018x}", get_ttbr0());
            let mut l1_table: L1TranslationTable =
                TranslationTable::new(get_ttbr0() as *mut L1Entry, config::ENTRIES_PER_TABLE);
            let l1_addr: usize = get_ttbr0();

            assert_eq!(
                l1_table.address(),
                VirtualAddress::try_from(l1_addr).unwrap()
            );
            assert_eq!(
                &l1_table[config::RECURSIVE_L1_INDEX] as *const _ as usize,
                l1_addr + config::RECURSIVE_L1_INDEX * 8
            );

            l1_table.set_up_init_mapping();
        }

        // set up the recursive entry
    }
}
