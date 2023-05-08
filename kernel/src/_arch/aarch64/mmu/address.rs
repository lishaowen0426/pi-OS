use super::config;
use crate::{errno::*, utils::bitfields::Bitfields};
use core::{
    convert::{TryFrom, TryInto},
    fmt,
};

#[allow(non_snake_case)]
pub trait Address {
    fn is_4K_aligned(&self) -> bool;
    fn is_16K_aligned(&self) -> bool;
    fn is_64K_aligned(&self) -> bool;
    fn is_2M_aligned(&self) -> bool;
    fn is_1G_aligned(&self) -> bool;

    fn shift_4K(&self) -> usize;
    fn shift_16K(&self) -> usize;
    fn shift_64K(&self) -> usize;
    fn shift_2M(&self) -> usize;
    fn shift_1G(&self) -> usize;
}
macro_rules! declare_address {
    ($name:ident, $tt:ty, $lit: literal $(,)?) => {
        #[derive(Default, Eq, PartialEq, Debug, Clone, Copy)]
        #[repr(transparent)]
        pub struct $name($tt);

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, $lit, self.0)
            }
        }
        impl fmt::LowerHex for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, $lit, self.0)
            }
        }

        impl Address for $name {
            fn is_4K_aligned(&self) -> bool {
                (self.0 & !config::MASK_4K) == self.0
            }
            fn is_16K_aligned(&self) -> bool {
                (self.0 & !config::MASK_16K) == self.0
            }
            fn is_64K_aligned(&self) -> bool {
                (self.0 & !config::MASK_64K) == self.0
            }
            fn is_2M_aligned(&self) -> bool {
                (self.0 & !config::MASK_2M) == self.0
            }
            fn is_1G_aligned(&self) -> bool {
                (self.0 & !config::MASK_1G) == self.0
            }

            fn shift_4K(&self) -> usize {
                self.0 >> config::SHIFT_4K
            }
            fn shift_16K(&self) -> usize {
                self.0 >> config::SHIFT_16K
            }
            fn shift_64K(&self) -> usize {
                self.0 >> config::SHIFT_64K
            }
            fn shift_2M(&self) -> usize {
                self.0 >> config::SHIFT_2M
            }
            fn shift_1G(&self) -> usize {
                self.0 >> config::SHIFT_1G
            }
        }
    };
}
macro_rules! declare_number {
    ($name:ident, $tt:ty,  $lit: literal $(,)?) => {
        #[derive(Default, Eq, PartialEq, Debug, Clone, Copy)]
        #[repr(transparent)]
        pub struct $name($tt);

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, $lit, self.0)
            }
        }
    };
}

declare_address!(VirtualAddress, usize, "{:#018x}");
declare_address!(PhysicalAddress, usize, "{:#018x}");
declare_number!(PageNumber, usize, "{}");
declare_number!(FrameNumber, usize, "{}");

impl TryFrom<usize> for VirtualAddress {
    type Error = core::convert::Infallible;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(Self(value))
    }
}
impl TryFrom<usize> for PhysicalAddress {
    type Error = ErrorCode;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value > ((1usize << 48) - 1) {
            Err(EOVERFLOW)
        } else {
            Ok(Self(value))
        }
    }
}
impl TryFrom<usize> for PageNumber {
    type Error = core::convert::Infallible;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(Self(value))
    }
}
impl TryFrom<usize> for FrameNumber {
    type Error = ErrorCode;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value > (((1usize << 48) - 1) >> config::SHIFT_4K) {
            Err(EOVERFLOW)
        } else {
            Ok(Self(value))
        }
    }
}

pub trait Virtual {
    fn to_address(&self) -> VirtualAddress;
    fn to_page(&self) -> PageNumber;
}

pub trait Physical {
    fn to_address(&self) -> PhysicalAddress;
    fn to_frame(&self) -> FrameNumber;
}

