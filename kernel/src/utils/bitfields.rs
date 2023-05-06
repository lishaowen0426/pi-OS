use crate::println;
use core::ops::Range;

pub trait Bitfields
where
    Self: Sized,
{
    type Output = Self;

    fn get_bit(&self, index: usize) -> Self::Output;
    fn get_bits(&self, range: Range<usize>) -> Self::Output;

    fn set_bit(&mut self, index: usize, val: Self);
    fn set_bits(&mut self, range: Range<usize>, val: Self);
}

impl Bitfields for u64 {
    fn get_bit(&self, index: usize) -> Self::Output {
        (*self >> index) & 0b1
    }
    fn get_bits(&self, range: Range<usize>) -> Self::Output {
        let mask = (1 << (range.end - range.start)) - 1;
        (*self >> range.start) & mask
    }
    fn set_bit(&mut self, index: usize, val: Self) {
        let origin = *self;
        let mut higher: u64 = 0;
        if index < 63 {
            higher = (origin >> (index + 1)) << (index + 1);
        }
        let lower = origin & ((1 << index) - 1);
        let set = (val & 0b1) << index;
        *self = higher | set | lower;
    }
    fn set_bits(&mut self, range: Range<usize>, val: Self) {
        let origin = *self;
        let mut higher: u64 = 0;
        if range.end < 64 {
            higher = (origin >> range.end) << range.end;
        }
        let lower = origin & ((1 << range.start) - 1);
        let mask = (1 << (range.end - range.start)) - 1;
        let set = (val & mask) << range.start;
        *self = higher | set | lower;
    }
}
impl Bitfields for usize {
    fn get_bit(&self, index: usize) -> Self::Output {
        (*self >> index) & 0b1
    }
    fn get_bits(&self, range: Range<usize>) -> Self::Output {
        let mask = (1 << (range.end - range.start)) - 1;
        (*self >> range.start) & mask
    }
    fn set_bit(&mut self, index: usize, val: Self) {
        let origin = *self;
        let higher = (origin >> (index + 1)) << (index + 1);
        let lower = origin & ((1 << index) - 1);
        let set = (val & 0b1) << index;
        *self = higher | set | lower;
    }
    fn set_bits(&mut self, range: Range<usize>, val: Self) {
        let origin = *self;
        let higher = (origin >> range.end) << range.end;
        let lower = origin & ((1 << range.start) - 1);
        let mask = (1 << (range.end - range.start)) - 1;
        let set = (val & mask) << range.start;
        *self = higher | set | lower;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use test_macros::kernel_test;
    #[kernel_test]
    fn test_bitfields() {
        {
            let bf: u64 = 0x1234_5678_9ABC_DEF0;
            assert_eq!(bf.get_bit(63), 0);
            assert_eq!(bf.get_bit(0), 0);
            assert_eq!(bf.get_bit(60), 1);
            assert_eq!(bf.get_bit(4), 1);
        }

        {
            let bf: u64 = 0x1234_5678_9ABC_DEF0;
            assert_eq!(bf.get_bits(0..5), 0b10000);
            assert_eq!(bf.get_bits(60..64), 0b0001);
            assert_eq!(bf.get_bits(52..59), 0b0100011);
        }

        {
            let mut bf: u64 = 0b0001001000110100010101100111100010011010101111001101111011110000;
            bf.set_bit(60, 1);
            assert_eq!(
                bf,
                0b0001001000110100010101100111100010011010101111001101111011110000
            );
        }
        {
            let mut bf: u64 = 0b0001001000110100010101100111100010011010101111001101111011110000;
            bf.set_bit(60, 0);
            assert_eq!(
                bf,
                0b0000001000110100010101100111100010011010101111001101111011110000
            );
        }
        {
            let mut bf: u64 = 0b0001001000110100010101100111100010011010101111001101111011110000;
            bf.set_bits(56..61, 0b11010);
            assert_eq!(
                bf,
                0b0001101000110100010101100111100010011010101111001101111011110000
            );
        }
        {
            let mut bf: u64 = 0b0001001000110100010101100111100010011010101111001101111011110000;
            bf.set_bits(0..6, 0b11010);
            assert_eq!(
                bf,
                0b0001001000110100010101100111100010011010101111001101111011011010
            );
        }
    }
}
