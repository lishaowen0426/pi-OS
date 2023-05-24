use super::{address::*, config};
use crate::{errno::*, unsafe_println, utils::bitfields::Bitfields};
use core::{fmt, marker::PhantomData, ops::Range};

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

pub type L1Entry = TranslationTableEntry<Level1>;
pub type L2Entry = TranslationTableEntry<Level2>;
pub type L3Entry = TranslationTableEntry<Level3>;

#[derive(Copy, Clone)]
pub enum Descriptor {
    L1BlockEntry(u64),
    L2BlockEntry(u64),
    TableEntry(u64),
    PageEntry(u64),
    INVALID,
}

#[derive(Copy, Clone)]
pub enum MemoryType {
    RwNormal,
    RoNormal,
    XNormal,
    RWXNormal,
    RwDevice,
    RoDevice,
    Table,
    INVALID,
}

impl fmt::Display for MemoryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::RwNormal => write!(f, "rwrite normal"),
            Self::RoNormal => write!(f, "ronly normal"),
            Self::XNormal => write!(f, "executable normal"),
            Self::RWXNormal => write!(f, "rwexecutable normal(only for debug)"),
            Self::RwDevice => write!(f, "rwrite device"),
            Self::RoDevice => write!(f, "ronly device"),
            Self::Table => write!(f, "next-level table"),
            Self::INVALID => write!(f, "invalid page"),
        }
    }
}

impl fmt::Debug for MemoryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::RwNormal => write!(f, "rwrite normal"),
            Self::RoNormal => write!(f, "ronly normal"),
            Self::XNormal => write!(f, "executable normal"),
            Self::RWXNormal => write!(f, "rwexecutable normal(only for debug)"),
            Self::RwDevice => write!(f, "rwrite device"),
            Self::RoDevice => write!(f, "ronly device"),
            Self::Table => write!(f, "next-level table"),
            Self::INVALID => write!(f, "invalid page"),
        }
    }
}

pub static RWNORMAL: &MemoryType = &MemoryType::RwNormal;
pub static RONORMAL: &MemoryType = &MemoryType::RoNormal;
pub static XNORMAL: &MemoryType = &MemoryType::XNormal;
pub static RWXNORMAL: &MemoryType = &MemoryType::RWXNormal;
pub static RWDEVICE: &MemoryType = &MemoryType::RwDevice;
pub static RODEVICE: &MemoryType = &MemoryType::RoDevice;
pub static TABLE_PAGE: &MemoryType = &MemoryType::Table;
pub static INVALID_PAGE: &MemoryType = &MemoryType::INVALID;

#[allow(non_snake_case, dead_code, non_upper_case_globals)]
impl Descriptor {
    pub const VALID: usize = 0;
    pub const NSTable: usize = 63;
    pub const APTable: Range<usize> = 61..63;
    pub const UXNTable: usize = 60;
    pub const PXNTable: usize = 59;
    pub const UXN: usize = 54;
    pub const PXN: usize = 53;
    pub const Contiguous: usize = 52;
    pub const nG: usize = 11;
    pub const AF: usize = 10;
    pub const SH: Range<usize> = 8..10;
    pub const AP: Range<usize> = 6..8;
    pub const NS: usize = 5;
    pub const AttrIndx: Range<usize> = 2..5;
    pub const RW_NORMAL: u64 = Self::RW_normal();
    pub const RO_NORMAL: u64 = Self::RO_normal();
    pub const X_NORMAL: u64 = Self::X_normal();
    pub const RWX_NORMAL: u64 = Self::RWX_normal();
    pub const RW_DEVICE: u64 = Self::RW_device();
    pub const RO_DEVICE: u64 = Self::RO_device();
    pub const TABLE_ATTR: u64 = Self::Table_attr();

    pub const BLOCK_PAGE_ATTR_MASK: u64 =
        (0b111u64 << Self::Contiguous) | (0b1111111111u64 << Self::AttrIndx.start);
    pub const TABLE_ATTR_MASK: u64 = (0b11111u64 << Self::PXNTable) | (0b1 << Self::AF);

