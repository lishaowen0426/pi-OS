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
}

#[cfg(feature = "bsp_rpi4")]
pub mod mmio {
    use super::*;
    const START: usize = 0xFE00_0000;
    pub const GPIO_START: usize = START + GPIO_OFFSET;
    pub const UART_START: usize = START + UART_OFFSET;
    pub const MINI_UART_START: usize = START + MINI_UART_OFFSET;
}
