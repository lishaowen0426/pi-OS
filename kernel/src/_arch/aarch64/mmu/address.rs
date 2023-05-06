use super::config;
use crate::utils::bitfields::Bitfields;
use core::fmt;

macro_rules! declare_address {
    ($name:ident, $tt:ty, $lit: literal $(,)?) => {
        #[derive(Default, Eq, PartialEq, Debug)]
        #[repr(transparent)]
        pub struct $name($tt);

        impl From<$tt> for $name {
            fn from(v: $tt) -> Self {
                Self(v)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, $lit, self.0)
            }
        }
    };
}

declare_address!(VirtualAddress, usize, "{:#018x}");
declare_address!(PhysicalAddress, usize, "{:#018x}");
declare_address!(PageNumber, usize, "{}");
declare_address!(FrameNumber, usize, "{}");

impl VirtualAddress {
    pub fn level1(&self) -> usize {
        (self.0 >> config::L1_INDEX_SHIFT) & config::INDEX_MASK
    }
    pub fn level2(&self) -> usize {
        (self.0 >> config::L2_INDEX_SHIFT) & config::INDEX_MASK
    }
    pub fn level3(&self) -> usize {
        (self.0 >> config::L3_INDEX_SHIFT) & config::INDEX_MASK
    }
    pub fn offset(&self) -> usize {
        (self.0 >> config::OFFSET_SHIFT) & config::OFFSET_MASK
    }

    pub fn set_level1(&self, idx: usize) -> Self {
        let mut addr = self.0;
        addr.set_bits(config::L1_RANGE, idx);
        Self(addr)
    }
    pub fn set_level2(&self, idx: usize) -> Self {
        let mut addr = self.0;
        addr.set_bits(config::L2_RANGE, idx);
        Self(addr)
    }
    pub fn set_level3(&self, idx: usize) -> Self {
        let mut addr = self.0;
        addr.set_bits(config::L3_RANGE, idx);
        Self(addr)
    }
    pub fn set_offset(&self, idx: usize) -> Self {
        let mut addr = self.0;
        addr.set_bits(config::OFFSET_RANGE, idx);
        Self(addr)
    }

    pub fn containing_page_number(&self) -> PageNumber {
        PageNumber::from(self.0 >> config::OFFSET_BITS)
    }
}

impl PhysicalAddress {
    pub fn containing_frame_number(&self) -> FrameNumber {
        FrameNumber::from(self.0 >> config::OFFSET_BITS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use test_macros::kernel_test;

    #[kernel_test]
    fn test_input_address_index() {
        {
            let va0: VirtualAddress = VirtualAddress::from(
                0b0000000000000000_000100100_011010001_010110011_110001001_101010111100,
            );
            assert!(va0.offset() == 0b101010111100);
            assert!(va0.level3() == 0b110001001);
            assert!(va0.level2() == 0b010110011);
            assert!(va0.level1() == 0b011010001);
        }

        {
            let va0: VirtualAddress = VirtualAddress::from(
                0b0000000000000000_000100100_011010001_010110011_110001001_101010111100,
            );
            assert_eq!(
                va0.set_offset(0b010101000011),
                VirtualAddress::from(
                    0b0000000000000000_000100100_011010001_010110011_110001001_010101000011
                )
            );
        }
        {
            let va0: VirtualAddress = VirtualAddress::from(
                0b0000000000000000_000100100_011010001_010110011_110001001_101010111100,
            );
            assert_eq!(
                va0.set_level3(0b011100101),
                VirtualAddress::from(
                    0b0000000000000000_000100100_011010001_010110011_011100101_101010111100
                )
            );
        }
        {
            let va0: VirtualAddress = VirtualAddress::from(
                0b0000000000000000_000100100_011010001_010110011_110001001_101010111100,
            );
            assert_eq!(
                va0.set_level2(0b011100101),
                VirtualAddress::from(
                    0b0000000000000000_000100100_011010001_011100101_110001001_101010111100
                )
            );
        }
        {
            let va0: VirtualAddress = VirtualAddress::from(VirtualAddress::from(
                0b0000000000000000_000100100_011010001_010110011_110001001_101010111100,
            ));
            assert_eq!(
                va0.set_level1(0b011100101),
                VirtualAddress::from(
                    0b0000000000000000_000100100_011100101_010110011_110001001_101010111100
                )
            );
        }
        {
            let va0: VirtualAddress = VirtualAddress::from(VirtualAddress::from(
                0b0000000000000000_000100100_011010001_010110011_110001001_101010111100,
            ));
            assert_eq!(
                va0.containing_page_number(),
                PageNumber::from(0b0000000000000000_000100100_011010001_010110011_110001001)
            );
        }
        {
            let last_frame = (0xFFFF_FFFF + 1) / 4096 - 1;
            let pa: PhysicalAddress = PhysicalAddress::from(0xFFFF_FFFF);
            assert_eq!(pa.containing_frame_number(), FrameNumber::from(last_frame));
        }
    }
}
