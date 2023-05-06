use super::{address::*, config};
use crate::{
    errno::{ErrorCode, EALIGN},
    println,
    utils::bitfields::Bitfields,
};
use core::{fmt, marker::PhantomData};

#[derive(Default)]
pub struct Level1;
#[derive(Default)]
pub struct Level2;
#[derive(Default)]
pub struct Level3;

pub trait TranslationTableLevel {}
impl TranslationTableLevel for Level1 {}
impl TranslationTableLevel for Level2 {}
impl TranslationTableLevel for Level3 {}

pub trait TranslationTableLevel1Or2 {}
impl TranslationTableLevel1Or2 for Level1 {}
impl TranslationTableLevel1Or2 for Level2 {}

pub trait TranslationTableLevel1 {}
impl TranslationTableLevel1 for Level1 {}

pub trait TranslationTableLevel2 {}
impl TranslationTableLevel2 for Level2 {}

pub trait TranslationTableLevel3 {}
impl TranslationTableLevel3 for Level3 {}

#[derive(Default)]
#[repr(transparent)]
pub struct TranslationTableEntry<L> {
    entry: u64,
    _l: PhantomData<L>,
}

impl fmt::Display for TranslationTableEntry<Level1> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "L1: {:#018x}", self.entry)
    }
}
impl fmt::Display for TranslationTableEntry<Level2> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "L2: {:#018x}", self.entry)
    }
}
impl fmt::Display for TranslationTableEntry<Level3> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "L3: {:#018x}", self.entry)
    }
}

pub trait BlockAddress {
    fn set_output_addr(&mut self, v: u64) -> Result<(), ErrorCode>;
    fn get_output_addr(&self) -> u64;
}

#[repr(transparent)]
pub struct BlockEntry<'a, L> {
    entry: &'a mut u64,
    _l: PhantomData<L>,
}

#[repr(transparent)]
pub struct TableEntry<'a> {
    entry: &'a mut u64,
}

#[repr(transparent)]
pub struct PageEntry<'a> {
    entry: &'a mut u64,
}

#[allow(non_snake_case, dead_code)]
impl<'a, L: TranslationTableLevel> BlockEntry<'a, L> {
    pub fn get_UXN(&self) -> u8 {
        self.entry.get_bit(54) as u8
    }

    pub fn get_PXN(&self) -> u8 {
        self.entry.get_bit(53) as u8
    }

    pub fn get_Contiguous(&self) -> u8 {
        self.entry.get_bit(52) as u8
    }

    pub fn get_nG(&self) -> u8 {
        self.entry.get_bit(11) as u8
    }

    pub fn get_AF(&self) -> u8 {
        self.entry.get_bit(10) as u8
    }

    pub fn get_SH(&self) -> u8 {
        self.entry.get_bits(8..10) as u8
    }

    pub fn get_AP(&self) -> u8 {
        self.entry.get_bits(6..8) as u8
    }

    pub fn get_NS(&self) -> u8 {
        self.entry.get_bit(5) as u8
    }
    pub fn get_AttrIndx(&mut self) -> u8 {
        self.entry.get_bits(2..5) as u8
    }

    pub fn set_UXN(&mut self, v: u64) -> &mut Self {
        self.entry.set_bit(54, v);
        self
    }

    pub fn set_PXN(&mut self, v: u64) -> &mut Self {
        self.entry.set_bit(53, v);
        self
    }

    pub fn set_Contiguous(&mut self, v: u64) -> &mut Self {
        self.entry.set_bit(52, v);
        self
    }

    pub fn set_nG(&mut self, v: u64) -> &mut Self {
        self.entry.set_bit(11, v);
        self
    }

    pub fn set_AF(&mut self, v: u64) -> &mut Self {
        self.entry.set_bit(10, v);
        self
    }

    pub fn set_SH(&mut self, v: u64) -> &mut Self {
        self.entry.set_bits(8..10, v);
        self
    }

    pub fn set_AP(&mut self, v: u64) -> &mut Self {
        self.entry.set_bits(6..8, v);
        self
    }

