//! PI4 GIC Interrupt controller

use crate::memory::config;

const GIC_400_OFFSET: usize = 0xff840000 - config::PHYSICAL_PERIPHERAL_START;
const GIC_400_VIRTUAL_START: usize = config::VIRTUAL_PERIPHERAL_START + GIC_400_OFFSET;
