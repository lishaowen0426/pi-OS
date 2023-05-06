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

const GPIO_OFFSET: usize = 0x0020_0000;
const UART_OFFSET: usize = 0x0020_1000;
const MINI_UART_OFFSET: usize = 0x0021_5000;

#[cfg(feature = "bsp_rpi3")]
pub mod mmio {
    use super::*;

    const START: usize = 0x3F00_0000;
    pub const GPIO_START: usize = START + GPIO_OFFSET;
    pub const UART_START: usize = START + UART_OFFSET;
    pub const MINI_UART_START: usize = START + MINI_UART_OFFSET;
    pub const PHYSICAL_MEMORY_END_INCLUSIVE: usize = 0x40000_0000;
}

#[cfg(feature = "bsp_rpi4")]
pub mod mmio {
    use super::*;

    const START: usize = 0xFE00_0000;
    pub const GPIO_START: usize = START + GPIO_OFFSET;
    pub const UART_START: usize = START + UART_OFFSET;
    pub const MINI_UART_START: usize = START + MINI_UART_OFFSET;
    pub const PHYSICAL_MEMORY_END_INCLUSIVE: usize = 0xFFFF_FFFF; // we assume pi4 has 4GB memory
}
