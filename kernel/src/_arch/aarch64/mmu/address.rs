use super::config;
use crate::{errno::*, utils::bitfields::Bitfields};
use core::{
    convert::{TryFrom, TryInto},
    fmt,
    iter::Iterator,
    ops::{Add, AddAssign, Sub, SubAssign},
};

macro_rules! declare_address {
    ($name:ident, $name_range: ident, $tt:ty  $(,)?) => {
        #[derive(Default, Eq, PartialEq, Ord, PartialOrd, Clone, Copy)]
        #[repr(transparent)]
        pub struct $name($tt);

        #[derive(Clone, Copy, Default)]
        #[repr(C)]
        pub struct $name_range {
            start: $name,
            end: $name,
        }
    };
}

macro_rules! impl_address {
    ($name: ident) => {
        #[allow(non_snake_case, dead_code)]
        impl $name {
            pub const _4K: Self = Self(0x1000);
            pub const _2M: Self = Self(0x200000);
            pub const _1G: Self = Self(0x40000000);
            pub const fn value(&self) -> usize {
                self.0
            }
            pub fn offset(&self) -> usize {
                (self.0 >> config::OFFSET_SHIFT) & config::OFFSET_MASK
            }
            pub fn set_offset<T>(&mut self, offset: T)
            where
                T: TryInto<u64>,
                <T as TryInto<u64>>::Error: fmt::Debug,
            {
                self.0
                    .set_bits(config::OFFSET_RANGE, offset.try_into().unwrap());
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

            pub fn align_to_4K_up(self) -> Self {
                Self(self.0 & config::ALIGN_4K)
            }
            pub fn align_to_16K_up(self) -> Self {
                Self(self.0 & config::ALIGN_16K)
            }
            pub fn align_to_64K_up(self) -> Self {
                Self(self.0 & config::ALIGN_64K)
            }
            pub fn align_to_2M_up(self) -> Self {
                Self(self.0 & config::ALIGN_2M)
            }
            pub fn align_to_1G_up(self) -> Self {
                Self(self.0 & config::ALIGN_1G)
            }

            pub fn align_up(self, alignment: usize) -> Self {
                let align = !(alignment - 1);
                Self(self.0 & align)
            }

            pub fn align_to_4K_down(self) -> Self {
                Self((self.0 + config::MASK_4K) & config::ALIGN_4K)
            }
            pub fn align_to_16K_down(self) -> Self {
                Self((self.0 + config::MASK_16K) & config::ALIGN_16K)
            }
            pub fn align_to_64K_down(self) -> Self {
                Self((self.0 + config::MASK_64K) & config::ALIGN_64K)
            }
            pub fn align_to_2M_down(self) -> Self {
                Self((self.0 + config::MASK_2M) & config::ALIGN_2M)
            }
            pub fn align_to_1G_down(self) -> Self {
                Self((self.0 + config::MASK_1G) & config::ALIGN_1G)
            }
            pub fn align_down(self, alignment: usize) -> Self {
                let align = !(alignment - 1);
                Self((self.0 + alignment - 1) & align)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:#018x}", self.0)
            }
        }
        impl fmt::LowerHex for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:#018x}", self.0)
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
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

// an end-exclusive address range
pub trait AddressRange {
    type Address;
    fn start(&self) -> Self::Address;
    fn end(&self) -> Self::Address;
    fn empty(&self) -> bool;

    // start is aligned down, end is aligned up
    fn align_to_4K(&mut self);
    fn align_to_2M(&mut self);

    fn pop_bytes_front(&mut self, nbytes: usize) -> Option<Self::Address>;

    fn pop_4K_front(&mut self) -> Option<Self::Address>;
    fn pop_2M_front(&mut self) -> Option<Self::Address>;

    fn pop_4K_back(&mut self) -> Option<Self::Address>;
    fn pop_2M_back(&mut self) -> Option<Self::Address>;

    fn pop_4K_at(self, at: Self::Address) -> Option<(Self, Self::Address, Self)>
    where
        Self: Sized;

    fn pop_2M_at(self, addr: Self::Address) -> Option<(Self, Self::Address, Self)>
    where
        Self: Sized;

    fn is_4K_multiple(&self) -> bool;
    fn is_2M_multiple(&self) -> bool;

    fn size_in_bytes(&self) -> usize;
    fn split(self, front_percentage: usize) -> (Self, Self)
    where
        Self: Sized;

    fn merge(&mut self, other: &Self) -> Result<(), ErrorCode>;
    fn len(&self) -> Self::Address;

    fn is_4K(&self) -> bool;
    fn is_2M(&self) -> bool;
    fn set_start(&mut self, s: Self::Address);
    fn set_end(&mut self, s: Self::Address);
}

macro_rules! impl_address_range {
    ($name: ident, $addr: ty) => {
        impl $name {
            pub fn new<T>(start: T, end: T) -> Self
            where
                T: TryInto<$addr>,
                <T as TryInto<$addr>>::Error: fmt::Debug,
            {
                Self {
                    start: T::try_into(start).unwrap(),
                    end: T::try_into(end).unwrap(),
                }
            }

            pub fn count_4K(&self) -> Result<usize, ErrorCode> {
                let diff = self.end - self.start;
                if !diff.is_4K_aligned() {
                    Err(EALIGN)
                } else {
                    Ok(diff.value() / <$addr>::_4K.value())
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}  ->  {}", self.start, self.end)
            }
        }
        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:?}  ->  {:?}", self.start, self.end)
            }
        }

        impl AddressRange for $name {
            type Address = $addr;

            fn start(&self) -> Self::Address {
                self.start
            }
            fn end(&self) -> Self::Address {
                self.end
            }

            fn set_start(&mut self, s: $addr) {
                self.start = s;
            }
            fn set_end(&mut self, e: $addr) {
                self.end = e;
            }

            fn len(&self) -> Self::Address {
                self.end() - self.start()
            }

            fn empty(&self) -> bool {
                self.start == self.end
            }

            // start is aligned down, end is aligned up
            fn align_to_4K(&mut self) {
                self.start = self.start.align_to_4K_down();
                self.end = self.end.align_to_4K_up();
            }
            fn align_to_2M(&mut self) {
                self.start = self.start.align_to_2M_down();
                self.end = self.end.align_to_2M_up();
            }

            fn pop_bytes_front(&mut self, nbytes: usize) -> Option<Self::Address> {
                if (self.end - self.start).value() < nbytes {
                    None
                } else {
                    let popped = self.start;
                    self.start = self.start + Self::Address::try_from(nbytes).unwrap();
                    Some(popped)
                }
            }
            fn pop_4K_front(&mut self) -> Option<Self::Address> {
                if self.end - self.start < Self::Address::_4K {
                    None
                } else {
                    let popped = self.start;
                    self.start = self.start + Self::Address::_4K;
                    Some(popped)
                }
            }
            fn pop_2M_front(&mut self) -> Option<Self::Address> {
                if self.end - self.start < Self::Address::_2M {
                    None
                } else {
                    let popped = self.start;
                    self.start = self.start + Self::Address::_2M;
                    Some(popped)
                }
            }

            fn pop_4K_back(&mut self) -> Option<Self::Address> {
                if self.end - self.start < Self::Address::_4K {
                    None
                } else {
                    self.end = self.end - Self::Address::_4K;
                    let popped = self.end;
                    Some(popped)
                }
            }
            fn pop_2M_back(&mut self) -> Option<Self::Address> {
                if self.end - self.start < Self::Address::_2M {
                    None
                } else {
                    self.end = self.end - Self::Address::_2M;
                    let popped = self.end;
                    Some(popped)
                }
            }

            fn pop_4K_at(self, addr: Self::Address) -> Option<(Self, Self::Address, Self)>
            where
                Self: Sized,
            {
                if addr < self.start {
                    None
                } else if addr + Self::Address::_4K > self.end {
                    None
                } else {
                    Some((
                        Self {
                            start: self.start,
                            end: addr,
                        },
                        addr,
                        Self {
                            start: addr + Self::Address::_4K,
                            end: self.end,
                        },
                    ))
                }
            }
            fn pop_2M_at(self, addr: Self::Address) -> Option<(Self, Self::Address, Self)>
            where
                Self: Sized,
            {
                if addr < self.start {
                    None
                } else if addr + Self::Address::_2M > self.end {
                    None
                } else {
                    Some((
                        Self {
                            start: self.start,
                            end: addr,
                        },
                        addr,
                        Self {
                            start: addr + Self::Address::_2M,
                            end: self.end,
                        },
                    ))
                }
            }
            fn is_4K_multiple(&self) -> bool {
                (self.end - self.start).is_4K_aligned()
            }
            fn is_2M_multiple(&self) -> bool {
                (self.end - self.start).is_2M_aligned()
            }

            fn size_in_bytes(&self) -> usize {
                (self.end - self.start).value()
            }

            fn split(self, front_percentage: usize) -> (Self, Self)
            where
                Self: Sized,
            {
                let front_size = self.size_in_bytes() * front_percentage / 100;
                let front = Self {
                    start: self.start,
                    end: self.start + Self::Address::try_from(front_size).unwrap(),
                };
                let back = Self {
                    start: front.end,
                    end: self.end,
                };
                (front, back)
            }

            fn merge(&mut self, other: &Self) -> Result<(), ErrorCode> {
                if self.end == other.start {
                    self.end = other.end;
                    Ok(())
                } else if other.end == self.start {
                    self.start = other.start;
                    Ok(())
                } else {
                    Err(EINVAL)
                }
            }

            fn is_4K(&self) -> bool {
                self.size_in_bytes() == Self::Address::_4K.value()
            }
            fn is_2M(&self) -> bool {
                self.size_in_bytes() == Self::Address::_2M.value()
            }
        }

        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                self.start == other.start && self.end == other.end
            }
        }

        impl Eq for $name {}
    };
}
declare_address!(VirtualAddress, VaRange, usize);
declare_address!(PhysicalAddress, PaRange, usize);
declare_address!(PageNumber, PageRange, usize);
declare_address!(FrameNumber, FrameRange, usize);

