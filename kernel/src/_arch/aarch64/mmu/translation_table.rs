use super::{address::*, config, translation_entry::*};
use crate::println;
use aarch64_cpu::registers::TTBR0_EL1;
use core::{
    marker::PhantomData,
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
            PhysicalAddress::from(&__bss_end_exclusive as *const u8 as usize)
                .containing_frame_number();
        println!("available frame strating from : {}", available_frame_idx);
    }

    // identify mapping from 0x0000_0000 to 0xFFFF_FFFF(0b11_111111111_111111111_111111111111)
    // level1: 00, 01, 10, 11 (4 entries = 4GB)
    // level1's 511 entry is used for recursive mapping the level1 table

    FrameNumber::from(0)
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
    }
}