    pub fn set_NS(&mut self, v: u64) -> &mut Self {
        self.entry.set_bit(5, v);
        self
    }

    pub fn set_AttrIndx(&mut self, v: u64) -> &mut Self {
        self.entry.set_bits(2..5, v);
        self
    }

    pub fn set_Upper(&mut self, v: u64) -> &mut Self {
        self.entry.set_bits(52..55, v & 0b111);
        self
    }
    pub fn set_Lower(&mut self, v: u64) -> &mut Self {
        self.entry.set_bits(2..12, v & 0b11_1111_1111);
        self
    }

    pub fn value(&self) -> u64 {
        *self.entry
    }
    pub fn set_valid(mut self) {
        self.entry.set_bit(0, 1);
    }
}
impl<'a> BlockAddress for BlockEntry<'a, Level1> {
    fn set_output_addr(&mut self, v: u64) -> Result<(), ErrorCode> {
        let output: u64 = (v >> 30) & 0x3FFFF;
        if (output << 30) != v {
            Err(EALIGN)
        } else {
            self.entry.set_bits(30..48, output);
            Ok(())
        }
    }
    fn get_output_addr(&self) -> u64 {
        self.entry.get_bits(30..48) << 30
    }
}
impl<'a> BlockAddress for BlockEntry<'a, Level2> {
    fn set_output_addr(&mut self, v: u64) -> Result<(), ErrorCode> {
        let output: u64 = (v >> 21) & ((1 << 27) - 1);
        if (output << 21) != v {
            Err(EALIGN)
        } else {
            self.entry.set_bits(21..48, output);
            Ok(())
        }
    }
    fn get_output_addr(&self) -> u64 {
        self.entry.get_bits(21..48) << 21
    }
}
#[allow(non_snake_case, dead_code)]
impl<'a> PageEntry<'a> {
    pub fn get_UXN(&self) -> u8 {
        self.entry.get_bit(54) as u8
    }

    pub fn get_PXN(&self) -> u8 {
        self.entry.get_bit(53) as u8
    }

    pub fn get_Contiguous(&self) -> u8 {
        self.entry.get_bit(52) as u8
    }

    pub fn get_nG(&self) -> u8 {
        self.entry.get_bit(11) as u8
    }

    pub fn get_AF(&self) -> u8 {
        self.entry.get_bit(10) as u8
    }

    pub fn get_SH(&self) -> u8 {
        self.entry.get_bits(8..10) as u8
    }

    pub fn get_AP(&self) -> u8 {
        self.entry.get_bits(6..8) as u8
    }

    pub fn get_NS(&self) -> u8 {
        self.entry.get_bit(5) as u8
    }
    pub fn get_AttrIndx(&mut self) -> u8 {
        self.entry.get_bits(2..5) as u8
    }

    pub fn set_UXN(&mut self, v: u64) -> &mut Self {
        self.entry.set_bit(54, v);
        self
    }

    pub fn set_PXN(&mut self, v: u64) -> &mut Self {
        self.entry.set_bit(53, v);
        self
    }

    pub fn set_Contiguous(&mut self, v: u64) -> &mut Self {
        self.entry.set_bit(52, v);
        self
    }

    pub fn set_nG(&mut self, v: u64) -> &mut Self {
        self.entry.set_bit(11, v);
        self
    }

    pub fn set_AF(&mut self, v: u64) -> &mut Self {
        self.entry.set_bit(10, v);
        self
    }

    pub fn set_SH(&mut self, v: u64) -> &mut Self {
        self.entry.set_bits(8..10, v);
        self
    }

    pub fn set_AP(&mut self, v: u64) -> &mut Self {
        self.entry.set_bits(6..8, v);
        self
    }

    pub fn set_NS(&mut self, v: u64) -> &mut Self {
        self.entry.set_bit(5, v);
        self
    }

    pub fn set_AttrIndx(&mut self, v: u64) -> &mut Self {
        self.entry.set_bits(2..5, v);
        self
    }