    pub fn get_attributes(&self) -> &MemoryType {
        match *self {
            Self::INVALID => INVALID_PAGE,
            Self::L1BlockEntry(e) | Self::L2BlockEntry(e) | Self::PageEntry(e) => {
                let attr = e & Self::BLOCK_PAGE_ATTR_MASK;
                if attr == Self::RW_NORMAL {
                    RWNORMAL
                } else if attr == Self::RO_NORMAL {
                    RONORMAL
                } else if attr == Self::X_NORMAL {
                    XNORMAL
                } else if attr == Self::RW_DEVICE {
                    RWDEVICE
                } else if attr == Self::RO_DEVICE {
                    RODEVICE
                } else if attr == Self::RWX_NORMAL {
                    RWXNORMAL
                } else {
                    INVALID_PAGE
                }
            }
            Self::TableEntry(e) => {
                let attr = e & Self::TABLE_ATTR_MASK;
                if attr == Self::TABLE_ATTR {
                    TABLE_PAGE
                } else {
                    INVALID_PAGE
                }
            }
        }
    }

    pub fn set_l1_block(self) -> Result<Self, ErrorCode> {
        match self {
            Self::INVALID => Ok(Self::L1BlockEntry(0b01)),
            _ => Err(EINVAL),
        }
    }
    pub fn set_l2_block(self) -> Result<Self, ErrorCode> {
        match self {
            Self::INVALID => Ok(Self::L2BlockEntry(0b01)),
            _ => Err(EINVAL),
        }
    }
    pub fn set_table(self) -> Result<Self, ErrorCode> {
        match self {
            Self::INVALID => Ok(Self::TableEntry(0b11)),
            _ => Err(EINVAL),
        }
    }
    pub fn set_page(self) -> Result<Self, ErrorCode> {
        match self {
            Self::INVALID => Ok(Self::PageEntry(0b11)),
            _ => Err(EINVAL),
        }
    }

    pub unsafe fn table_to_page(self) -> Result<Self, ErrorCode> {
        match self {
            Self::TableEntry(e) => Ok(Self::PageEntry(e)),
            _ => Err(EINVAL),
        }
    }

    pub fn get_NSTable(&self) -> Option<u8> {
        match *self {
            Self::TableEntry(e) => Some(e.get_bit(Self::NSTable) as u8),
            _ => None,
        }
    }

    pub fn get_APTable(&self) -> Option<u8> {
        match *self {
            Self::TableEntry(e) => Some(e.get_bits(Self::APTable) as u8),
            _ => None,
        }
    }
    pub fn get_UXNTable(&self) -> Option<u8> {
        match *self {
            Self::TableEntry(e) => Some(e.get_bit(Self::UXNTable) as u8),
            _ => None,
        }
    }
    pub fn get_PXNTable(&self) -> Option<u8> {
        match *self {
            Self::TableEntry(e) => Some(e.get_bit(Self::PXNTable) as u8),
            _ => None,
        }
    }
    pub fn get_UXN(&self) -> Option<u8> {
        match *self {
            Self::L1BlockEntry(e) => Some(e.get_bit(Self::UXN) as u8),
            Self::L2BlockEntry(e) => Some(e.get_bit(Self::UXN) as u8),
            Self::PageEntry(e) => Some(e.get_bit(Self::UXN) as u8),
            _ => None,
        }
    }
    pub fn get_PXN(&self) -> Option<u8> {
        match *self {
            Self::L1BlockEntry(e) => Some(e.get_bit(Self::PXN) as u8),
            Self::L2BlockEntry(e) => Some(e.get_bit(Self::PXN) as u8),
            Self::PageEntry(e) => Some(e.get_bit(Self::PXN) as u8),
            _ => None,
        }
    }

