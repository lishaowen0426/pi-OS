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

use core::arch::global_asm;

use aarch64_cpu::{asm, registers::*};
use tock_registers::interfaces::Writeable;
// Assembly counterpart to this file.
global_asm!(
    include_str!("boot.s"),
    CONST_CORE_ID_MASK = const 0b11 ,
    CONST_EL2 = const 0b1000,
);

#[inline(always)]
unsafe fn prepare_el2_to_el1(phys_boot_core_stack_end_exclusive_addr: u64) {
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
    SP_EL1.set(phys_boot_core_stack_end_exclusive_addr);
}

#[no_mangle]
pub unsafe extern "C" fn _start_rust(phys_boot_core_stack_end_exclusive_addr: u64) -> ! {
    prepare_el2_to_el1(phys_boot_core_stack_end_exclusive_addr);
    asm::eret()
}
