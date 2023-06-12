//! The physical memory layout designated by the linker script
//!
//! The Raspberry's firmware copies the kernel binary to 0x8_0000. The preceding region will be used
//! as the boot core's stack.
//!
//! +---------------------------------------+
//! |                                       | 0x0
//! |                                       |                                ^
//! | Boot-core Stack                       |                                | stack
//! |                                       |                                | growth
//! |                                       |                                | direction
//! +---------------------------------------+
//! |                                       | code_start @ 0x8_0000
//! | .text                                 |
//! | .rodata                               |
//! |                                       |
//! +---------------------------------------+
//! |                                       | code_end_exclusive
//! | .data                                 |
//! | .bss                                  |
//! |                                       |
//! +---------------------------------------+
//! | .got                                  |
//! |                                       |

#[cfg(feature = "bsp_rpi3")]
pub mod mmio {
    use super::*;

    pub const PHYSICAL_PERIPHERAL_START: usize = 0x3F00_0000;
    pub const PHYSICAL_MEMORY_END_INCLUSIVE: usize = 0x3FFF_FFFF;
    pub const PHYSICAL_MEMORY_END_EXCLUSIVE: usize = PHYSICAL_MEMORY_END_INCLUSIVE + 1;

    pub const GPIO_OFFSET: usize = 0x0020_0000;
    pub const UART_OFFSET: usize = 0x0020_1000;
    pub const MINI_UART_OFFSET: usize = 0x0021_5000;
    pub const IC_OFFSET: usize = 0x3F00B000 - PHYSICAL_PERIPHERAL_START;
}

#[cfg(feature = "bsp_rpi4")]
pub mod mmio {
    use super::*;

    pub const PHYSICAL_PERIPHERAL_START: usize = 0xFE00_0000;
    pub const PHYSICAL_MEMORY_END_INCLUSIVE: usize = 0xFFFF_FFFF; // we assume pi4 has 4GB memory
    pub const PHYSICAL_MEMORY_END_EXCLUSIVE: usize = PHYSICAL_MEMORY_END_INCLUSIVE + 1;

    pub const GPIO_OFFSET: usize = 0x0020_0000;
    pub const UART_OFFSET: usize = 0x0020_1000;
    pub const MINI_UART_OFFSET: usize = 0x0021_5000;
    pub const IC_OFFSET: usize = 0xFF840000 - PHYSICAL_PERIPHERAL_START;
}
