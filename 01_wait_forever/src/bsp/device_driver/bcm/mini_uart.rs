use crate::{
    bsp::device_driver::common::MMIODerefWrapper, console, cpu, driver, synchronization,
    synchronization::Spinlock,
};
use core::fmt;
use tock_registers::{
    interfaces::{ReadWriteable, Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite, WriteOnly},
};

const CLOCK: u64 = 500000000;
const BAUD_RATE: u64 = 115200;

#[derive(PartialEq)]
enum BlockingMode {
    Blocking,
    NonBlocking,
}

register_bitfields! {

    u32,

    ENABLES[
        MINI_UART OFFSET(0) NUMBITS(1)[
            Unset = 0b0,
            Set = 0b1
        ],
    ],


    IER[
        TRANSMIT_INTERRUPT OFFSET(0) NUMBITS(1)[
            Unset = 0,
            Set = 1
        ],
        RECEIVE_INTERRUPT OFFSET(1) NUMBITS(1)[
            Unset = 0,
            Set = 1
        ],
    ],

    IIR[
        FIFO OFFSET(1) NUMBITS(2) [
            Clear = 0b11
        ]
    ],

    LCR[
        DATA_SIZE OFFSET(0) NUMBITS(1) [
            SevenBits = 0b0,
            EightBits = 0b1,
        ],
    ],

    MCR[
        RTS OFFSET(1) NUMBITS(1) [
            Unset = 0,
            Set = 1,
        ],
    ],

    LSR[
        READY OFFSET(0) NUMBITS(1)[
            Unset = 0,
            Set = 1
        ],
        OVERRUN OFFSET(1) NUMBITS(1)[
            Unset = 0,
            Set = 1
        ],
        EMPTY OFFSET(5) NUMBITS(1)[
            Unset = 0,
            Set = 1
        ],
        IDLE OFFSET(6) NUMBITS(1)[
            Unset = 0,
            Set = 1
        ],

    ],

    MSR[
        CTS OFFSET(4) NUMBITS(1)[
            Unset = 0,
            Set = 1
        ],
    ],


    CNTL[
        RECEIVER OFFSET(0) NUMBITS(1)[
            Unset = 0,
            Set = 1
        ],
        TRANSMITTER OFFSET(1) NUMBITS(1)[
            Unset = 0,
            Set = 1
        ]
    ],


    BAUD[
        BAUD_RATE OFFSET(0) NUMBITS(16) []
    ],

}

register_structs! {
    #[allow(non_snake_case)]
    pub RegisterBlock {
        (0x00 => _reserved1),
        (0x04 => pub ENABLES: ReadWrite<u32> ),
        (0x08 => _reserved2),
        (0x40 => pub IO: ReadWrite<u32>),
        (0x44 => pub  IER: ReadWrite<u32, IER::Register>),
        (0x48 => pub  IIR: ReadWrite<u32>),
        (0x4c => pub LCR: ReadWrite<u32>),
        (0x50 => pub MCR: ReadWrite<u32, MCR::Register>),
        (0x54 => pub LSR: ReadOnly<u32, LSR::Register>),
        (0x58 => pub MSR: ReadOnly<u32>),
        (0x5c => _reserved3),
        (0x60 => pub CNTL: ReadWrite<u32, CNTL::Register>),
        (0x64 => _reserved4),
        (0x68 => pub BAUD: WriteOnly<u32, BAUD::Register>),
        (0x6c => @END),
    }
}

type Registers = MMIODerefWrapper<RegisterBlock>;

pub type MiniRegisters = MMIODerefWrapper<RegisterBlock>;

struct MiniUartInner {
    registers: Registers,
    chars_written: usize,
    chars_read: usize,
}

pub struct MiniUart {
    inner: Spinlock<MiniUartInner>,
}

impl MiniUartInner {
    const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            registers: Registers::new(mmio_start_addr),
            chars_written: 0,
            chars_read: 0,
        }
    }

    fn mu_baud_reg(&self, clock: u64, baud_rate: u64) -> u16 {
        let baud_reg = (clock / (8 * baud_rate) - 1) as u16;
        baud_reg
    }

    fn init(&mut self) {
        self.registers.ENABLES.set(1);
        self.registers
            .IER
            .modify(IER::TRANSMIT_INTERRUPT::Unset + IER::RECEIVE_INTERRUPT::Unset);
        self.registers.CNTL.set(0);
        self.registers.LCR.set(3);
        self.registers.MCR.modify(MCR::RTS::CLEAR);
        self.registers.IER.set(0);
        self.registers.IIR.set(0xC6);
        self.registers
            .BAUD
            .write(BAUD::BAUD_RATE.val(self.mu_baud_reg(CLOCK, BAUD_RATE) as u32));
        self.registers
            .CNTL
            .modify(CNTL::RECEIVER::Set + CNTL::TRANSMITTER::Set);
    }

    fn flush(&self) {
        while self.registers.LSR.matches_all(LSR::IDLE::Unset) {
            cpu::nop();
        }
    }

    fn write_char(&mut self, c: char) {
        while self.registers.LSR.matches_all(LSR::EMPTY::Unset) {
            cpu::nop();
        }

        self.registers.IO.set(c as u32);
        self.chars_written += 1;
    }
    fn read_char_converting(&mut self, blocking_mode: BlockingMode) -> Option<char> {
        if self.registers.LSR.matches_all(LSR::EMPTY::Unset) {
            if blocking_mode == BlockingMode::NonBlocking {
                return None;
            }
            while self.registers.LSR.matches_all(LSR::EMPTY::Unset) {
                cpu::nop();
            }
        }

        let mut ret = self.registers.IO.get() as u8 as char;

        if ret == '\r' {
            ret = '\n'
        }

        Some(ret)
    }
}
impl fmt::Write for MiniUartInner {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }
        Ok(())
    }
}

impl MiniUart {
    pub const COMPATIBLE: &'static str = "BCM Mini UART";

    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            inner: Spinlock::new(MiniUartInner::new(mmio_start_addr)),
        }
    }
}

impl driver::interface::DeviceDriver for MiniUart {
    fn compatible(&self) -> &'static str {
        Self::COMPATIBLE
    }

    unsafe fn init(&self) -> Result<(), &'static str> {
        let mut locked = self.inner.lock();
        locked.init();

        Ok(())
    }
}

impl console::interface::Write for MiniUart {
    fn write_char(&self, c: char) {
        self.inner.lock().write_char(c);
    }

    fn write_fmt(&self, args: core::fmt::Arguments) -> fmt::Result {
        let mut locked = self.inner.lock();
        fmt::Write::write_fmt(&mut (*locked), args)
    }

    fn flush(&self) {
        self.inner.lock().flush();
    }
}

impl console::interface::Read for MiniUart {
    fn read_char(&self) -> char {
        self.inner
            .lock()
            .read_char_converting(BlockingMode::Blocking)
            .unwrap()
    }

    fn clear_rx(&self) {
        while self
            .inner
            .lock()
            .read_char_converting(BlockingMode::NonBlocking)
            .is_some()
        {}
    }
}

impl console::interface::Statistics for MiniUart {
    fn chars_written(&self) -> usize {
        self.inner.lock().chars_written
    }
    fn chars_read(&self) -> usize {
        self.inner.lock().chars_read
    }
}

impl console::interface::All for MiniUart {}
