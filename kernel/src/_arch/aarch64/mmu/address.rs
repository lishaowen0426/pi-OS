use super::config;
use crate::{errno::*, utils::bitfields::Bitfields};
use core::{
    convert::{TryFrom, TryInto},
    fmt,
    iter::Iterator,
    ops::{Add, AddAssign, Sub, SubAssign},
};

#[allow(non_snake_case, dead_code)]
pub mod AddressEdit {
    use super::config;

    pub fn is_4K_aligned(addr: usize) -> bool {
        (addr & config::ALIGN_4K) == addr
    }
    pub fn is_16K_aligned(addr: usize) -> bool {
        (addr & config::ALIGN_16K) == addr
    }
    pub fn is_64K_aligned(addr: usize) -> bool {
        (addr & config::ALIGN_64K) == addr
    }
    pub fn is_2M_aligned(addr: usize) -> bool {
        (addr & config::ALIGN_2M) == addr
    }
    pub fn is_1G_aligned(addr: usize) -> bool {
        (addr & config::ALIGN_1G) == addr
    }
    pub fn is_aligned_to(addr: usize, alignment: usize) -> bool {
        (addr & (!(alignment - 1))) == addr
    }

    pub fn shift_4K(addr: usize) -> usize {
        addr >> config::SHIFT_4K
    }
    pub fn shift_16K(addr: usize) -> usize {
        addr >> config::SHIFT_16K
    }
    pub fn shift_64K(addr: usize) -> usize {
        addr >> config::SHIFT_64K
    }
    pub fn shift_2M(addr: usize) -> usize {
        addr >> config::SHIFT_2M
    }
    pub fn shift_1G(addr: usize) -> usize {
        addr >> config::SHIFT_1G
    }

    pub fn align_to_4K(addr: usize) -> usize {
        addr & config::ALIGN_4K
    }
    pub fn align_to_16K(addr: usize) -> usize {
        addr & config::ALIGN_16K
    }
    pub fn align_to_64K(addr: usize) -> usize {
        addr & config::ALIGN_64K
    }
    pub fn align_to_2M(addr: usize) -> usize {
        addr & config::ALIGN_2M
    }
    pub fn align_to_1G(addr: usize) -> usize {
        addr & config::ALIGN_1G
    }
}
macro_rules! declare_address {
    ($name:ident, $tt:ty, $lit: literal $(,)?) => {
        #[derive(Default, Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Copy)]
        #[repr(transparent)]
        pub struct $name(pub $tt);

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

        impl Add for $name {
            type Output = Self;
            fn add(self, other: Self) -> Self {
                Self::try_from(self.0.checked_add(other.0).unwrap()).unwrap()
            }
        }
        impl Sub for $name {
            type Output = Self;
            fn sub(self, other: Self) -> Self {
                Self::try_from(self.0.checked_sub(other.0).unwrap()).unwrap()
            }
        }

        impl AddAssign for $name {
            fn add_assign(&mut self, rhs: Self) {
                self.0 = self.0.checked_add(rhs.0).unwrap();
            }
        }
        impl SubAssign for $name {
            fn sub_assign(&mut self, rhs: Self) {
                self.0 = self.0.checked_sub(rhs.0).unwrap();
            }
        }
    };
}

declare_address!(VirtualAddress, usize, "{:#018x}");
declare_address!(PhysicalAddress, usize, "{:#018x}");
declare_address!(PageNumber, usize, "{}");
declare_address!(FrameNumber, usize, "{}");

// end is exclusive
#[derive(Debug)]
pub struct AddressIterator<T> {
    current: T,
    step: T,
    end: T,
}

impl<T> AddressIterator<T>
where
    T: Copy,
{
    fn new(current: T, step: T, end: T) -> Self {
        Self { current, step, end }
    }
}

impl<T> Iterator for AddressIterator<T>
where
    T: PartialOrd + AddAssign + Copy,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.end {
            let p = self.current;
            self.current += self.step;
            Some(p)
        } else {
            None
        }
    }
}

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

