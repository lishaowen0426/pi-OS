use crate::println;
use core::{convert::Into, ops::Range};

pub trait Bitfields {
    type Output;

    fn get_bit(&self, index: usize) -> Self::Output;
    fn get_bits(&self, range: Range<usize>) -> Self::Output;

    fn set_bit(&mut self, index: usize, val: u64);
    fn set_bits(&mut self, range: Range<usize>, val: u64);

    fn trailing_zeros(&self) -> usize;
    fn trailing_ones(&self) -> usize;

    fn first_zero(&self) -> usize;
    fn first_one(&self) -> usize;
}
// the rightmost bit is index 0
impl Bitfields for u64 {
    type Output = u64;
    fn get_bit(&self, index: usize) -> Self::Output {
        (*self >> index) & 0b1
    }
    fn get_bits(&self, range: Range<usize>) -> Self::Output {
        let mask = (1 << (range.end - range.start)) - 1;
        (*self >> range.start) & mask
    }
    fn set_bit(&mut self, index: usize, val: u64) {
        let origin = *self;
        let mut higher: u64 = 0;
        if index < 63 {
            higher = (origin >> (index + 1)) << (index + 1);
        }
        let lower = origin & ((1 << index) - 1);
        let set: u64 = (val as u64 & 0b1) << index;
        *self = higher | set | lower;
    }
    fn set_bits(&mut self, range: Range<usize>, val: u64) {
        let origin = *self;
        let mut higher: u64 = 0;
        if range.end < 64 {
            higher = (origin >> range.end) << range.end;
        }
        let lower = origin & ((1 << range.start) - 1);
        let mask = (1 << (range.end - range.start)) - 1;
        let set: u64 = (val as u64 & mask) << range.start;
        *self = higher | set | lower;
    }
    fn trailing_zeros(&self) -> usize {
        u64::trailing_zeros(*self) as usize
    }
    fn trailing_ones(&self) -> usize {
        u64::trailing_ones(*self) as usize
    }

    fn first_zero(&self) -> usize {
        let negated = !*self;
        u64::trailing_zeros(negated) as usize
    }
    fn first_one(&self) -> usize {
        u64::trailing_zeros(*self) as usize
    }
}

// the rightmost bit is index 0
impl Bitfields for usize {
    type Output = usize;
    fn get_bit(&self, index: usize) -> Self::Output {
        (*self >> index) & 0b1
    }
    fn get_bits(&self, range: Range<usize>) -> Self::Output {
        let mask = (1 << (range.end - range.start)) - 1;
        (*self >> range.start) & mask
    }
    fn set_bit(&mut self, index: usize, val: u64) {
        let origin = *self;
        let higher = (origin >> (index + 1)) << (index + 1);
        let lower = origin & ((1 << index) - 1);
        let set: usize = (val as usize & 0b1) << index;
        *self = higher | set | lower;
    }
    fn set_bits(&mut self, range: Range<usize>, val: u64) {
        let origin = *self;
        let higher = (origin >> range.end) << range.end;
        let lower = origin & ((1 << range.start) - 1);
        let mask = (1 << (range.end - range.start)) - 1;
        let set: usize = (val as usize & mask) << range.start;
        *self = higher | set | lower;
    }
    fn trailing_zeros(&self) -> usize {
        usize::trailing_zeros(*self) as usize
    }
    fn trailing_ones(&self) -> usize {
        usize::trailing_ones(*self) as usize
    }
    fn first_zero(&self) -> usize {
        let negated = !*self;
        usize::trailing_zeros(negated) as usize
    }
    fn first_one(&self) -> usize {
        usize::trailing_zeros(*self) as usize
    }
}
// index 0 starts from the rightmost bit, i.e., the last element of the array
impl Bitfields for [u64] {
    type Output = u64;
    fn get_bit(&self, index: usize) -> Self::Output {
        let base = index / 64;
        let offset = index % 64;

        let idx = self.len() - 1 - base;
        self[idx].get_bit(offset)
    }
    fn get_bits(&self, range: Range<usize>) -> Self::Output {
        let start_idx = self.len() - 1 - range.start / 64;
        let start_offset = range.start % 64;
        let end_idx = self.len() - 1 - range.end / 64;
        let end_offset = range.end % 64;

        if start_idx == end_idx {
            self[start_idx].get_bits(start_offset..end_offset)
        } else {
            let lower = self[start_offset].get_bits(start_offset..64);
            let higher = self[end_offset].get_bits(0..end_offset);
            (higher << (64 - start_offset)) | lower
        }
    }
    fn set_bit(&mut self, index: usize, val: u64) {
        let base = index / 64;
        let offset = index % 64;

        let idx = self.len() - 1 - base;
        self[idx].set_bit(offset, val)
    }
    fn set_bits(&mut self, range: Range<usize>, val: u64) {
        let start_idx = self.len() - 1 - range.start / 64;
        let start_offset = range.start % 64;
        let end_idx = self.len() - 1 - range.end / 64;
        let end_offset = range.end % 64;

        if start_idx == end_idx {
            self[start_idx].set_bits(start_offset..end_offset, val);
        } else {
            let lower_mask = (1 << (64 - start_offset)) - 1;
            let lower = val & lower_mask;
            let higher = val >> (64 - start_offset);
            self[start_idx].set_bits(start_offset..64, lower);
            self[end_idx].set_bits(0..end_offset, higher);
        }
    }
    fn trailing_zeros(&self) -> usize {
        let mut zeros: usize = 0;
        for e in self.iter().rev() {
            let e_zeros = u64::trailing_zeros(*e) as usize;
            zeros = zeros + e_zeros;
            if e_zeros < 64 {
                return zeros;
            }
        }
        zeros
    }
    fn trailing_ones(&self) -> usize {
        let mut ones: usize = 0;
        for e in self.iter().rev() {
            let e_ones = u64::trailing_ones(*e) as usize;
            ones = ones + e_ones;
            if e_ones < 64 {
                return ones;
            }
        }
        ones
    }

