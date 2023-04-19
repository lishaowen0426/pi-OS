use crate::{
    bsp::{device_driver::utils::*, mmio::MINI_UART_START},
    console::{self, interface::Read},
    cpu::nop,
};

use core::fmt;

const AUX_ENABLES: usize = MINI_UART_START + 0x4;
const AUX_MU_IO_REG: usize = MINI_UART_START + 0x40;
const AUX_MU_IER_REG: usize = MINI_UART_START + 0x44;
const AUX_MU_IIR_REG: usize = MINI_UART_START + 0x48;
const AUX_MU_LCR_REG: usize = MINI_UART_START + 0x4c;
const AUX_MU_MCR_REG: usize = MINI_UART_START + 0x50;
const AUX_MU_LSR_REG: usize = MINI_UART_START + 0x54;
const AUX_MU_CNTL_REG: usize = MINI_UART_START + 0x60;
const AUX_MU_BAUD_REG: usize = MINI_UART_START + 0x68;

pub enum BlockingMode {
    Blocking,
    NonBlocking,
}

pub struct MiniUart {
    chars_read: usize,
    chars_written: usize,
}

impl MiniUart {
    fn mu_baud_reg(clock: u64, baud: u32) -> u32 {
        ((clock / (baud * 8) as u64) - 1) as u32
    }

    pub const fn new() -> Self {
        Self {
            chars_read: 0,
            chars_written: 0,
        }
    }

    pub fn init(&mut self, clock: u64, baud: u32) {
        mmio_write(AUX_ENABLES, 1); // enable UART1
        mmio_write(AUX_MU_IER_REG, 0);
        mmio_write(AUX_MU_CNTL_REG, 0);
        mmio_write(AUX_MU_LCR_REG, 3); // 8 bits
        mmio_write(AUX_MU_MCR_REG, 0);
        mmio_write(AUX_MU_IER_REG, 0);
        mmio_write(AUX_MU_IIR_REG, 0xC6); // disable interrupts
        mmio_write(AUX_MU_BAUD_REG, Self::mu_baud_reg(clock, baud));
        mmio_write(AUX_MU_CNTL_REG, 3); // enable RX/TX
                                        //
                                        // self.clear_rx();
    }

    fn is_writeable(&self) -> bool {
        mmio_is_set(AUX_MU_LSR_REG, 5)
    }

    fn is_idle(&self) -> bool {
        mmio_is_set(AUX_MU_LSR_REG, 6)
    }

    fn is_readable(&self) -> bool {
        mmio_is_set(AUX_MU_LSR_REG, 0)
    }

    fn write_char(&mut self, c: char) {
        while !self.is_writeable() {
            nop();
        }
        mmio_write(AUX_MU_IO_REG, c as u32);
        self.chars_written += 1;
    }

    fn read_char_converting(&mut self, blocking: BlockingMode) -> Option<char> {
        if !self.is_readable() {
            if let BlockingMode::NonBlocking = blocking {
                return None;
            } else {
                while !self.is_readable() {
                    nop();
                }
            }
        }

        let mut data = mmio_read(AUX_MU_IO_REG) as u8 as char;

        if data == '\r' {
            data = '\n';
        }

        self.chars_read += 1;
        Some(data)
    }

    pub fn flush(&mut self) {
        while !self.is_idle() {
            nop();
        }
    }
}

impl console::interface::Write for MiniUart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }

        Ok(())
    }
}

impl console::interface::Read for MiniUart {
    fn read_char(&mut self) -> char {
        self.read_char_converting(BlockingMode::Blocking).unwrap()
    }

    fn clear_rx(&mut self) {
        while self
            .read_char_converting(BlockingMode::NonBlocking)
            .is_some()
        {}
    }
}

impl console::interface::Statistics for MiniUart {
    fn chars_read(&self) -> usize {
        self.chars_read
    }

    fn chars_written(&self) -> usize {
        self.chars_written
    }
}

impl console::interface::All for MiniUart {}
