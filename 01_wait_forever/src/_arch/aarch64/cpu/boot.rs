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

const SCTLR_RESERVED: u32 = (3 << 28) | (3 << 22) | (1 << 20) | (1 << 11);
const SCTLR_EE_LITTLE_ENDIAN: u32 = 0 << 25;
const SCTLR_EOE_LITTLE_ENDIAN: u32 = 0 << 24;
const SCTLR_I_CACHE_DISABLED: u32 = 0 << 12;
const SCTLR_D_CACHE_DISABLED: u32 = 0 << 2;
const SCTLR_MMU_DISABLED: u32 = 0;
const SCTLR_MMU_ENABLED: u32 = 1;

const SCTLR_VALUE_MMU_DISABLED: u32 = (SCTLR_RESERVED
    | SCTLR_EE_LITTLE_ENDIAN
    | SCTLR_I_CACHE_DISABLED
    | SCTLR_D_CACHE_DISABLED
    | SCTLR_MMU_DISABLED);

// Assembly counterpart to this file.
global_asm!(
    include_str!("boot.s"),
    CONST_CORE_ID_MASK = const 0b11 ,
);

extern "C" {
    fn get_exception_level() -> u64;
}

#[no_mangle]
pub unsafe fn _start_rust() -> ! {
    // let el = get_exception_level();
    crate::kernel_init()
}
