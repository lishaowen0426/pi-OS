use crate::{
    errno::{ErrorCode, EAGAIN},
    println,
};
use aarch64_cpu::{asm::barrier,registers::*};
use tock_registers::interfaces::{Readable, Writeable, ReadWriteable};

#[path = "mmu/address.rs"]
mod address;
#[path = "mmu/translation_table.rs"]
mod translation_table;
#[path = "mmu/translation_entry.rs"]
mod translation_entry;
#[path = "mmu/config.rs"]
mod mmu_config;
#[path = "mmu/cache.rs"]
mod cache;

#[path = "mmu/frame_allocator.rs"]
mod frame_allocator;

use mmu_config::config;
use translation_table::*;
use translation_entry::*;
use cache::*;

use core::arch::asm;
pub struct MemoryManagementUnit;

impl MemoryManagementUnit {
    pub const fn new() -> Self {
        Self {}
    }

    pub fn cache_info(&self) {
        let a64_caches = A64CacheSet::new().unwrap();
        println!("{}", a64_caches);
    } 

    fn is_4kb_page_supported(&self) -> bool {
        ID_AA64MMFR0_EL1.read(ID_AA64MMFR0_EL1::TGran4) == 0
    }

    fn config_tcr_el1(&self) -> Result<(), ErrorCode> {
        // let t0sz: u64 = (64 - (PHYSICAL_MEMORY_END_INCLUSIVE + 1).trailing_zeros()) as u64; //
        // currently just identity map
        let t0sz: u64 = 16 + 9; // start from level 1

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
                + TCR_EL1::T0SZ.val(t0sz) 
                + TCR_EL1::TBI0::Used /*Memory Taggging Extension(MTE) is enabled for Range0 */
                + TCR_EL1::A1::TTBR0
                + TCR_EL1::TG0::KiB_4
                + TCR_EL1::SH0::Inner
                + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
                + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
                //+ TCR_EL1::ORGN0::NonCacheable
                //+ TCR_EL1::IRGN0::NonCacheable
                + TCR_EL1::EPD1::DisableTTBR1Walks          + TCR_EL1::EPD0::EnableTTBR0Walks,
        );
        Ok(())
    }

    fn config_mair_el1(&self) {

        // Be careful when change this!
        // We use the attribute index in some places when we set the block/page table entry AttrIdx
        // Remember to change those if MAIR_EL1 is modified.
        
        
        
        MAIR_EL1.write(
            MAIR_EL1::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc
                + MAIR_EL1::Attr1_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc
                + MAIR_EL1::Attr0_Device::nonGathering_nonReordering_EarlyWriteAck,
        );
        
        
       /* 
        MAIR_EL1.write(
            MAIR_EL1::Attr1_Normal_Outer::NonCacheable
                + MAIR_EL1::Attr1_Normal_Inner::NonCacheable
                + MAIR_EL1::Attr0_Device::nonGathering_nonReordering_EarlyWriteAck,
        );
        */
        
        
        
    }

    // AARCH64 requires:
    // all caches reset to IMPLEMENTATION DEFINED and UNKNOWN states
    // all memory locations are treated as non-cacheable at reset

    #[inline(never)]
    pub fn config(&self) -> Result<(), ErrorCode>{
        self.cache_info();
        //config tcr
        self.config_tcr_el1().unwrap();
        //config mair
        self.config_mair_el1();
        //set up initial mapping
        //
        let l1_table: L1TranslationTable =
            TranslationTable::new(get_ttbr0() as *mut L1Entry, config::ENTRIES_PER_TABLE);
        l1_table.set_up_init_mapping();
        //barrier
        barrier::isb(barrier::SY);
            
        
        /*
        unsafe{
            asm!(
                "IC IALLU", /*invalidate instruction cache*/
                "TLBI ALLE1",
            );
        }
            */
        
            
        // Enable the MMU and turn on data and instruction caching.
        
        SCTLR_EL1.modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable+ SCTLR_EL1::WXN::Disable);
        
        barrier::isb(barrier::SY);
        unsafe{
            let l1: *const u64 = config::L1_VIRTUAL_ADDRESS as *const u64;
            println!("Trying to access l1 page table through its virtual address {:#018x} after enabling mmu", config::L1_VIRTUAL_ADDRESS);
            println!("the first entry of the l1 table: {}", *l1);
        }
        ///

        Ok(())
    }
}

pub static MMU: MemoryManagementUnit = MemoryManagementUnit::new();

#[cfg(test)] 
#[allow(unused_imports,unused_variables,dead_code)]
 mod tests{
 use super::*;
use test_macros::kernel_test; 
 #[kernel_test] 
 fn test_mmu(){
        MMU.config().unwrap();
    }
} 
