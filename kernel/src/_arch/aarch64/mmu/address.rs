use super::config;
use crate::{errno::*, println, utils::bitfields::Bitfields};
use core::{
    convert::{TryFrom, TryInto},
    fmt,
    iter::Iterator,
    ops::{Add, AddAssign, Sub, SubAssign},
};

macro_rules! declare_address {
    ($name:ident, $name_range: ident, $tt:ty, $lit: literal $(,)?) => {
        #[derive(Default, Eq, PartialEq, Ord, PartialOrd, Clone, Copy)]
        #[repr(transparent)]
        pub struct $name($tt);

        #[repr(C)]
        pub struct $name_range {
            start: $name,
            end: $name,
        }

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
        impl fmt::Display for $name_range {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}  ->  {}", self.start, self.end)
            }
        }
        impl fmt::Debug for $name_range {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:?}  ->  {:?}", self.start, self.end)
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
                *self = Self::try_from(self.0.checked_add(rhs.0).unwrap()).unwrap();
            }
        }
        impl SubAssign for $name {
            fn sub_assign(&mut self, rhs: Self) {
                *self = Self::try_from(self.0.checked_sub(rhs.0).unwrap()).unwrap();
            }
        }
    };
}

macro_rules! impl_address {
    ($name: ident) => {
        #[allow(non_snake_case, dead_code)]
        impl $name {
            pub fn value(&self) -> usize {
                self.0
            }
            pub fn offset(&self) -> usize {
                (self.0 >> config::OFFSET_SHIFT) & config::OFFSET_MASK
            }
            pub fn set_offset<T>(&self, offset: T) -> Self
            where
                T: TryInto<usize>,
                <T as TryInto<usize>>::Error: fmt::Debug,
            {
                Self(
                    self.0
                        .set_bits(config::OFFSET_RANGE, offset.try_into().unwrap()),
                )
            }

            pub fn as_const_ptr<T>(&self) -> *const T {
                self.0 as *const T
            }
            pub fn as_mut_ptr<T>(&self) -> *mut T {
                self.0 as *mut T
            }

            pub fn is_4K_aligned(&self) -> bool {
                (self.0 & config::ALIGN_4K) == self.0
            }
            pub fn is_16K_aligned(&self) -> bool {
                (self.0 & config::ALIGN_16K) == self.0
            }
            pub fn is_64K_aligned(&self) -> bool {
                (self.0 & config::ALIGN_64K) == self.0
            }
            pub fn is_2M_aligned(&self) -> bool {
                (self.0 & config::ALIGN_2M) == self.0
            }
            pub fn is_1G_aligned(&self) -> bool {
                (self.0 & config::ALIGN_1G) == self.0
            }
            pub fn is_aligned_to(&self, alignment: usize) -> bool {
                (self.0 & (!(alignment - 1))) == self.0
            }

            pub fn shift_4K(&self) -> Self {
                Self(self.0 >> config::SHIFT_4K)
            }
            pub fn shift_16K(&self) -> Self {
                Self(self.0 >> config::SHIFT_16K)
            }
            pub fn shift_64K(&self) -> Self {
                Self(self.0 >> config::SHIFT_64K)
            }
            pub fn shift_2M(&self) -> Self {
                Self(self.0 >> config::SHIFT_2M)
            }
            pub fn shift_1G(&self) -> Self {
                Self(self.0 >> config::SHIFT_1G)
            }

            pub fn align_to_4K_up(&self) -> Self {
                Self(self.0 & config::ALIGN_4K)
            }
            pub fn align_to_16K_up(&self) -> Self {
                Self(self.0 & config::ALIGN_16K)
            }
            pub fn align_to_64K_up(&self) -> Self {
                Self(self.0 & config::ALIGN_64K)
            }
            pub fn align_to_2M_up(&self) -> Self {
                Self(self.0 & config::ALIGN_2M)
            }
            pub fn align_to_1G_up(&self) -> Self {
                Self(self.0 & config::ALIGN_1G)
            }

            pub fn align_up(&self, alignment: usize) -> Self {
                let align = !(alignment - 1);
                Self(self.0 & align)
            }

            pub fn align_to_4K_down(&self) -> Self {
                Self((self.0 + config::MASK_4K) & config::ALIGN_4K)
            }
            pub fn align_to_16K_down(&self) -> Self {
                Self((self.0 + config::MASK_16K) & config::ALIGN_16K)
            }
            pub fn align_to_64K_down(&self) -> Self {
                Self((self.0 + config::MASK_64K) & config::ALIGN_64K)
            }
            pub fn align_to_2M_down(&self) -> Self {
                Self((self.0 + config::MASK_2M) & config::ALIGN_2M)
            }
            pub fn align_to_1G_down(&self) -> Self {
                Self((self.0 + config::MASK_1G) & config::ALIGN_1G)
            }
            pub fn align_down(&self, alignment: usize) -> Self {
                let align = !(alignment - 1);
                Self((self.0 + alignment - 1) & align)
            }
        }
    };
}

