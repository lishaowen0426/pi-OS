use crate::{
    bsp::{device_driver::utils::*, mmio::MINI_UART_START},
    console,
    cpu::nop,
};

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

struct MiniUartInner {
    chars_read: usize,
    chars_written: usize,
}

impl MiniUartInner {
    const fn new() -> Self {
        Self {
            chars_read: 0,
            chars_written: 0,
        }
    }
}

pub struct MiniUart {
    inner: SyncUnsafeCell<MiniUartInner>,
}

impl MiniUart {
    pub const fn new() -> Self {
        Self {
            inner: SyncUnsafeCell::new(MiniUartInner::new()),
        }
    }
}

impl MiniUartInner {
    fn mu_baud_reg(clock: u64, baud: u32) -> u32 {
        ((clock / (baud * 8) as u64) - 1) as u32
    }

    fn init(&mut self) {
        mmio_write(AUX_ENABLES, 1); // enable UART1
        mmio_write(AUX_MU_IER_REG, 0);
        mmio_write(AUX_MU_CNTL_REG, 0);
        mmio_write(AUX_MU_LCR_REG, 3); // 8 bits
        mmio_write(AUX_MU_MCR_REG, 0);
        mmio_write(AUX_MU_IER_REG, 0);
        mmio_write(AUX_MU_IIR_REG, 0xC6); // disable interrupts
        mmio_write(AUX_MU_BAUD_REG, Self::mu_baud_reg(CLOCK, BAUD_RATE));
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

impl console::interface::Console for MiniUart {
    fn init(&self) {
        let data = unsafe { &mut *self.inner.get() };
        data.init()
    }
    fn _write_str(&self, s: &str) -> fmt::Result {
        use core::fmt::Write;
        let data = unsafe { &mut *self.inner.get() };
        data.write_str(s)
    }
    fn _write_char(&self, c: char) -> fmt::Result {
        let data = unsafe { &mut *self.inner.get() };
        data.write_char(c)
    }
    fn _write_fmt(&self, args: fmt::Arguments) -> fmt::Result {
        use core::fmt::Write;
        let data = unsafe { &mut *self.inner.get() };
        data.write_fmt(args)
    }
    fn _flush(&self) {
        let data = unsafe { &mut *self.inner.get() };
        data.flush()
    }

    fn _read_char(&self) -> char {
        let data = unsafe { &mut *self.inner.get() };
        data.read_char(BlockingMode::Blocking).unwrap()
    }
    fn _clear_rx(&self) {
        let data = unsafe { &mut *self.inner.get() };
        while data.read_char(BlockingMode::NonBlocking).is_some() {}
    }
    fn _chars_written(&self) -> usize {
        let data = unsafe { &mut *self.inner.get() };
        data.chars_written
    }
    fn _chars_read(&self) -> usize {
        let data = unsafe { &mut *self.inner.get() };
        data.chars_read
    }
}

pub static MINI_UART: MiniUart = MiniUart::new();