impl_address!(VirtualAddress);
impl_address!(PhysicalAddress);
impl_number!(PageNumber);
impl_number!(FrameNumber);
impl_address_range!(VaRange, VirtualAddress);
impl_address_range!(PaRange, PhysicalAddress);

// 32 bytes
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Mapped {
    pub va: VaRange,
    pub pa: PaRange,
}

impl Mapped {
    pub fn new(va: VaRange, pa: PaRange) -> Self {
        Self { va, pa }
    }
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
impl From<usize> for PhysicalAddress {
    fn from(value: usize) -> Self {
        Self(value)
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
impl PhysicalAddress {
    pub fn to_4K_range(&self) -> PaRange {
        PaRange::new(*self, *self + PhysicalAddress::_4K)
    }
    pub fn to_2M_range(&self) -> PaRange {
        PaRange::new(*self, *self + PhysicalAddress::_2M)
    }
    pub fn to_1G_range(&self) -> PaRange {
        PaRange::new(*self, *self + PhysicalAddress::_1G)
    }
    pub fn to_bytes_range(&self, nbytes: usize) -> PaRange {
        PaRange::new(*self, *self + PhysicalAddress::from(nbytes))
    }
    fn _iter_to(start: Self, step: usize, end: impl Physical) -> Option<AddressIterator<Self>> {
        let end_addr = end.to_address();

        if !start.is_aligned_to(step) || !end_addr.is_aligned_to(step) {
            None
        } else {
            Some(AddressIterator::new(
                start,
                PhysicalAddress::from(step),
                end_addr,
            ))
        }
    }

    fn _iter_for(start: Self, step: usize, n: usize) -> Option<AddressIterator<Self>> {
        let end_addr = start + PhysicalAddress::from(step.checked_mul(n).unwrap());
        if !start.is_aligned_to(step) || !end_addr.is_aligned_to(step) {
            None
        } else {
            Some(AddressIterator::new(
                start,
                PhysicalAddress::from(step),
                end_addr,
            ))
        }
    }

    // end is exclusive
    pub fn iter_4K_to(&self, end: impl Physical) -> Option<AddressIterator<Self>> {
        let end_addr = end.to_address();
        Self::_iter_to(
            self.align_to_4K_up(),
            1 << config::SHIFT_4K,
            end_addr.align_to_4K_down(),
        )
    }
    pub fn iter_16K_to(&self, end: impl Physical) -> Option<AddressIterator<Self>> {
        let end_addr = end.to_address();
        Self::_iter_to(
            self.align_to_16K_up(),
            1 << config::SHIFT_16K,
            end_addr.align_to_16K_down(),
        )
    }
    pub fn iter_64K_to(&self, end: impl Physical) -> Option<AddressIterator<Self>> {
        let end_addr = end.to_address();
        Self::_iter_to(
            self.align_to_64K_up(),
            1 << config::SHIFT_64K,
            end_addr.align_to_64K_down(),
        )
    }
    pub fn iter_2M_to(&self, end: impl Physical) -> Option<AddressIterator<Self>> {
        let end_addr = end.to_address();
        Self::_iter_to(
            self.align_to_2M_up(),
            1 << config::SHIFT_2M,
            end_addr.align_to_2M_down(),
        )
    }
    pub fn iter_1G_to(&self, end: impl Physical) -> Option<AddressIterator<Self>> {
        let end_addr = end.to_address();
        Self::_iter_to(
            self.align_to_1G_up(),
            1 << config::SHIFT_1G,
            end_addr.align_to_1G_down(),
        )
    }

    pub fn iter_to(&self, step: usize, end: impl Physical) -> Option<AddressIterator<Self>> {
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

    pub fn to_4K_range(&self) -> VaRange {
        VaRange::new(*self, *self + VirtualAddress::_4K)
    }
    pub fn to_2M_range(&self) -> VaRange {
        VaRange::new(*self, *self + VirtualAddress::_2M)
    }
    pub fn to_1G_range(&self) -> VaRange {
        VaRange::new(*self, *self + VirtualAddress::_1G)
    }
    pub fn to_bytes_range(&self, nbytes: usize) -> VaRange {
        VaRange::new(*self, *self + VirtualAddress::from(nbytes))
    }

    pub fn set_level1<T>(&mut self, idx: T) -> &mut Self
    where
        T: TryInto<u64>,
        <T as TryInto<u64>>::Error: fmt::Debug,
    {
        self.0.set_bits(config::L1_RANGE, idx.try_into().unwrap());
        self
    }
    pub fn set_level2<T>(&mut self, idx: T) -> &mut Self
    where
        T: TryInto<u64>,
        <T as TryInto<u64>>::Error: fmt::Debug,
    {
        self.0.set_bits(config::L2_RANGE, idx.try_into().unwrap());
        self
    }
    pub fn set_level3<T>(&mut self, idx: T) -> &mut Self
    where
        T: TryInto<u64>,
        <T as TryInto<u64>>::Error: fmt::Debug,
    {
        self.0.set_bits(config::L3_RANGE, idx.try_into().unwrap());
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
    #[allow(unused_imports)]
    use test_macros::kernel_test;

    //#[kernel_test]
    fn test_address_range() {}
}