impl Virtual for VirtualAddress {
    fn to_address(&self) -> VirtualAddress {
        *self
    }
    fn to_page(&self) -> PageNumber {
        PageNumber::try_from(self.0 >> config::SHIFT_4K).unwrap()
    }
}
impl Virtual for PageNumber {
    fn to_address(&self) -> VirtualAddress {
        VirtualAddress::try_from(self.0 << config::SHIFT_4K).unwrap()
    }
    fn to_page(&self) -> PageNumber {
        *self
    }
}
impl Physical for PhysicalAddress {
    fn to_address(&self) -> PhysicalAddress {
        *self
    }
    fn to_frame(&self) -> FrameNumber {
        FrameNumber::try_from(self.0 >> config::SHIFT_4K).unwrap()
    }
}
impl Physical for FrameNumber {
    fn to_address(&self) -> PhysicalAddress {
        PhysicalAddress::try_from(self.0 << config::SHIFT_4K).unwrap()
    }
    fn to_frame(&self) -> FrameNumber {
        *self
    }
}

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

    pub fn set_level1<T>(&mut self, idx: T)
    where
        T: TryInto<usize>,
        <T as TryInto<usize>>::Error: fmt::Debug,
    {
        self.0.set_bits(config::L1_RANGE, idx.try_into().unwrap());
    }
    pub fn set_level2<T>(&mut self, idx: T)
    where
        T: TryInto<usize>,
        <T as TryInto<usize>>::Error: fmt::Debug,
    {
        self.0.set_bits(config::L2_RANGE, idx.try_into().unwrap());
    }
    pub fn set_level3<T>(&mut self, idx: T)
    where
        T: TryInto<usize>,
        <T as TryInto<usize>>::Error: fmt::Debug,
    {
        self.0.set_bits(config::L3_RANGE, idx.try_into().unwrap());
    }
    pub fn set_offset<T>(&mut self, idx: T)
    where
        T: TryInto<usize>,
        <T as TryInto<usize>>::Error: fmt::Debug,
    {
        self.0
            .set_bits(config::OFFSET_RANGE, idx.try_into().unwrap());
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
            let va0: VirtualAddress = VirtualAddress::try_from(
                0b0000000000000000_000100100_011010001_010110011_110001001_101010111100usize,
            )
            .unwrap();
            assert!(va0.offset() == 0b101010111100);
            assert!(va0.level3() == 0b110001001);
            assert!(va0.level2() == 0b010110011);
            assert!(va0.level1() == 0b011010001);
        }

        {
            let mut va0: VirtualAddress = VirtualAddress::try_from(
                0b0000000000000000_000100100_011010001_010110011_110001001_101010111100usize,
            )
            .unwrap();
            va0.set_offset(0b010101000011);
            assert_eq!(
                va0,
                VirtualAddress::try_from(
                    0b0000000000000000_000100100_011010001_010110011_110001001_010101000011usize
                )
                .unwrap()
            );
        }
        {
            let mut va0: VirtualAddress = VirtualAddress::try_from(
                0b0000000000000000_000100100_011010001_010110011_110001001_101010111100usize,
            )
            .unwrap();
            va0.set_level3(0b011100101);
            assert_eq!(
                va0,
                VirtualAddress::try_from(
                    0b0000000000000000_000100100_011010001_010110011_011100101_101010111100usize
                )
                .unwrap()
            );
        }
        {
            let mut va0: VirtualAddress = VirtualAddress::try_from(
                0b0000000000000000_000100100_011010001_010110011_110001001_101010111100usize,
            )
            .unwrap();
            va0.set_level2(0b011100101);
            assert_eq!(
                va0,
                VirtualAddress::try_from(
                    0b0000000000000000_000100100_011010001_011100101_110001001_101010111100usize
                )
                .unwrap()
            );
        }
        {
            let mut va0: VirtualAddress = VirtualAddress::try_from(
                0b0000000000000000_000100100_011010001_010110011_110001001_101010111100usize,
            )
            .unwrap();
            va0.set_level1(0b011100101);
            assert_eq!(
                va0,
                VirtualAddress::try_from(
                    0b0000000000000000_000100100_011100101_010110011_110001001_101010111100usize
                )
                .unwrap()
            );
        }
        {
            let va0: VirtualAddress = VirtualAddress::try_from(
                0b0000000000000000_000100100_011010001_010110011_110001001_101010111100usize,
            )
            .unwrap();
            assert_eq!(
                va0.to_page(),
                PageNumber::try_from(
                    0b0000000000000000_000100100_011010001_010110011_110001001usize
                )
                .unwrap()
            );
        }
        {
            let last_frame = (0xFFFF_FFFFusize + 1) / 4096 - 1;
            let pa: PhysicalAddress = PhysicalAddress::try_from(0xFFFF_FFFFusize).unwrap();
            assert_eq!(pa.to_frame(), FrameNumber::try_from(last_frame).unwrap());
        }
        {
            let va = VirtualAddress::try_from(0xFFF_000).unwrap();
            assert_eq!(va.is_4K_aligned(), true);
            assert_eq!(va.is_16K_aligned(), false);
            assert_eq!(va.is_64K_aligned(), false);
            assert_eq!(va.is_2M_aligned(), false);
            assert_eq!(va.is_1G_aligned(), false);
        }
        {
            let va = VirtualAddress::try_from(0xC000).unwrap();
            assert_eq!(va.is_4K_aligned(), true);
            assert_eq!(va.is_16K_aligned(), true);
            assert_eq!(va.is_64K_aligned(), false);
            assert_eq!(va.is_2M_aligned(), false);
            assert_eq!(va.is_1G_aligned(), false);
        }
        {
            let va = VirtualAddress::try_from(0x30000).unwrap();
            assert_eq!(va.is_4K_aligned(), true);
            assert_eq!(va.is_16K_aligned(), true);
            assert_eq!(va.is_64K_aligned(), true);
            assert_eq!(va.is_2M_aligned(), false);
            assert_eq!(va.is_1G_aligned(), false);
        }
        {
            let va = VirtualAddress::try_from(0x200000).unwrap();
            assert_eq!(va.is_4K_aligned(), true);
            assert_eq!(va.is_16K_aligned(), true);
            assert_eq!(va.is_64K_aligned(), true);
            assert_eq!(va.is_2M_aligned(), true);
            assert_eq!(va.is_1G_aligned(), false);
        }
        {
            let va = VirtualAddress::try_from(0xC0000000usize).unwrap();
            assert_eq!(va.is_4K_aligned(), true);
            assert_eq!(va.is_16K_aligned(), true);
            assert_eq!(va.is_64K_aligned(), true);
            assert_eq!(va.is_2M_aligned(), true);
            assert_eq!(va.is_1G_aligned(), true);
        }

        {
            PhysicalAddress::try_from(1usize << 48).expect_err("Overflow");
            FrameNumber::try_from((((1usize << 48) - 1) >> config::SHIFT_4K) + 1)
                .expect_err("Overflow");
        }
    }
}
