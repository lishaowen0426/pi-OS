// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020-2022 Andre Richter <andre.o.richter@gmail.com>

//! Processor code.

#[cfg(target_arch = "aarch64")]
#[path = "_arch/aarch64/cpu.rs"]
mod arch_cpu;
pub use arch_cpu::*;

#[cfg(target_arch = "aarch64")]
#[path = "_arch/aarch64/timer.rs"]
pub mod arch_timer;

pub use arch_timer as timer;

#[cfg(feature = "bsp_rpi3")]
pub use arch_cpu::spin_for_cycles;

#[cfg(feature = "test_build")]
pub use arch_cpu::{qemu_exit_failure, qemu_exit_success};