    pub fn get_Contiguous(&self) -> Option<u8> {
        match *self {
            Self::L1BlockEntry(e) => Some(e.get_bit(Self::Contiguous) as u8),
            Self::L2BlockEntry(e) => Some(e.get_bit(Self::Contiguous) as u8),
            Self::PageEntry(e) => Some(e.get_bit(Self::Contiguous) as u8),
            _ => None,
        }
    }
    pub fn get_nG(&self) -> Option<u8> {
        match *self {
            Self::L1BlockEntry(e) => Some(e.get_bit(Self::nG) as u8),
            Self::L2BlockEntry(e) => Some(e.get_bit(Self::nG) as u8),
            Self::PageEntry(e) => Some(e.get_bit(Self::nG) as u8),
            _ => None,
        }
    }
    pub fn get_AF(&self) -> Option<u8> {
        match *self {
            Self::L1BlockEntry(e) => Some(e.get_bit(Self::AF) as u8),
            Self::L2BlockEntry(e) => Some(e.get_bit(Self::AF) as u8),
            Self::PageEntry(e) => Some(e.get_bit(Self::AF) as u8),
            _ => None,
        }
    }
    pub fn get_SH(&self) -> Option<u8> {
        match *self {
            Self::L1BlockEntry(e) => Some(e.get_bits(Self::SH) as u8),
            Self::L2BlockEntry(e) => Some(e.get_bits(Self::SH) as u8),
            Self::PageEntry(e) => Some(e.get_bits(Self::SH) as u8),
            _ => None,
        }
    }
    pub fn get_AP(&self) -> Option<u8> {
        match *self {
            Self::L1BlockEntry(e) => Some(e.get_bits(Self::AP) as u8),
            Self::L2BlockEntry(e) => Some(e.get_bits(Self::AP) as u8),
            Self::PageEntry(e) => Some(e.get_bits(Self::AP) as u8),
            _ => None,
        }
    }
    pub fn get_NS(&self) -> Option<u8> {
        match *self {
            Self::L1BlockEntry(e) => Some(e.get_bit(Self::NS) as u8),
            Self::L2BlockEntry(e) => Some(e.get_bit(Self::NS) as u8),
            Self::PageEntry(e) => Some(e.get_bit(Self::NS) as u8),
            _ => None,
        }
    }
    pub fn get_AttrIndx(&self) -> Option<u8> {
        match *self {
            Self::L1BlockEntry(e) => Some(e.get_bits(Self::AttrIndx) as u8),
            Self::L2BlockEntry(e) => Some(e.get_bits(Self::AttrIndx) as u8),
            Self::PageEntry(e) => Some(e.get_bits(Self::AttrIndx) as u8),
            _ => None,
        }
    }

    pub fn set_NSTable(self, v: u64) -> Result<Self, ErrorCode> {
        match self {
            Self::TableEntry(e) => Ok(Self::TableEntry(e.set_bit(Self::NSTable, v))),
            _ => Err(EINVAL),
        }
    }
    pub fn set_APTable(self, v: u64) -> Result<Self, ErrorCode> {
        match self {
            Self::TableEntry(e) => Ok(Self::TableEntry(e.set_bits(Self::APTable, v))),
            _ => Err(EINVAL),
        }
    }
    pub fn set_UXNTable(self, v: u64) -> Result<Self, ErrorCode> {
        match self {
            Self::TableEntry(e) => Ok(Self::TableEntry(e.set_bit(Self::UXNTable, v))),
            _ => Err(EINVAL),
        }
    }
    pub fn set_PXNTable(self, v: u64) -> Result<Self, ErrorCode> {
        match self {
            Self::TableEntry(e) => Ok(Self::TableEntry(e.set_bit(Self::PXNTable, v))),
            _ => Err(EINVAL),
        }
    }
    pub fn set_UXN(self, v: u64) -> Result<Self, ErrorCode> {
        match self {
            Self::L1BlockEntry(e) => Ok(Self::L1BlockEntry(e.set_bit(Self::UXN, v))),
            Self::L2BlockEntry(e) => Ok(Self::L2BlockEntry(e.set_bit(Self::UXN, v))),
            Self::PageEntry(e) => Ok(Self::PageEntry(e.set_bit(Self::UXN, v))),
            _ => Err(EINVAL),
        }
    }
    pub fn set_PXN(self, v: u64) -> Result<Self, ErrorCode> {
        match self {
            Self::L1BlockEntry(e) => Ok(Self::L1BlockEntry(e.set_bit(Self::PXN, v))),
            Self::L2BlockEntry(e) => Ok(Self::L2BlockEntry(e.set_bit(Self::PXN, v))),
            Self::PageEntry(e) => Ok(Self::PageEntry(e.set_bit(Self::PXN, v))),
            _ => Err(EINVAL),
        }
    }

