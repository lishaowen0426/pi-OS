// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020-2022 Andre Richter <andre.o.richter@gmail.com>

//! Processor code.

mod boot;

#[cfg(target_arch = "aarch64")]
#[path = "_arch/aarch64/cpu.rs"]
mod arch_cpu;

pub use arch_cpu::{nop, wait_forever};

#[cfg(feature = "bsp_rpi3")]
pub use arch_cpu::spin_for_cycles;
