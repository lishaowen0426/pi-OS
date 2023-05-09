use crate::{
    errno::{ErrorCode, EAGAIN},
    println,
};
use aarch64_cpu::registers::*;
use tock_registers::interfaces::{Readable, Writeable};

#[path = "mmu/address.rs"]
mod address;
#[path = "mmu/translation_table.rs"]
mod translation_table;
#[path = "mmu/translation_entry.rs"]
mod translation_entry;
#[path = "mmu/config.rs"]
mod mmu_config;

use mmu_config::config;



pub struct MemoryManagementUnit;

impl MemoryManagementUnit {
    pub const fn new() -> Self {
        Self {}
    }

    fn is_4kb_page_supported(&self) -> bool {
        ID_AA64MMFR0_EL1.read(ID_AA64MMFR0_EL1::TGran4) == 0
    }

    pub fn config_tcr_el1(&self) -> Result<(), ErrorCode> {
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

    pub fn config(&self) -> Result<(), ErrorCode>{
        //config tcr
        //config mair
        //set up initial mapping
        //
        //barrier
        // Enable the MMU and turn on data and instruction caching.
        //SCTLR_EL1.modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable);
        ////barrier

        Ok(())
    }
}

pub static MMU: MemoryManagementUnit = MemoryManagementUnit::new();