    pub fn set_Contiguous(self, v: u64) -> Result<Self, ErrorCode> {
        match self {
            Self::L1BlockEntry(e) => Ok(Self::L1BlockEntry(e.set_bit(Self::Contiguous, v))),
            Self::L2BlockEntry(e) => Ok(Self::L2BlockEntry(e.set_bit(Self::Contiguous, v))),
            Self::PageEntry(e) => Ok(Self::PageEntry(e.set_bit(Self::Contiguous, v))),
            _ => Err(EINVAL),
        }
    }
    pub fn set_nG(self, v: u64) -> Result<Self, ErrorCode> {
        match self {
            Self::L1BlockEntry(e) => Ok(Self::L1BlockEntry(e.set_bit(Self::nG, v))),
            Self::L2BlockEntry(e) => Ok(Self::L2BlockEntry(e.set_bit(Self::nG, v))),
            Self::PageEntry(e) => Ok(Self::PageEntry(e.set_bit(Self::nG, v))),
            _ => Err(EINVAL),
        }
    }
    pub fn set_AF(self, v: u64) -> Result<Self, ErrorCode> {
        match self {
            Self::L1BlockEntry(e) => Ok(Self::L1BlockEntry(e.set_bit(Self::AF, v))),
            Self::L2BlockEntry(e) => Ok(Self::L2BlockEntry(e.set_bit(Self::AF, v))),
            Self::PageEntry(e) => Ok(Self::PageEntry(e.set_bit(Self::AF, v))),
            _ => Err(EINVAL),
        }
    }
    pub fn set_SH(self, v: u64) -> Result<Self, ErrorCode> {
        match self {
            Self::L1BlockEntry(e) => Ok(Self::L1BlockEntry(e.set_bits(Self::SH, v))),
            Self::L2BlockEntry(e) => Ok(Self::L2BlockEntry(e.set_bits(Self::SH, v))),
            Self::PageEntry(e) => Ok(Self::PageEntry(e.set_bits(Self::SH, v))),
            _ => Err(EINVAL),
        }
    }
    pub fn set_AP(self, v: u64) -> Result<Self, ErrorCode> {
        match self {
            Self::L1BlockEntry(e) => Ok(Self::L1BlockEntry(e.set_bits(Self::AP, v))),
            Self::L2BlockEntry(e) => Ok(Self::L2BlockEntry(e.set_bits(Self::AP, v))),
            Self::PageEntry(e) => Ok(Self::PageEntry(e.set_bits(Self::AP, v))),
            _ => Err(EINVAL),
        }
    }
    pub fn set_NS(self, v: u64) -> Result<Self, ErrorCode> {
        match self {
            Self::L1BlockEntry(e) => Ok(Self::L1BlockEntry(e.set_bit(Self::NS, v))),
            Self::L2BlockEntry(e) => Ok(Self::L2BlockEntry(e.set_bit(Self::NS, v))),
            Self::PageEntry(e) => Ok(Self::PageEntry(e.set_bit(Self::NS, v))),
            _ => Err(EINVAL),
        }
    }
    pub fn set_AttrIndx(self, v: u64) -> Result<Self, ErrorCode> {
        match self {
            Self::L1BlockEntry(e) => Ok(Self::L1BlockEntry(e.set_bits(Self::AttrIndx, v))),
            Self::L2BlockEntry(e) => Ok(Self::L2BlockEntry(e.set_bits(Self::AttrIndx, v))),
            Self::PageEntry(e) => Ok(Self::PageEntry(e.set_bits(Self::AttrIndx, v))),
            _ => Err(EINVAL),
        }
    }

    pub fn value(&self) -> u64 {
        match *self {
            Self::L1BlockEntry(e)
            | Self::L2BlockEntry(e)
            | Self::TableEntry(e)
            | Self::PageEntry(e) => e,
            Self::INVALID => 0,
        }
    }

    pub fn get_address(&self) -> Option<PhysicalAddress> {
        match *self {
            Self::L1BlockEntry(e) => {
                PhysicalAddress::try_from((e.get_bits(30..48) as usize) << config::SHIFT_1G).ok()
            }
            Self::L2BlockEntry(e) => {
                PhysicalAddress::try_from((e.get_bits(21..48) as usize) << config::SHIFT_2M).ok()
            }
            Self::PageEntry(e) | Self::TableEntry(e) => {
                PhysicalAddress::try_from((e.get_bits(12..48) as usize) << config::SHIFT_4K).ok()
            }
            _ => None,
        }
    }

