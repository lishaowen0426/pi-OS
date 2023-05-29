use crate::{
    bsp::{device_driver::utils::*, mmio::*, PERIPHERAL_BASE},
    cpu::nop,
};
fn mu_baud_reg(clock: u64, baud: u32) -> u32 {
    ((clock / (baud * 8) as u64) - 1) as u32
}

#[no_mangle]
pub unsafe extern "C" fn init_mini_uart() {
    mmio_write(AUX_ENABLES, 1); // enable UART1
    mmio_write(AUX_MU_IER_REG, 0);
    mmio_write(AUX_MU_CNTL_REG, 0);
    mmio_write(AUX_MU_LCR_REG, 3); // 8 bits
    mmio_write(AUX_MU_MCR_REG, 0);
    mmio_write(AUX_MU_IER_REG, 0);
    mmio_write(AUX_MU_IIR_REG, 0xC6); // disable interrupts
    mmio_write(AUX_MU_BAUD_REG, mu_baud_reg(CLOCK, BAUD_RATE));
    mmio_write(AUX_MU_CNTL_REG, 3); // enable RX/TX
                                    //
                                    // self.clear_rx();
    return;
}

use core::cell::SyncUnsafeCell;

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

const CLOCK: u64 = 500000000;
const BAUD_RATE: u32 = 115200;

pub enum BlockingMode {
    Blocking,
    NonBlocking,
}

pub struct MiniUartInner {
    chars_read: usize,
    chars_written: usize,
}

impl MiniUartInner {
    pub const fn new() -> Self {
        Self {
            chars_read: 0,
            chars_written: 0,
        }
    }
}

pub struct UnsafeMiniUart {
    inner: SyncUnsafeCell<MiniUartInner>,
}

impl UnsafeMiniUart {
    pub const fn new() -> Self {
        Self {
            inner: SyncUnsafeCell::new(MiniUartInner::new()),
        }
    }
}

impl MiniUartInner {
    fn is_writeable(&self) -> bool {
        mmio_is_set(AUX_MU_LSR_REG, 5)
    }

    fn is_idle(&self) -> bool {
        mmio_is_set(AUX_MU_LSR_REG, 6)
    }

    fn is_readable(&self) -> bool {
        mmio_is_set(AUX_MU_LSR_REG, 0)
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        while !self.is_writeable() {
            nop();
        }
        mmio_write(AUX_MU_IO_REG, c as u32);
        self.chars_written += 1;
        Ok(())
    }

    fn read_char(&mut self, blocking: BlockingMode) -> Option<char> {
        if !self.is_readable() {
            if let BlockingMode::NonBlocking = blocking {
                return None;
            } else {
                while !self.is_readable() {
                    nop();
                }
            }
        }

        let data = mmio_read(AUX_MU_IO_REG) as u8 as char;

        self.chars_read += 1;
        Some(data)
    }

    fn flush(&mut self) {
        while !self.is_idle() {
            nop();
        }
    }
}

impl core::fmt::Write for MiniUartInner {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            let _ = self.write_char(c);
        }

        Ok(())
    }
}

impl UnsafeMiniUart {
    pub fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result {
        use core::fmt::Write;
        unsafe { self.inner.get().as_mut().unwrap().write_fmt(args) }
    }
}

pub static UNSAFE_MINI_UART: UnsafeMiniUart = UnsafeMiniUart::new();
