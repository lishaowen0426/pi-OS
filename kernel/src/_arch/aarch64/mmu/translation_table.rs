use super::{address::*, config, translation_entry::*};
use crate::{errno::*, println};
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

// #[repr(transparent)]
// pub struct TranslationTable<L> {
// pub entries: [TranslationTableEntry<L>; config::ENTRIES_PER_TABLE],
// _l: PhantomData<L>,
// }
//
// impl<L: TranslationTableLevel> Index<usize> for TranslationTable<L> {
// type Output = TranslationTableEntry<L>;
// fn index(&self, index: usize) -> &Self::Output {
// &self.entries[index]
// }
// }
// impl<L: TranslationTableLevel> IndexMut<usize> for TranslationTable<L> {
// fn index_mut(&mut self, index: usize) -> &mut Self::Output {
// &mut self.entries[index]
// }
// }

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

    pub fn address(&self) -> usize {
        self.base.addr().get()
    }
}

pub fn get_ttbr0() -> usize {
    TTBR0_EL1.get_baddr() as usize
}

pub fn set_up_init_translation_table() -> FrameNumber {
    let mut l1_table: TranslationTable<Level1> = TranslationTable::new(
        get_ttbr0() as *mut TranslationTableEntry<Level1>,
        config::ENTRIES_PER_TABLE,
    );

    println!("l1 table address: {:x}", l1_table.address());
    println!(
        "l1 table 511 entry's address: {:x}",
        &l1_table[config::RECURSIVE_L1_INDEX] as *const _ as usize
    );
    unsafe {
        let mut available_frame_idx =
            PhysicalAddress::try_from(&__bss_end_exclusive as *const u8 as usize)
                .unwrap()
                .to_frame();
        println!("available frame strating from : {}", available_frame_idx);
    }

    // identify mapping from 0x0000_0000 to 0xFFFF_FFFF(0b11_111111111_111111111_111111111111)
    // level1: 00, 01, 10, 11 (4 entries = 4GB)
    // level1's 511 entry is used for recursive mapping the level1 table

    FrameNumber::try_from(0).unwrap()
}

impl TranslationTable<Level1> {
    pub fn set_up_init_mapping(&mut self) -> Result<(), ErrorCode> {
        // 1. recursive L1
        let addr: usize = self.base.addr().get();
        println!("l1 physical addr: {:x}", addr);
        let mut recursive_table_entry = self[config::RECURSIVE_L1_INDEX].set_table();
        // APTable = 01: Accesses from EL0 are never permitted in subsequent tables
        recursive_table_entry
            .set_APTable(0b01)
            .set_UXNTable(0b1)
            .set_PXNTable(0b1)
            .set_NSTable(0);
        unsafe {
            let mut recursive_page_entry = recursive_table_entry.convert_to_page();
            // AP = 00:  EL1 -> R/W, EL0 -> Fault
            // UXN = 1:  Non-executable at EL0
            // PXN = 1:  Non-executable at EL1
            // SH = 11:  L1 page table is inner shareable
            // AttrIdx = 1:  Attr1 is used. Check the TCR_EL1 and MAIR_EL1 config
            // Contiguous = 0
            // NS = 0:   The output address is always in the secure state
            // nG = 0:   The translation is global. We only use a single AS, this should not matter
            // AF = 1?:  Not sure
            recursive_page_entry
                .set_AP(0b00)
                .set_UXN(0b1)
                .set_PXN(0b1)
                .set_SH(0b11)
                .set_AttrIdx(0b1)
                .set_Contiguous(0)
                .set_NS(0)
                .set_nG(0)
                .set_AF(1)
                .set_output_addr(PhysicalAddress::try_from(addr).unwrap())
                .unwrap();
            println!("recursive entry: {}", recursive_page_entry);
            recursive_page_entry.set_valid();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use test_macros::kernel_test;

    #[kernel_test]
    fn test_translation_table() {
        unsafe {
            println!("boot_core_stack_end:{:p}", &__boot_core_stack_end_exclusive);
            println!("code_start:{:p}", &__code_start);
            println!("code_end:{:p}", &__code_end_exclusive);
            println!("bss_start:{:p}", &__bss_start);
            println!("table:{:x}", get_ttbr0());
            println!("bss_end:{:p}", &__bss_end_exclusive);
        }

        let mut l1_table: TranslationTable<Level1> = TranslationTable::new(
            get_ttbr0() as *mut TranslationTableEntry<Level1>,
            config::ENTRIES_PER_TABLE,
        );
        let l1_addr: usize = get_ttbr0();

        assert_eq!(l1_table.address(), l1_addr);
        assert_eq!(
            &l1_table[config::RECURSIVE_L1_INDEX] as *const _ as usize,
            l1_addr + config::RECURSIVE_L1_INDEX * 8
        );

        l1_table.set_up_init_mapping();

        // set up the recursive entry
    }
}