    pub fn set_address(&mut self, addr: PhysicalAddress) -> Result<(), ErrorCode> {
        match *self {
            Self::L1BlockEntry(e) => {
                if !addr.is_1G_aligned() {
                    Err(EALIGN)
                } else {
                    *self = Self::L1BlockEntry(e | addr.value() as u64);
                    Ok(())
                }
            }
            Self::L2BlockEntry(e) => {
                if !addr.is_2M_aligned() {
                    Err(EALIGN)
                } else {
                    *self = Self::L2BlockEntry(e | addr.value() as u64);
                    Ok(())
                }
            }
            Self::PageEntry(e) => {
                if !addr.is_4K_aligned() {
                    Err(EALIGN)
                } else {
                    *self = Self::PageEntry(e | addr.value() as u64);
                    Ok(())
                }
            }
            Self::TableEntry(e) => {
                if !addr.is_4K_aligned() {
                    Err(EALIGN)
                } else {
                    *self = Self::TableEntry(e | addr.value() as u64);
                    Ok(())
                }
            }
            _ => Err(EINVAL),
        }
    }

    const fn RW_normal() -> u64 {
        (0b1 << Self::AttrIndx.start) // Normal Memory
            | (0b0 << Self::NS) // Alway secure
            | (0b00 << Self::AP.start) //Read Write
            | (0b11 << Self::SH.start) //Inner Shareable
            | (0b1 << Self::AF) //Accessed
            | (0b0 << Self::nG) //Always global
            | (0b0 << Self::Contiguous) //Non contiguous
            | (0b1 << Self::PXN) // Never Execute at EL1
            | (0b1 << Self::UXN) // Never Execute at EL0
    }

    pub fn set_RW_normal(&mut self) -> Result<(), ErrorCode> {
        match *self {
            Self::L1BlockEntry(e) => {
                *self = Self::L1BlockEntry(e | Self::RW_NORMAL);
                Ok(())
            }
            Self::L2BlockEntry(e) => {
                *self = Self::L2BlockEntry(e | Self::RW_NORMAL);
                Ok(())
            }
            Self::PageEntry(e) => {
                *self = Self::PageEntry(e | Self::RW_NORMAL);
                Ok(())
            }
            _ => Err(EINVAL),
        }
    }
    const fn RWX_normal() -> u64 {
        (0b1 << Self::AttrIndx.start) // Normal Memory
            | (0b0 << Self::NS) // Alway secure
            | (0b00 << Self::AP.start) //Read Write
            | (0b11 << Self::SH.start) //Inner Shareable
            | (0b1 << Self::AF) //Accessed
            | (0b0 << Self::nG) //Always global
            | (0b0 << Self::Contiguous) //Non contiguous
            | (0b0 << Self::PXN) // Executable at EL1
            | (0b0 << Self::UXN) // Never Executable at EL0
    }

