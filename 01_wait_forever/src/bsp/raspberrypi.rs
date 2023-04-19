// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2022 Andre Richter <andre.o.richter@gmail.com>

//! Top-level BSP file for the Raspberry Pi 3 and 4.

mod cpu;
mod driver;
mod memory;

pub use cpu::*;
pub use driver::*;
pub use memory::*;

pub fn board_name() -> &'static str {
    #[cfg(feature = "bsp_rpi3")]
    {
        "Raspberry Pi 3"
    }
    #[cfg(feature = "bsp_rpi4")]
    {
        "Raspberry Pi 4"
    }
}
