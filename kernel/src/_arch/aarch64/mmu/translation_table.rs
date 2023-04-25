pub struct TGRAN4K;
pub struct TGRAN16K;
pub struct TGRAN64K;

pub trait Granule {
    const ENTRIES: usize;
    const BITS_RESOLVED: u8;
    const SHIFT: u8;
    const MASK: u64;
}

impl Granule for TGRAN4K {
    const ENTRIES: usize = 512;
    const BITS_RESOLVED: u8 = 9;
    const SHIFT: u8 = 12;
    const MASK: u64 = (0xFFFF_FFFF_FFFF_FFFF >> Self::SHIFT) << Self::SHIFT;
}

impl Granule for TGRAN16K {
    const ENTRIES: usize = 2048;
    const BITS_RESOLVED: u8 = 11;
    const SHIFT: u8 = 14;
    const MASK: u64 = (0xFFFF_FFFF_FFFF_FFFF >> Self::SHIFT) << Self::SHIFT;
}

impl Granule for TGRAN64K {
    const ENTRIES: usize = 8192;
    const BITS_RESOLVED: u8 = 13;
    const SHIFT: u8 = 16;
    const MASK: u64 = (0xFFFF_FFFF_FFFF_FFFF >> Self::SHIFT) << Self::SHIFT;
}

pub struct PageDescriptor;
pub struct BlockDescriptor;
pub struct TableDescriptor;