    pub fn set_RWX_normal(&mut self) -> Result<(), ErrorCode> {
        match *self {
            Self::L1BlockEntry(e) => {
                *self = Self::L1BlockEntry(e | Self::RWX_NORMAL);
                Ok(())
            }
            Self::L2BlockEntry(e) => {
                *self = Self::L2BlockEntry(e | Self::RWX_NORMAL);
                Ok(())
            }
            Self::PageEntry(e) => {
                *self = Self::PageEntry(e | Self::RWX_NORMAL);
                Ok(())
            }
            _ => Err(EINVAL),
        }
    }
    const fn RO_normal() -> u64 {
        (0b1 << Self::AttrIndx.start) // Normal Memory
            | (0b0 << Self::NS) // Alway secure
            | (0b10 << Self::AP.start) //Read Only
            | (0b11 << Self::SH.start) //Inner Shareable
            | (0b1 << Self::AF) //Accessed
            | (0b0 << Self::nG) //Always global
            | (0b0 << Self::Contiguous) //Non contiguous
            | (0b1 << Self::PXN) // Never Execute at EL1
            | (0b1 << Self::UXN) // Never Execute at EL0
    }
    pub fn set_RO_normal(&mut self) -> Result<(), ErrorCode> {
        match *self {
            Self::L1BlockEntry(e) => {
                *self = Self::L1BlockEntry(e | Self::RO_NORMAL);
                Ok(())
            }
            Self::L2BlockEntry(e) => {
                *self = Self::L2BlockEntry(e | Self::RO_NORMAL);
                Ok(())
            }
            Self::PageEntry(e) => {
                *self = Self::PageEntry(e | Self::RO_NORMAL);
                Ok(())
            }
            _ => Err(EINVAL),
        }
    }
    const fn X_normal() -> u64 {
        (0b1 << Self::AttrIndx.start) // Normal Memory
            | (0b0 << Self::NS) // Alway secure
            | (0b10 << Self::AP.start) //Read Only
            | (0b11 << Self::SH.start) //Inner Shareable
            | (0b1 << Self::AF) //Accessed
            | (0b0 << Self::nG) //Always global
            | (0b0 << Self::Contiguous) //Non contiguous
            | (0b0 << Self::PXN) // Executable at EL1
            | (0b1 << Self::UXN) // Never Execute at EL0
    }

    pub fn set_X_normal(&mut self) -> Result<(), ErrorCode> {
        match *self {
            Self::L1BlockEntry(e) => {
                *self = Self::L1BlockEntry(e | Self::X_NORMAL);
                Ok(())
            }
            Self::L2BlockEntry(e) => {
                *self = Self::L2BlockEntry(e | Self::X_NORMAL);
                Ok(())
            }
            Self::PageEntry(e) => {
                *self = Self::PageEntry(e | Self::X_NORMAL);
                Ok(())
            }
            _ => Err(EINVAL),
        }
    }
    const fn RW_device() -> u64 {
        (0b0 << Self::AttrIndx.start) // Device Memory
            | (0b0 << Self::NS) // Alway secure
            | (0b00 << Self::AP.start) //Read Write
            | (0b1 << Self::AF) //Accessed
            | (0b0 << Self::nG) //Always global
            | (0b0 << Self::Contiguous) //Non contiguous
            | (0b1 << Self::PXN) // Never Execute at EL1
            | (0b1 << Self::UXN) // Never Execute at EL0
                                 // Shareability does not matter to device memory
    }

    pub fn set_RW_device(&mut self) -> Result<(), ErrorCode> {
        match *self {
            Self::L1BlockEntry(e) => {
                *self = Self::L1BlockEntry(e | Self::RW_DEVICE);
                Ok(())
            }
            Self::L2BlockEntry(e) => {
                *self = Self::L2BlockEntry(e | Self::RW_DEVICE);
                Ok(())
            }
            Self::PageEntry(e) => {
                *self = Self::PageEntry(e | Self::RW_DEVICE);
                Ok(())
            }
            _ => Err(EINVAL),
        }
    }
    const fn RO_device() -> u64 {
        (0b0 << Self::AttrIndx.start) // Device Memory
            | (0b0 << Self::NS) // Alway secure
            | (0b10 << Self::AP.start) //Read Only
            | (0b1 << Self::AF) //Accessed
            | (0b0 << Self::nG) //Always global
            | (0b0 << Self::Contiguous) //Non contiguous
            | (0b1 << Self::PXN) // Never Execute at EL1
            | (0b1 << Self::UXN) // Never Execute at EL0
                                 // Shareability does not matter to device memory
    }

    pub fn set_RO_device(&mut self) -> Result<(), ErrorCode> {
        match *self {
            Self::L1BlockEntry(e) => {
                *self = Self::L1BlockEntry(e | Self::RO_DEVICE);
                Ok(())
            }
            Self::L2BlockEntry(e) => {
                *self = Self::L2BlockEntry(e | Self::RO_DEVICE);
                Ok(())
            }
            Self::PageEntry(e) => {
                *self = Self::PageEntry(e | Self::RO_DEVICE);
                Ok(())
            }
            _ => Err(EINVAL),
        }
    }