#[allow(non_snake_case)]
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

    pub fn set_level1<T>(&mut self, idx: T) -> &mut Self
    where
        T: TryInto<usize>,
        <T as TryInto<usize>>::Error: fmt::Debug,
    {
        self.0.set_bits(config::L1_RANGE, idx.try_into().unwrap());
        self
    }
    pub fn set_level2<T>(&mut self, idx: T) -> &mut Self
    where
        T: TryInto<usize>,
        <T as TryInto<usize>>::Error: fmt::Debug,
    {
        self.0.set_bits(config::L2_RANGE, idx.try_into().unwrap());
        self
    }
    pub fn set_level3<T>(&mut self, idx: T) -> &mut Self
    where
        T: TryInto<usize>,
        <T as TryInto<usize>>::Error: fmt::Debug,
    {
        self.0.set_bits(config::L3_RANGE, idx.try_into().unwrap());
        self
    }
    pub fn set_offset<T>(&mut self, idx: T) -> &mut Self
    where
        T: TryInto<usize>,
        <T as TryInto<usize>>::Error: fmt::Debug,
    {
        self.0
            .set_bits(config::OFFSET_RANGE, idx.try_into().unwrap());
        self
    }

    fn iter_to(start: Self, step: usize, end: impl Virtual) -> Option<AddressIterator<Self>> {
        let end_addr = end.to_address();

        if !AddressEdit::is_aligned_to(start.0, step)
            || !AddressEdit::is_aligned_to(end_addr.0, step)
        {
            None
        } else {
            Some(AddressIterator::new(
                start,
                VirtualAddress::try_from(step).unwrap(),
                end_addr,
            ))
        }
    }

    fn iter_for(start: Self, step: usize, n: usize) -> Option<AddressIterator<Self>> {
        let end_addr = start + VirtualAddress::try_from(step.checked_mul(n).unwrap()).unwrap();
        if !AddressEdit::is_aligned_to(start.0, step)
            || !AddressEdit::is_aligned_to(end_addr.0, step)
        {
            None
        } else {
            Some(AddressIterator::new(
                start,
                VirtualAddress::try_from(step).unwrap(),
                end_addr,
            ))
        }
    }

    // end is exclusive
    pub fn iter_4K_to(&self, end: impl Virtual) -> Option<AddressIterator<Self>> {
        let end_addr = end.to_address();
        Self::iter_to(
            VirtualAddress::try_from(AddressEdit::align_to_4K(self.0)).unwrap(),
            1 << config::SHIFT_4K,
            VirtualAddress::try_from(AddressEdit::align_to_4K(end_addr.0)).unwrap(),
        )
    }
    pub fn iter_16K_to(&self, end: impl Virtual) -> Option<AddressIterator<Self>> {
        let end_addr = end.to_address();
        Self::iter_to(
            VirtualAddress::try_from(AddressEdit::align_to_16K(self.0)).unwrap(),
            1 << config::SHIFT_16K,
            VirtualAddress::try_from(AddressEdit::align_to_16K(end_addr.0)).unwrap(),
        )
    }
    pub fn iter_64K_to(&self, end: impl Virtual) -> Option<AddressIterator<Self>> {
        let end_addr = end.to_address();
        Self::iter_to(
            VirtualAddress::try_from(AddressEdit::align_to_64K(self.0)).unwrap(),
            1 << config::SHIFT_64K,
            VirtualAddress::try_from(AddressEdit::align_to_64K(end_addr.0)).unwrap(),
        )
    }
    pub fn iter_2M_to(&self, end: impl Virtual) -> Option<AddressIterator<Self>> {
        let end_addr = end.to_address();
        Self::iter_to(
            VirtualAddress::try_from(AddressEdit::align_to_2M(self.0)).unwrap(),
            1 << config::SHIFT_2M,
            VirtualAddress::try_from(AddressEdit::align_to_2M(end_addr.0)).unwrap(),
        )
    }
    pub fn iter_1G_to(&self, end: impl Virtual) -> Option<AddressIterator<Self>> {
        let end_addr = end.to_address();
        Self::iter_to(
            VirtualAddress::try_from(AddressEdit::align_to_1G(self.0)).unwrap(),
            1 << config::SHIFT_1G,
            VirtualAddress::try_from(AddressEdit::align_to_1G(end_addr.0)).unwrap(),
        )
    }

    pub fn iter_4K_for(&self, n: usize) -> Option<AddressIterator<Self>> {
        Self::iter_for(
            VirtualAddress::try_from(AddressEdit::align_to_4K(self.0)).unwrap(),
            1 << config::SHIFT_4K,
            n,
        )
    }
    pub fn iter_16K_for(&self, n: usize) -> Option<AddressIterator<Self>> {
        Self::iter_for(
            VirtualAddress::try_from(AddressEdit::align_to_16K(self.0)).unwrap(),
            1 << config::SHIFT_16K,
            n,
        )
    }
    pub fn iter_64K_for(&self, n: usize) -> Option<AddressIterator<Self>> {
        Self::iter_for(
            VirtualAddress::try_from(AddressEdit::align_to_64K(self.0)).unwrap(),
            1 << config::SHIFT_64K,
            n,
        )
    }
    pub fn iter_2M_for(&self, n: usize) -> Option<AddressIterator<Self>> {
        Self::iter_for(
            VirtualAddress::try_from(AddressEdit::align_to_2M(self.0)).unwrap(),
            1 << config::SHIFT_2M,
            n,
        )
    }
    pub fn iter_1G_for(&self, n: usize) -> Option<AddressIterator<Self>> {
        Self::iter_for(
            VirtualAddress::try_from(AddressEdit::align_to_1G(self.0)).unwrap(),
            1 << config::SHIFT_1G,
            n,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bsp;
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
            PhysicalAddress::try_from(1usize << 48).expect_err("Overflow");
            FrameNumber::try_from((((1usize << 48) - 1) >> config::SHIFT_4K) + 1)
                .expect_err("Overflow");
        }
    }
    #[kernel_test]
    fn test_address_iterator() {
        let end = VirtualAddress::try_from(0xFFF_FFFFusize).unwrap();
        {
            let start = VirtualAddress::try_from(0usize).unwrap();
            for (i, v) in start.iter_4K_to(end).unwrap().enumerate() {
                assert_eq!(
                    v,
                    VirtualAddress::try_from(i * (1 << config::SHIFT_4K)).unwrap()
                )
            }
        }
        {
            let start = VirtualAddress::try_from(0usize).unwrap();
            for (i, v) in start.iter_16K_to(end).unwrap().enumerate() {
                assert_eq!(
                    v,
                    VirtualAddress::try_from(i * (1 << config::SHIFT_16K)).unwrap()
                )
            }
        }
        {
            let start = VirtualAddress::try_from(0usize).unwrap();
            for (i, v) in start.iter_64K_to(end).unwrap().enumerate() {
                assert_eq!(
                    v,
                    VirtualAddress::try_from(i * (1 << config::SHIFT_64K)).unwrap()
                )
            }
        }
        {
            let start = VirtualAddress::try_from(0usize).unwrap();
            for (i, v) in start.iter_2M_to(end).unwrap().enumerate() {
                assert_eq!(
                    v,
                    VirtualAddress::try_from(i * (1 << config::SHIFT_2M)).unwrap()
                )
            }
        }
        {
            let start = VirtualAddress::try_from(0usize).unwrap();
            for (i, v) in start.iter_1G_to(end).unwrap().enumerate() {
                assert_eq!(
                    v,
                    VirtualAddress::try_from(i * (1 << config::SHIFT_1G)).unwrap()
                )
            }
        }
        {
            let start = VirtualAddress::try_from(0xFFFBC69usize).unwrap();
            let n = 100;
            for (i, v) in start.iter_1G_for(n).unwrap().enumerate() {
                assert_eq!(
                    v,
                    VirtualAddress::try_from(i * (config::MASK_1G + 1)).unwrap()
                )
            }
        }
        {
            let start = VirtualAddress::try_from(0xFFFBC69usize).unwrap();
            let n = 100;
            for (i, v) in start.iter_4K_for(n).unwrap().enumerate() {
                assert_eq!(
                    v,
                    VirtualAddress::try_from(
                        (0xFFFBC69usize & config::ALIGN_4K) + i * (config::MASK_4K + 1)
                    )
                    .unwrap()
                )
            }
        }
        {
            let start = VirtualAddress::try_from(0usize).unwrap();
            for (_, _) in start.iter_4K_for(0).unwrap().enumerate() {
                panic!()
            }
            for (_, _) in start
                .iter_1G_to(VirtualAddress::try_from(bsp::mmio::PERIPHERAL_START).unwrap())
                .unwrap()
                .enumerate()
            {
                panic!()
            }
        }
    }
}
