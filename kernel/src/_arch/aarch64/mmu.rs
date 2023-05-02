use crate::{
    bsp::mmio::PHYSICAL_MEMORY_END_INCLUSIVE,
    errno::{ErrorCode, EAGAIN},
    println,
};
use aarch64_cpu::registers::*;
use core::ops::Range;
use tock_registers::interfaces::{Readable, Writeable};

#[path = "mmu/address.rs"]
mod address;
#[path = "mmu/translation_table.rs"]
mod translation_table;

// use translation_table::*;
#[derive(Default)]
pub struct TGRAN4K;
pub struct TGRAN16K;
pub struct TGRAN64K;

pub trait Granule {
    const ENTRIES: usize;
    const BITS_RESOLVED: u8;
    const SHIFT: u8; // offset bits

    const OFFSET_MASK: u64 = (1 << Self::SHIFT) - 1;
    const LEVEL_MASK: u64 = (1 << Self::BITS_RESOLVED) - 1;

    // Index range
    const LEVEL0: Range<usize>;
    const LEVEL1: Range<usize>;
    const LEVEL2: Range<usize>;
    const LEVEL3: Range<usize>;
    const OFFSET: Range<usize>;

    // Descriptor fields
    const VALID: Range<usize> = 0..1;
    const TYPE: Range<usize> = 1..2;
    const TABLE_ATTR: Range<usize> = 59..64;
    const BLOCK_ATTR: Range<usize> = 52..64;

    const NEXT_LEVEL_TABLE_ADDR: Range<usize>;

    const LEVLE1_BLOCK_ADDR: Range<usize>;
    const LEVLE2_BLOCK_ADDR: Range<usize>;
}

impl Granule for TGRAN4K {
    const ENTRIES: usize = 512;
    const BITS_RESOLVED: u8 = 9;
    const SHIFT: u8 = 12;

    const LEVEL0: Range<usize> = 39..48;
    const LEVEL1: Range<usize> = 30..39;
    const LEVEL2: Range<usize> = 21..30;
    const LEVEL3: Range<usize> = 12..21;
    const OFFSET: Range<usize> = 0..12;

    const NEXT_LEVEL_TABLE_ADDR: Range<usize> = 12..48;
    const LEVLE1_BLOCK_ADDR: Range<usize> = 30..48;
    const LEVLE2_BLOCK_ADDR: Range<usize> = 21..48;
}

impl Granule for TGRAN16K {
    const ENTRIES: usize = 2048;
    const BITS_RESOLVED: u8 = 11;
    const SHIFT: u8 = 14;

    const LEVEL0: Range<usize> = 47..48;
    const LEVEL1: Range<usize> = 36..47;
    const LEVEL2: Range<usize> = 25..36;
    const LEVEL3: Range<usize> = 14..25;
    const OFFSET: Range<usize> = 0..14;

    const NEXT_LEVEL_TABLE_ADDR: Range<usize> = 14..48;
    const LEVLE1_BLOCK_ADDR: Range<usize> = 0..0;
    const LEVLE2_BLOCK_ADDR: Range<usize> = 25..48;
}

impl Granule for TGRAN64K {
    const ENTRIES: usize = 8192;
    const BITS_RESOLVED: u8 = 13;
    const SHIFT: u8 = 16;

    const LEVEL0: Range<usize> = 0..0;
    const LEVEL1: Range<usize> = 42..48;
    const LEVEL2: Range<usize> = 29..42;
    const LEVEL3: Range<usize> = 16..29;
    const OFFSET: Range<usize> = 0..16;

    const NEXT_LEVEL_TABLE_ADDR: Range<usize> = 16..48;
    const LEVLE1_BLOCK_ADDR: Range<usize> = 0..0;
    const LEVLE2_BLOCK_ADDR: Range<usize> = 29..48;
}

pub struct MemoryManagementUnit;

impl MemoryManagementUnit {
    pub const fn new() -> Self {
        Self {}
    }

    fn is_4kb_page_supported(&self) -> bool {
        ID_AA64MMFR0_EL1.read(ID_AA64MMFR0_EL1::TGran4) == 0
    }

    pub fn config_tcr_el1(&self) -> Result<(), ErrorCode> {
        let t0sz: u64 = (64 - (PHYSICAL_MEMORY_END_INCLUSIVE + 1).trailing_zeros()) as u64; // currently just identity map

        println!(
            "[MMU]: TTBR0: 0x0 - {:#x}",
            u64::pow(2, (64 - t0sz) as u32) - 1
        );

        if !self.is_4kb_page_supported() {
            return Err(EAGAIN);
        } else {
            println!("[MMU]: use 4kb frame size");
        }

        // Support physical memory up to 64GB
        TCR_EL1.write(
            TCR_EL1::IPS::Bits_32 /*pi4 has 4GB memory*/
                + TCR_EL1::T0SZ.val(t0sz) /*T0 supports: 0x0000_0000 - 0xFFFF_FFFF, the initial look up is from level 1 */
                + TCR_EL1::TBI0::Used
                + TCR_EL1::A1::TTBR0
                + TCR_EL1::TG0::KiB_4
                + TCR_EL1::SH0::Inner
                + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
                + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
                + TCR_EL1::EPD1::DisableTTBR1Walks
                + TCR_EL1::EPD0::EnableTTBR0Walks,
        );
        Ok(())
    }

    pub fn config_mair_el1(&self) {
        MAIR_EL1.write(
            MAIR_EL1::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc
                + MAIR_EL1::Attr1_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc
                + MAIR_EL1::Attr0_Device::nonGathering_nonReordering_EarlyWriteAck,
        );
    }
}

pub static MMU: MemoryManagementUnit = MemoryManagementUnit::new();