macro_rules! impl_number {
    ($name: ident) => {
        impl $name {
            pub fn value(&self) -> usize {
                self.0
            }

            pub fn next(&mut self) -> Option<Self> {
                let copy = *self;
                if let Ok(n) = Self::try_from(self.0 + 1) {
                    *self = n;
                    Some(copy)
                } else {
                    None
                }
            }
        }
    };
}
declare_address!(VirtualAddress, VaRange, usize, "{:#018x}");
declare_address!(PhysicalAddress, PaRange, usize, "{:#018x}");
declare_address!(PageNumber, PageRange, usize, "{}");
declare_address!(FrameNumber, FrameRange, usize, "{}");

impl_address!(VirtualAddress);
impl_address!(PhysicalAddress);
impl_number!(PageNumber);
impl_number!(FrameNumber);

// 32 bytes
#[repr(C)]
pub struct Mapped {
    va: VaRange,
    pa: PaRange,
}

impl fmt::Display for Mapped {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "va {} is mapped to pa {}", self.va, self.pa)
    }
}

impl fmt::Debug for Mapped {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "va {:?} is mapped to pa {:?}", self.va, self.pa)
    }
}

impl fmt::Debug for VirtualAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "VA = {:#018x}, L1[{}], L2[{}], L3[{}]",
            self.0,
            self.level1(),
            self.level2(),
            self.level3()
        )
    }
}
impl fmt::Debug for PhysicalAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PA = {:#018x}, frame = {}", self.0, self.to_frame(),)
    }
}
impl fmt::Debug for PageNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}  ", self.to_address())
    }
}
impl fmt::Debug for FrameNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}  ", self.to_address())
    }
}

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

impl From<usize> for VirtualAddress {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl TryFrom<usize> for PhysicalAddress {
    type Error = ErrorCode;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value > config::PHYSICAL_MEMORY_END_EXCLUSIVE {
            Err(EOVERFLOW)
        } else {
            Ok(Self(value))
        }
    }
}

impl TryFrom<usize> for PageNumber {
    type Error = ErrorCode;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value > config::NUMBER_OF_PAGES {
            Err(EOVERFLOW)
        } else {
            Ok(Self(value))
        }
    }
}

