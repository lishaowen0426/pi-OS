use crate::{
    errno::{ErrorCode, EAGAIN},
    unsafe_println,
};
use aarch64_cpu::{asm::barrier, registers::*};
use spin::{mutex::SpinMutex, once::Once};
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

#[path = "mmu/address.rs"]
mod address;
#[path = "mmu/cache.rs"]
mod cache;
#[path = "mmu/config.rs"]
pub mod mmu_config;
#[path = "mmu/translation_entry.rs"]
mod translation_entry;
#[path = "mmu/translation_table.rs"]
mod translation_table;

#[path = "mmu/frame_allocator.rs"]
mod frame_allocator;

use cache::*;
pub use mmu_config::config;
use translation_entry::*;
use translation_table::*;


fn config_registers_el1() -> Result<(), ErrorCode> {
        // let t0sz: u64 = (64 - (PHYSICAL_MEMORY_END_INCLUSIVE + 1).trailing_zeros()) as u64; //
        // currently just identity map
    let t0sz: u64 = 16 + 9; // start from level 1

    unsafe_println!(
        "TTBR0: 0x0 - {:#x}",
        u64::pow(2, (64 - t0sz) as u32) - 1
    );


    let is_4kb_page_supported = || -> bool {
        ID_AA64MMFR0_EL1.read(ID_AA64MMFR0_EL1::TGran4) == 0
    };


    if !is_4kb_page_supported() {
        return Err(EAGAIN);
    }
        

    // Support physical memory up to 64GB
    TCR_EL1.write(
        TCR_EL1::IPS::Bits_32 /*pi4 has 4GB memory*/
        + TCR_EL1::T0SZ.val(t0sz) 
        + TCR_EL1::TBI0::Ignored /*Memory Taggging Extension(MTE) is not supported on pi */
        + TCR_EL1::AS::ASID8Bits /* Sizeof ASID = 8 bits*/
        + TCR_EL1::A1::TTBR0
        + TCR_EL1::TG0::KiB_4
        + TCR_EL1::SH0::Inner /*AArch64 assumes all PEs use the same OS are in the same Inner Shareable domain*/
        + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::EPD1::DisableTTBR1Walks          + TCR_EL1::EPD0::EnableTTBR0Walks,
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


        

pub fn init() -> Result<(), ErrorCode> {
    config_registers_el1()?;
    translation_table::set_up_init_mapping()?;

    // Enable the MMU and turn on data and instruction caching.

    SCTLR_EL1.modify(
        SCTLR_EL1::M::Enable
        + SCTLR_EL1::C::Cacheable
        + SCTLR_EL1::I::Cacheable
        + SCTLR_EL1::WXN::Disable
        + SCTLR_EL1::UCI::Trap, // Cache maintenance instruction at EL0 are not allowed
    );

    barrier::isb(barrier::SY);


    MMU.call_once(|| {
        MemoryManagementUnit::new(UnsafeTranslationTable::new(
            config::L1_VIRTUAL_ADDRESS as *mut L1Entry,
        ))
    });

    Ok(())
}

#[derive(Copy, Clone)]
pub enum BlockSize{
    _4K,
    _2M,
    _1G,
}

pub struct MemoryManagementUnit {
    l1: SpinMutex<UnsafeTranslationTable<Level1>>,
    cache: A64CacheSet,
}
impl MemoryManagementUnit {
    pub fn new(l1_table: UnsafeTranslationTable<Level1>) -> Self{
        Self{
            l1: SpinMutex::new(l1_table),
            cache: A64CacheSet::new().unwrap(),
        }
    }

    pub fn map() {
        todo!()
    }


}

pub static MMU: Once<MemoryManagementUnit> = Once::new();

#[cfg(test)]
#[allow(unused_imports, unused_variables, dead_code)]
mod tests {
    use super::*;
    use test_macros::kernel_test;
    #[kernel_test]
    fn test_mmu() {

    }
}
