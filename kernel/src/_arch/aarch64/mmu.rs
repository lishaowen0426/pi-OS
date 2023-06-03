use crate::{
    errno::{ErrorCode, EAGAIN},
    println, BootInfo,
};
use aarch64_cpu::registers::*;
use spin::{mutex::SpinMutex, once::Once};
use tock_registers::interfaces::{Readable, Writeable};

#[path = "mmu/address.rs"]
pub mod address;
#[path = "mmu/cache.rs"]
mod cache;
#[path = "mmu/config.rs"]
pub mod config;
#[path = "mmu/translation_entry.rs"]
pub mod translation_entry;
#[path = "mmu/translation_table.rs"]
pub mod translation_table;

#[path = "mmu/allocator.rs"]
mod allocator;
#[path = "mmu/heap.rs"]
pub mod heap;

use address::*;
use allocator::*;
use cache::*;
use translation_entry::*;
use translation_table::*;

fn config_registers_el1() -> Result<(), ErrorCode> {
    // let t0sz: u64 = (64 - (PHYSICAL_MEMORY_END_INCLUSIVE + 1).trailing_zeros()) as u64; //
    // currently just identity map
    let t0sz: u64 = 16 + 9; // start from level 1
    let t1sz: u64 = 16 + 9; // start from level 1

    println!("TTBR0: 0x0 - {:#x}", u64::pow(2, (64 - t0sz) as u32) - 1);

    let is_4kb_page_supported = || -> bool { ID_AA64MMFR0_EL1.read(ID_AA64MMFR0_EL1::TGran4) == 0 };

    if !is_4kb_page_supported() {
        return Err(EAGAIN);
    }

    // Support physical memory up to 64GB
    TCR_EL1.write(
        TCR_EL1::IPS::Bits_32 /*pi4 has 4GB memory*/
        + TCR_EL1::T0SZ.val(t0sz)
        + TCR_EL1::T1SZ.val(t1sz)
        //+ TCR_EL1::TBI0::Used /*Memory Taggging Extension(MTE) is not supported on pi */
        //+ TCR_EL1::TBI1::Used /*Memory Taggging Extension(MTE) is not supported on pi */
        + TCR_EL1::AS::ASID8Bits /* Sizeof ASID = 8 bits*/
        + TCR_EL1::A1::TTBR0
        + TCR_EL1::TG0::KiB_4
        + TCR_EL1::TG1::KiB_4
        + TCR_EL1::SH0::Inner /*AArch64 assumes all PEs use the same OS are in the same Inner Shareable domain*/
        + TCR_EL1::SH1::Inner /*AArch64 assumes all PEs use the same OS are in the same Inner Shareable domain*/
        + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::ORGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::IRGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::EPD1::EnableTTBR1Walks
        + TCR_EL1::EPD0::EnableTTBR0Walks,
    );

    // Be careful when change this!
    // We use the attribute index in some places when we set the block/page table entry AttrIdx
    // Remember to change those if MAIR_EL1 is modified.
    MAIR_EL1.write(
        MAIR_EL1::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc
            + MAIR_EL1::Attr1_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc
            + MAIR_EL1::Attr0_Device::nonGathering_nonReordering_EarlyWriteAck,
    );

    Ok(())
}

pub fn init(boot_info: &BootInfo) -> Result<(), ErrorCode> {
    MMU.call_once(|| {
        MemoryManagementUnit::new(
            UnsafeTranslationTable::new(config::LOWER_L1_VIRTUAL_ADDRESS as *mut L1Entry),
            UnsafeTranslationTable::new(config::HIGHER_L1_VIRTUAL_ADDRESS as *mut L1Entry),
        )
    });

    //  give 2m va and pa to the heap allocator so that it can distribute memory to the page and
    //   frame allocator
    let mut lower_free_page_range = boot_info.lower_free_page;
    let mut higher_free_page_range = boot_info.higher_free_page;
    println!("higher page range: {}", higher_free_page_range);
    let mut free_frame = boot_info.free_frame;

    Ok(())
}

#[derive(Copy, Clone)]
pub enum BlockSize {
    _4K,
    _2M,
    _1G,
}

pub struct MemoryManagementUnit {
    lower_l1: SpinMutex<UnsafeTranslationTable<Level1>>,
    higher_l1: SpinMutex<UnsafeTranslationTable<Level1>>,
    cache: A64CacheSet,
}
impl MemoryManagementUnit {
    pub fn new(
        lower_l1_table: UnsafeTranslationTable<Level1>,
        higher_l1_table: UnsafeTranslationTable<Level1>,
    ) -> Self {
        Self {
            lower_l1: SpinMutex::new(lower_l1_table),
            higher_l1: SpinMutex::new(higher_l1_table),
            cache: A64CacheSet::new().unwrap(),
        }
    }

    pub fn map(
        &self,
        va: VirtualAddress,
        pa: PhysicalAddress,
        mt: &MemoryType,
    ) -> Result<(), ErrorCode> {
        if va.is_lower() {
            self.lower_l1.lock().map(va, pa, mt, BlockSize::_4K)
        } else {
            self.higher_l1.lock().map(va, pa, mt, BlockSize::_4K)
        }
    }
    pub fn translate(&self, va: VirtualAddress) -> Option<PhysicalAddress> {
        if va.is_lower() {
            self.lower_l1.lock().translate(va)
        } else {
            self.higher_l1.lock().translate(va)
        }
    }
}

pub static MMU: Once<MemoryManagementUnit> = Once::new();

#[cfg(test)]
#[allow(unused_imports, unused_variables, dead_code)]
mod tests {
    use super::*;
    use test_macros::kernel_test;
    #[kernel_test]
    fn test_mmu() {}
}