impl TryFrom<usize> for FrameNumber {
    type Error = ErrorCode;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value > config::NUMBER_OF_FRAMES {
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
        VirtualAddress::from(self.0 << config::SHIFT_4K)
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

    pub fn set_level1<T>(&mut self, idx: T) -> &mut Self
    where
        T: TryInto<usize>,
        <T as TryInto<usize>>::Error: fmt::Debug,
    {
        self.0 = self.0.set_bits(config::L1_RANGE, idx.try_into().unwrap());
        self
    }
    pub fn set_level2<T>(&mut self, idx: T) -> &mut Self
    where
        T: TryInto<usize>,
        <T as TryInto<usize>>::Error: fmt::Debug,
    {
        self.0 = self.0.set_bits(config::L2_RANGE, idx.try_into().unwrap());
        self
    }
    pub fn set_level3<T>(&mut self, idx: T) -> &mut Self
    where
        T: TryInto<usize>,
        <T as TryInto<usize>>::Error: fmt::Debug,
    {
        self.0 = self.0.set_bits(config::L3_RANGE, idx.try_into().unwrap());
        self
    }

    pub fn is_lower(&self) -> bool {
        (self.0 >> 48) == 0
    }
    pub fn is_higher(&self) -> bool {
        (self.0 >> 48) == 0xFFFF
    }

    fn _iter_to(start: Self, step: usize, end: impl Virtual) -> Option<AddressIterator<Self>> {
        let end_addr = end.to_address();

        if !start.is_aligned_to(step) || !end_addr.is_aligned_to(step) {
            None
        } else {
            Some(AddressIterator::new(
                start,
                VirtualAddress::from(step),
                end_addr,
            ))
        }
    }

    fn _iter_for(start: Self, step: usize, n: usize) -> Option<AddressIterator<Self>> {
        let end_addr = start + VirtualAddress::from(step.checked_mul(n).unwrap());
        if !start.is_aligned_to(step) || !end_addr.is_aligned_to(step) {
            None
        } else {
            Some(AddressIterator::new(
                start,
                VirtualAddress::from(step),
                end_addr,
            ))
        }
    }

    // end is exclusive
    pub fn iter_4K_to(&self, end: impl Virtual) -> Option<AddressIterator<Self>> {
        let end_addr = end.to_address();
        Self::_iter_to(
            self.align_to_4K_up(),
            1 << config::SHIFT_4K,
            end_addr.align_to_4K_down(),
        )
    }
    pub fn iter_16K_to(&self, end: impl Virtual) -> Option<AddressIterator<Self>> {
        let end_addr = end.to_address();
        Self::_iter_to(
            self.align_to_16K_up(),
            1 << config::SHIFT_16K,
            end_addr.align_to_16K_down(),
        )
    }
    pub fn iter_64K_to(&self, end: impl Virtual) -> Option<AddressIterator<Self>> {
        let end_addr = end.to_address();
        Self::_iter_to(
            self.align_to_64K_up(),
            1 << config::SHIFT_64K,
            end_addr.align_to_64K_down(),
        )
    }
    pub fn iter_2M_to(&self, end: impl Virtual) -> Option<AddressIterator<Self>> {
        let end_addr = end.to_address();
        Self::_iter_to(
            self.align_to_2M_up(),
            1 << config::SHIFT_2M,
            end_addr.align_to_2M_down(),
        )
    }
    pub fn iter_1G_to(&self, end: impl Virtual) -> Option<AddressIterator<Self>> {
        let end_addr = end.to_address();
        Self::_iter_to(
            self.align_to_1G_up(),
            1 << config::SHIFT_1G,
            end_addr.align_to_1G_down(),
        )
    }

    pub fn iter_to(&self, step: usize, end: impl Virtual) -> Option<AddressIterator<Self>> {
        let end_addr = end.to_address();
        Self::_iter_to(self.align_up(step), step, end_addr.align_down(step))
    }

    pub fn iter_4K_for(&self, n: usize) -> Option<AddressIterator<Self>> {
        Self::_iter_for(self.align_to_4K_up(), 1 << config::SHIFT_4K, n)
    }
    pub fn iter_16K_for(&self, n: usize) -> Option<AddressIterator<Self>> {
        Self::_iter_for(self.align_to_16K_up(), 1 << config::SHIFT_16K, n)
    }
    pub fn iter_64K_for(&self, n: usize) -> Option<AddressIterator<Self>> {
        Self::_iter_for(self.align_to_64K_up(), 1 << config::SHIFT_64K, n)
    }
    pub fn iter_2M_for(&self, n: usize) -> Option<AddressIterator<Self>> {
        Self::_iter_for(self.align_to_2M_up(), 1 << config::SHIFT_2M, n)
    }
    pub fn iter_1G_for(&self, n: usize) -> Option<AddressIterator<Self>> {
        Self::_iter_for(self.align_to_1G_up(), 1 << config::SHIFT_1G, n)
    }
    pub fn iter_for(&self, step: usize, n: usize) -> Option<AddressIterator<Self>> {
        Self::_iter_for(self.align_up(step), step, n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bsp;
    #[allow(unused_imports)]
    use test_macros::kernel_test;
}
