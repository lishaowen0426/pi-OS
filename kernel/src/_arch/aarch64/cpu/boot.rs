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

use crate::{memory::*, println, unsafe_println};
use aarch64_cpu::{asm, asm::barrier, registers::*};
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

extern "C" {
    static __code_start: u8;
    static __code_end_exclusive: u8;
    static __bss_start: u8;
    static __bss_end_exclusive: u8;
    static __data_start: u8;
    static __data_end_exclusive: u8;
    static l1_lower_page_table: u8;
    static initial_stack_top: u8;
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
    SP_EL1.set(&initial_stack_top as *const u8 as u64);
    unsafe_println!("HCR_EL2 = {:#066b}", HCR_EL2.get());
    unsafe_println!("SPSR_EL2 = {:#066b}", SPSR_EL2.get());
    unsafe_println!("CNTHCTL_EL2 = {:#066b}", CNTHCTL_EL2.get());
    unsafe_println!("CNTVOFF_EL2 = {:#066b}", CNTVOFF_EL2.get());
}

#[no_mangle]
pub unsafe extern "C" fn _start_rust(x0: u64) -> ! {
    {
        unsafe_println!("x0 = {:#066x}", x0);
    }

    prepare_el2_to_el1();
    asm::eret()
}