    pub fn set_Upper(&mut self, v: u64) -> &mut Self {
        self.entry.set_bits(52..55, v & 0b111);
        self
    }
    pub fn set_Lower(&mut self, v: u64) -> &mut Self {
        self.entry.set_bits(2..12, v & 0b11_1111_1111);
        self
    }
    pub fn get_output_addr(&self) -> u64 {
        self.entry.get_bits(12..48) << 12
    }

    pub fn set_output_addr(&mut self, v: u64) -> Result<(), ErrorCode> {
        let output: u64 = (v >> 12) & ((1 << 36) - 1);
        if (output << 12) != v {
            Err(EALIGN)
        } else {
            self.entry.set_bits(12..48, output);
            Ok(())
        }
    }
    pub fn value(&self) -> u64 {
        *self.entry
    }
    pub fn set_valid(mut self) {
        self.entry.set_bit(0, 1);
    }
}

#[allow(non_snake_case, dead_code)]
impl<'a> TableEntry<'a> {
    pub fn get_NS(&self) -> u8 {
        self.entry.get_bit(63) as u8
    }
    pub fn get_AP(&self) -> u8 {
        self.entry.get_bits(61..63) as u8
    }
    pub fn get_UXN(&self) -> u8 {
        self.entry.get_bit(60) as u8
    }
    pub fn get_PXN(&self) -> u8 {
        self.entry.get_bit(59) as u8
    }

    pub fn set_NS(&mut self, v: u64) -> &mut Self {
        self.entry.set_bit(63, v);
        self
    }
    pub fn set_AP(&mut self, v: u64) -> &mut Self {
        self.entry.set_bits(61..63, v);
        self
    }
    pub fn set_UXN(&mut self, v: u64) -> &mut Self {
        self.entry.set_bit(60, v);
        self
    }
    pub fn set_PXN(&mut self, v: u64) -> &mut Self {
        self.entry.set_bit(59, v);
        self
    }

    pub fn get_next_level_table_addr(&self) -> u64 {
        self.entry.get_bits(12..48) << 12
    }

    pub fn set_next_level_table_addr(&mut self, v: u64) -> Result<(), ErrorCode> {
        let output: u64 = (v >> 12) & ((1 << 36) - 1);
        if (output << 12) != v {
            Err(EALIGN)
        } else {
            self.entry.set_bits(12..48, output);
            Ok(())
        }
    }

    pub fn value(&self) -> u64 {
        *self.entry
    }
    pub fn set_valid(mut self) {
        self.entry.set_bit(0, 1);
    }
}
impl<L> TranslationTableEntry<L> {
    pub fn is_valid(&self) -> bool {
        self.entry.get_bit(0) == 1
    }

    pub fn invalid(&mut self) {
        self.entry = 0;
    }

    pub fn get_type(&self) -> u8 {
        self.entry.get_bit(1) as u8
    }

    pub fn value(&self) -> u64 {
        self.entry
    }
}

impl<L: TranslationTableLevel1Or2> TranslationTableEntry<L> {
    pub fn set_block(&mut self) -> BlockEntry<L> {
        self.entry.set_bit(1, 0);
        BlockEntry {
            entry: &mut self.entry,
            _l: PhantomData,
        }
    }
    pub fn set_table(&mut self) -> TableEntry {
        self.entry.set_bit(1, 1);
        TableEntry {
            entry: &mut self.entry,
        }
    }
}
impl<L: TranslationTableLevel3> TranslationTableEntry<L> {
    pub fn set_page(&mut self) -> PageEntry {
        self.entry.set_bit(1, 1);
        PageEntry {
            entry: &mut self.entry,
        }
    }
}

