use crate::{
    memory::{Granule, TGRAN4K},
    utils::bitfields::Bitfields,
};
use core::marker::PhantomData;

pub trait InputAddress {
    type Input = u64;
    type Output = u64;
    fn get_level3(&self) -> Option<Self::Output>;
    fn get_level2(&self) -> Option<Self::Output>;
    fn get_level1(&self) -> Option<Self::Output>;
    fn get_level0(&self) -> Option<Self::Output>;
    fn get_offset(&self) -> Option<Self::Output>;

    fn set_level3(&mut self, v: Self::Input) -> Option<Self::Output>;
    fn set_level2(&mut self, v: Self::Input) -> Option<Self::Output>;
    fn set_level1(&mut self, v: Self::Input) -> Option<Self::Output>;
    fn set_level0(&mut self, v: Self::Input) -> Option<Self::Output>;
    fn set_offset(&mut self, v: Self::Input) -> Option<Self::Output>;
}

#[derive(Eq, PartialEq, Debug)]
#[repr(transparent)]
pub struct VA<G: Granule>(u64, PhantomData<G>);

impl<G: Granule> From<u64> for VA<G> {
    fn from(addr: u64) -> Self {
        Self(addr, PhantomData)
    }
}

pub type VA4K = VA<TGRAN4K>;

impl InputAddress for VA4K {
    fn get_level0(&self) -> Option<Self::Output> {
        Some(self.0.get_bits(TGRAN4K::LEVEL0))
    }
    fn get_level1(&self) -> Option<Self::Output> {
        Some(self.0.get_bits(TGRAN4K::LEVEL1))
    }
    fn get_level2(&self) -> Option<Self::Output> {
        Some(self.0.get_bits(TGRAN4K::LEVEL2))
    }
    fn get_level3(&self) -> Option<Self::Output> {
        Some(self.0.get_bits(TGRAN4K::LEVEL3))
    }
    fn get_offset(&self) -> Option<Self::Output> {
        Some(self.0.get_bits(TGRAN4K::OFFSET))
    }

    fn set_level0(&mut self, v: Self::Input) -> Option<Self::Output> {
        if (v & TGRAN4K::LEVEL_MASK) != v {
            return None;
        }

        self.0.set_bits(TGRAN4K::LEVEL0, v);
        Some(self.0)
    }
    fn set_level1(&mut self, v: Self::Input) -> Option<Self::Output> {
        if (v & TGRAN4K::LEVEL_MASK) != v {
            return None;
        }

        self.0.set_bits(TGRAN4K::LEVEL1, v);
        Some(self.0)
    }
    fn set_level2(&mut self, v: Self::Input) -> Option<Self::Output> {
        if (v & TGRAN4K::LEVEL_MASK) != v {
            return None;
        }

        self.0.set_bits(TGRAN4K::LEVEL2, v);
        Some(self.0)
    }
    fn set_level3(&mut self, v: Self::Input) -> Option<Self::Output> {
        if (v & TGRAN4K::LEVEL_MASK) != v {
            return None;
        }

        self.0.set_bits(TGRAN4K::LEVEL3, v);
        Some(self.0)
    }
    fn set_offset(&mut self, v: Self::Input) -> Option<Self::Output> {
        if (v & TGRAN4K::OFFSET_MASK) != v {
            return None;
        }

        self.0.set_bits(TGRAN4K::OFFSET, v);
        Some(self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use crate::println_qemu;
    use test_macros::kernel_test;

    #[kernel_test]
    fn test_input_address_index() {
        {
            let va0: VA4K =
                VA::from(0b0000000000000000_000100100_011010001_010110011_110001001_101010111100);
            assert!(va0.get_offset().unwrap() == 0b101010111100);
            assert!(va0.get_level3().unwrap() == 0b110001001);
            assert!(va0.get_level2().unwrap() == 0b010110011);
            assert!(va0.get_level1().unwrap() == 0b011010001);
            assert!(va0.get_level0().unwrap() == 0b000100100);
        }

        {
            let mut va0: VA4K =
                VA::from(0b0000000000000000_000100100_011010001_010110011_110001001_101010111100);
            assert_eq!(
                va0.set_offset(0b010101000011).unwrap(),
                0b0000000000000000_000100100_011010001_010110011_110001001_010101000011
            );
        }
        {
            let mut va0: VA4K =
                VA::from(0b0000000000000000_000100100_011010001_010110011_110001001_101010111100);
            assert_eq!(
                va0.set_level3(0b011100101).unwrap(),
                0b0000000000000000_000100100_011010001_010110011_011100101_101010111100
            );
        }
        {
            let mut va0: VA4K =
                VA::from(0b0000000000000000_000100100_011010001_010110011_110001001_101010111100);
            assert_eq!(
                va0.set_level2(0b011100101).unwrap(),
                0b0000000000000000_000100100_011010001_011100101_110001001_101010111100
            );
        }
        {
            let mut va0: VA4K =
                VA::from(0b0000000000000000_000100100_011010001_010110011_110001001_101010111100);
            assert_eq!(
                va0.set_level1(0b011100101).unwrap(),
                0b0000000000000000_000100100_011100101_010110011_110001001_101010111100
            );
        }
        {
            let mut va0: VA4K =
                VA::from(0b0000000000000000_000100100_011010001_010110011_110001001_101010111100);
            assert_eq!(
                va0.set_level0(0b011100101).unwrap(),
                0b0000000000000000_011100101_011010001_010110011_110001001_101010111100
            );
        }
    }
}