    fn first_zero(&self) -> usize {
        let mut ones: usize = 0;
        for e in self.iter().rev() {
            let negated = !*e;
            let e_ones = u64::trailing_zeros(negated) as usize;
            ones = ones + e_ones;
            if e_ones < 64 {
                return ones;
            }
        }
        ones
    }
    fn first_one(&self) -> usize {
        let mut zeros: usize = 0;
        for e in self.iter().rev() {
            let e_zeros = u64::trailing_zeros(*e) as usize;
            zeros = zeros + e_zeros;
            if e_zeros < 64 {
                return zeros;
            }
        }
        zeros
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::println;
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
            bf.set_bit(60, 1u64);
            assert_eq!(
                bf,
                0b0001001000110100010101100111100010011010101111001101111011110000
            );
        }
        {
            let mut bf: u64 = 0b0001001000110100010101100111100010011010101111001101111011110000;
            bf.set_bit(60, 0u64);
            assert_eq!(
                bf,
                0b0000001000110100010101100111100010011010101111001101111011110000
            );
        }
        {
            let mut bf: u64 = 0b0001001000110100010101100111100010011010101111001101111011110000;
            bf.set_bits(56..61, 0b11010u64);
            assert_eq!(
                bf,
                0b0001101000110100010101100111100010011010101111001101111011110000
            );
        }
        {
            let mut bf: u64 = 0b0001001000110100010101100111100010011010101111001101111011110000;
            bf.set_bits(0..6, 0b11010u64);
            assert_eq!(
                bf,
                0b0001001000110100010101100111100010011010101111001101111011011010
            );
        }

        {
            let mut arr: [u64; 3] = [0x0, 0x0, 0x0];
            assert_eq!(arr.get_bit(64), 0);

            arr.set_bit(64, 1);
            assert_eq!(arr.get_bit(64), 1);
            arr.set_bits(128..135, 0b1001101);
            assert_eq!(arr[0], 0b1001101);

            arr.set_bits(128..130, 0b100);
            assert_eq!(arr[0], 0b1001100);

            arr.set_bits(60..68, 0b10010110);
            assert_eq!(arr[2], (0b0110) << 60);
            assert_eq!(arr[1], 0b1001);
        }
        {
            let mut arr: [u64; 3] = [0x0, 0x0, 0x0];
            arr.set_bit(64, 1);
            assert_eq!(arr.trailing_zeros(), 64);
            arr.set_bit(32, 1);
            assert_eq!(arr.trailing_zeros(), 32);
        }
        {
            let mut arr: [u64; 3] = [0x0, 0x0, 0x0];
            assert_eq!(arr.first_zero(), 0);
            arr.set_bits(0..8, 0b10010111);
            assert_eq!(arr.first_zero(), 3);
            arr.set_bits(0..8, 0);
            arr.set_bits(21..30, 0b111111111);
            assert_eq!(arr.first_zero(), 0);
            assert_eq!(arr.first_one(), 21);
        }
    }
}