// static TABLE: *mut TranslationTable<Level0> = &mut _page_table as *mut u8 as usize as *mut _;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::println;
    #[allow(unused_imports)]
    use test_macros::kernel_test;

    #[kernel_test]
    fn test_translation_table_entry() {
        // Level 1 block
        {
            let mut e: TranslationTableEntry<Level1> = Default::default();
            assert!(!e.is_valid());
            let mut b: BlockEntry<_> = e.set_block();
            let mut ans: u64 = (0b101 << 52) | (0b1000010101 << 2);
            b.set_UXN(1)
                .set_Contiguous(1)
                .set_nG(1)
                .set_AP(0b01)
                .set_AttrIndx(0b101);
            assert_eq!(b.value(), ans);

            b.set_output_addr(0x3FFFF << 30 | 0b1001)
                .expect_err("Wrong address");
            b.set_output_addr(0x4FFFF << 30).expect_err("Wrong address");
            b.set_output_addr(0x4FFFF << 30 | 0b1001)
                .expect_err("Wrong address");

            b.set_output_addr(0x0A46 << 30).unwrap();
            ans |= 0x0A46 << 30;
            assert_eq!(b.value(), ans);

            b.set_valid();
            ans |= 0b1;
            assert_eq!(e.value(), ans);

            e.invalid();
            ans = 0;
            assert_eq!(e.value(), ans);
        }
        // Level 2 block
        {
            let mut e: TranslationTableEntry<Level2> = Default::default();
            assert!(!e.is_valid());
            let mut b: BlockEntry<_> = e.set_block();
            let mut ans: u64 = (0b010 << 52) | (0b1010001010 << 2);
            b.set_PXN(1)
                .set_nG(1)
                .set_SH(0b10)
                .set_NS(1)
                .set_AttrIndx(0b010);

            assert_eq!(b.value(), ans);

            b.set_output_addr(0x7FFFFFF << 21 | 0b1001)
                .expect_err("Wrong address");
            b.set_output_addr(0x8FFFFFF << 21)
                .expect_err("Wrong address");

            b.set_output_addr(0x8FFFFFF << 21 | 0b1001)
                .expect_err("Wrong address");
            b.set_output_addr((0x30FBC56 as u64) << 21).unwrap();
            ans |= 0x30FBC56 << 21;
            assert_eq!(b.value(), ans);
            b.set_valid();
            ans |= 0b1;
            assert_eq!(e.value(), ans);

            e.invalid();
            ans = 0;
            assert_eq!(e.value(), ans);
        }

        //  Level 1 Table
        {
            let mut e: TranslationTableEntry<Level1> = Default::default();
            assert!(!e.is_valid());
            let mut b: TableEntry = e.set_table();
            let mut ans: u64 = 0b11001 << 59 | 0b10;
            b.set_NS(1).set_AP(0b10).set_PXN(1);
            assert_eq!(b.value(), ans);

            b.set_next_level_table_addr(0xFFFFFFFFF << 12 | 0b101)
                .expect_err("Wrong address");
            b.set_next_level_table_addr(0x1FFFFFFFFF << 12)
                .expect_err("Wrong address");
            b.set_next_level_table_addr(0x2FFFFFFFFF << 12 | 0b101)
                .expect_err("Wrong address");
            b.set_next_level_table_addr(0x174AB3DCF << 12).unwrap();
            ans |= 0x174AB3DCF << 12;
            assert_eq!(b.value(), ans);
            b.set_valid();
            ans |= 0b1;
            assert_eq!(e.value(), ans);

            e.invalid();
            ans = 0;
            assert_eq!(e.value(), ans);
        }

        // Level 3 Page
        {
            let mut e: TranslationTableEntry<Level3> = Default::default();
            let mut b: PageEntry = e.set_page();
            let mut ans: u64 = (0b101 << 52) | 0b100001010110;
            b.set_UXN(1)
                .set_Contiguous(1)
                .set_nG(1)
                .set_AP(0b01)
                .set_AttrIndx(0b101);
            assert_eq!(b.value(), ans);
            b.set_output_addr(0xFFFFFFFFF << 12 | 0b101)
                .expect_err("Wrong address");
            b.set_output_addr(0x1FFFFFFFFF << 12)
                .expect_err("Wrong address");
            b.set_output_addr(0x2FFFFFFFFF << 12 | 0b101)
                .expect_err("Wrong address");
            b.set_output_addr(0x174AB3DCF << 12).unwrap();
            ans |= 0x174AB3DCF << 12;
            assert_eq!(b.value(), ans);
            b.set_valid();
            ans |= 0b1;
            assert_eq!(e.value(), ans);

            e.invalid();
            ans = 0;
            assert_eq!(e.value(), ans);
        }
    }
}