    // Table attributes are fixed
    //
    // APTable = 01: Accesses from EL0 are never permitted in subsequent tables
    // UXN = PXN = 0: Does not effect the executability of subsequent tables
    // NS = 0: secure
    const fn Table_attr() -> u64 {
        (0b0 << Self::NSTable) // if NSTable = 1, subsequent entries are treated as non-global,
        // regardless of its nG bit.
            | (0b01 << Self::APTable.start)
            | (0b0 << Self::UXNTable)
            | (0b0 << Self::PXNTable)
            | (0b1 << Self::AF) // Accessed
    }
    pub fn set_table_attributes(&mut self) -> Result<(), ErrorCode> {
        match *self {
            Self::TableEntry(e) => {
                *self = Self::TableEntry(e | Self::TABLE_ATTR);
                Ok(())
            }
            _ => Err(EINVAL),
        }
    }

    pub fn set_invalid(&mut self) -> Result<(), ErrorCode> {
        *self = Self::INVALID;
        Ok(())
    }

    pub fn set_attributes(&mut self, mt: &MemoryType) -> Result<(), ErrorCode> {
        match *mt {
            MemoryType::RoNormal => self.set_RO_normal(),
            MemoryType::RwNormal => self.set_RW_normal(),
            MemoryType::XNormal => self.set_X_normal(),
            MemoryType::RWXNormal => self.set_RWX_normal(),
            MemoryType::RoDevice => self.set_RO_device(),
            MemoryType::RwDevice => self.set_RW_device(),
            MemoryType::Table => self.set_table_attributes(),
            MemoryType::INVALID => self.set_invalid(),
        }
    }
}

impl fmt::Debug for Descriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::L1BlockEntry(_) => write!(f, "L1 Block\nUXN = {:1}, PXN = {:1}, Contiguous = {:1}, nG = {:1}, AF = {:1}, SH = {:#04b}, AP = {:#04b}, NS = {:1}, AttrIndx = {:#05b}, L1 Block address = {}", self.get_UXN().unwrap(), self.get_PXN().unwrap(), self.get_Contiguous().unwrap(), self.get_nG().unwrap(), self.get_AF().unwrap(), self.get_SH().unwrap(), self.get_AP().unwrap(), self.get_NS().unwrap(), self.get_AttrIndx().unwrap(), self.get_address().unwrap()),
            Self::L2BlockEntry(_) => write!(f, "L2 Block\nUXN = {:1}, PXN = {:1}, Contiguous = {:1}, nG = {:1}, AF = {:1}, SH = {:#04b}, AP = {:#04b}, NS = {:1}, AttrIndx = {:#05b}, L2 Block address = {}", self.get_UXN().unwrap(), self.get_PXN().unwrap(), self.get_Contiguous().unwrap(), self.get_nG().unwrap(), self.get_AF().unwrap(), self.get_SH().unwrap(), self.get_AP().unwrap(), self.get_NS().unwrap(), self.get_AttrIndx().unwrap(), self.get_address().unwrap()),
            Self::PageEntry(_) => write!(f, "Page\nUXN = {:1}, PXN = {:1}, Contiguous = {:1}, nG = {:1}, AF = {:1}, SH = {:#04b}, AP = {:#04b}, NS = {:1}, AttrIndx = {:#05b}, Page Address(physical) = {}", self.get_UXN().unwrap(), self.get_PXN().unwrap(), self.get_Contiguous().unwrap(), self.get_nG().unwrap(), self.get_AF().unwrap(), self.get_SH().unwrap(), self.get_AP().unwrap(), self.get_NS().unwrap(), self.get_AttrIndx().unwrap(), self.get_address().unwrap()),
            Self::TableEntry(_) => write!(f, "Table\nNSTable = {:1}, APTable = {:#04b}, UXNTable = {:1}, PXNTable = {:1}, Table Address(physical) = {}", self.get_NSTable().unwrap(), self.get_APTable().unwrap(), self.get_UXNTable().unwrap(), self.get_PXNTable().unwrap(), self.get_address().unwrap()),
            Self::INVALID => write!(f, "Invalid descriptor"),
        }
    }
}

