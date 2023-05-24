// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2021-2022 Andre Richter <andre.o.richter@gmail.com>

//! Architectural boot code.
//!
//! # Orientation
//!
//! Since arch modules are imported into generic modules using the path attribute, the path of this
//! file is:
//!
//! crate::cpu::boot::arch_boot

#[cfg(test)]
use core::arch::global_asm;

use crate::{println,unsafe_println};
use aarch64_cpu::{asm::barrier, asm, registers::*};
use tock_registers::interfaces::{ReadWriteable,Writeable, Readable};
use crate::memory::*;

extern "C" {
    static __boot_core_stack_end_exclusive: u8;
    static __code_start: u8;
    static __code_end_exclusive: u8;
    static __bss_start: u8;
    static __bss_end_exclusive: u8;
    static __data_start: u8;
    static __data_end_exclusive: u8;
    static __l1_page_table_start: u8;
}

fn config_registers_el1()  {
        // let t0sz: u64 = (64 - (PHYSICAL_MEMORY_END_INCLUSIVE + 1).trailing_zeros()) as u64; //
        // currently just identity map
    let t0sz: u64 = 16 + 9; // start from level 1



    let is_4kb_page_supported = || -> bool {
        ID_AA64MMFR0_EL1.read(ID_AA64MMFR0_EL1::TGran4) == 0
    };


        

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

    TTBR0_EL1.modify(TTBR0_EL1::ASID.val(0 as u64));
    unsafe{
    TTBR0_EL1.set_baddr(&__l1_page_table_start as * const _ as u64);
    }
    barrier::isb(barrier::SY);

    SCTLR_EL1.modify(
        SCTLR_EL1::M::Enable
        + SCTLR_EL1::C::Cacheable
        + SCTLR_EL1::I::Cacheable
        + SCTLR_EL1::WXN::Disable
        + SCTLR_EL1::UCI::Trap, // Cache maintenance instruction at EL0 are not allowed
    );

    barrier::isb(barrier::SY);

}

#[inline(always)]
unsafe fn prepare_el2_to_el1() {
    CNTHCTL_EL2.write(CNTHCTL_EL2::EL1PCEN::SET + CNTHCTL_EL2::EL1PCTEN::SET);
    CNTVOFF_EL2.set(0);

    HCR_EL2.write(HCR_EL2::RW::EL1IsAarch64);

    SPSR_EL2.write(
        SPSR_EL2::D::Masked
            + SPSR_EL2::A::Masked
            + SPSR_EL2::I::Masked
            + SPSR_EL2::F::Masked
            + SPSR_EL2::M::EL1h,
    );

    ELR_EL2.set(crate::kernel_main as *const () as u64);
    SP_EL1.set(&__boot_core_stack_end_exclusive as *const u8 as u64);
}

#[no_mangle]
pub unsafe extern "C" fn _start_rust(x0:u64) -> ! {

    {
        unsafe_println!("enter rust");

    }
    



    prepare_el2_to_el1();
    asm::eret()
}