impl fmt::Display for Descriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::L1BlockEntry(_) => {
                write!(
                    f,
                    "L1 {} block, address = {}",
                    self.get_attributes(),
                    self.get_address().unwrap()
                )
            }
            Self::L2BlockEntry(_) => {
                write!(
                    f,
                    "L2 {} block, address = {}",
                    self.get_attributes(),
                    self.get_address().unwrap()
                )
            }
            Self::PageEntry(_) => {
                write!(
                    f,
                    "{} page, address = {}",
                    self.get_attributes(),
                    self.get_address().unwrap()
                )
            }
            Self::TableEntry(_) => {
                write!(
                    f,
                    "{}, address = {}",
                    self.get_attributes(),
                    self.get_address().unwrap()
                )
            }
            Self::INVALID => write!(f, "Invalid descriptor"),
        }
    }
}
impl<L> TranslationTableEntry<L> {
    pub fn is_valid(&self) -> bool {
        self.entry.get_bit(0) == 1
    }

    pub fn value(&self) -> u64 {
        self.entry
    }
}

impl<L> From<Descriptor> for TranslationTableEntry<L> {
    fn from(d: Descriptor) -> Self {
        Self {
            entry: d.value(),
            _l: PhantomData,
        }
    }
}

pub trait GetDescriptor {
    fn get(&self) -> Descriptor;
}

impl GetDescriptor for TranslationTableEntry<Level1> {
    fn get(&self) -> Descriptor {
        if !self.is_valid() {
            Descriptor::INVALID
        } else if self.entry.get_bit(1) == 0 {
            Descriptor::L1BlockEntry(self.value())
        } else {
            Descriptor::TableEntry(self.value())
        }
    }
}
impl GetDescriptor for TranslationTableEntry<Level2> {
    fn get(&self) -> Descriptor {
        if !self.is_valid() {
            Descriptor::INVALID
        } else if self.entry.get_bit(1) == 0 {
            Descriptor::L2BlockEntry(self.value())
        } else {
            Descriptor::TableEntry(self.value())
        }
    }
}
impl GetDescriptor for TranslationTableEntry<Level3> {
    fn get(&self) -> Descriptor {
        if !self.is_valid() {
            Descriptor::INVALID
        } else {
            Descriptor::PageEntry(self.value())
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
        {
            println!("RWNORMAL = {:#066b}", Descriptor::RW_NORMAL);
            println!("RONORMAL = {:#066b}", Descriptor::RO_NORMAL);
            println!("XNORMAL = {:#066b}", Descriptor::X_NORMAL);
            println!("RWXNORMAL = {:#066b}", Descriptor::RWX_NORMAL);
            println!("RWDEVICE = {:#066b}", Descriptor::RW_DEVICE);
            println!("RODEVICE = {:#066b}", Descriptor::RO_DEVICE);
            println!("Table_attr = {:#066b}", Descriptor::TABLE_ATTR);
        }
        // Level 1 block
        {
            let e: TranslationTableEntry<Level1> = Default::default();
            assert!(!e.is_valid());
            let mut ans = 0u64;
            let mut b = e.get().set_l1_block().unwrap();
            b.set_attributes(RWNORMAL).unwrap();

            ans = Descriptor::RW_NORMAL | 0b01;
            assert_eq!(b.value(), ans);
            b.set_address(PhysicalAddress::try_from(0xFF).unwrap())
                .expect_err("address is not aligned");
            assert_eq!(b.value(), ans);
            b.set_address(PhysicalAddress::try_from(0b0usize << config::SHIFT_1G).unwrap())
                .unwrap();
            ans |= 0b0u64 << config::SHIFT_1G;
            assert_eq!(b.value(), ans);
        }
        {
            let e: TranslationTableEntry<Level2> = Default::default();
            assert!(!e.is_valid());
            let mut ans = 0u64;
            let mut b = e.get().set_l2_block().unwrap();
            b.set_attributes(RONORMAL).unwrap();

            ans = Descriptor::RO_NORMAL | 0b01;
            assert_eq!(b.value(), ans);
            b.set_address(PhysicalAddress::try_from(0x91F).unwrap())
                .expect_err("address is not aligned");
            assert_eq!(b.value(), ans);
            b.set_address(PhysicalAddress::try_from(0xFFusize << config::SHIFT_2M).unwrap())
                .unwrap();
            ans |= 0xFFu64 << config::SHIFT_2M;
            assert_eq!(b.value(), ans);
        }
    }
}
