use crate::{
    bsp::{device_driver::utils::*, mmio},
    cpu::nop,
    errno::ErrorCode,
    memory::{config, MMIOWrapper},
    synchronization::Spinlock,
};
use core::{fmt, fmt::Write};
use spin::once::Once;
use tock_registers::{
    interfaces::{ReadWriteable, Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite, WriteOnly},
};
const MINI_UART_OFFSET: usize = 0x0021_5000;
const VIRTUAL_MINI_UART_START: usize = config::VIRTUAL_PERIPHERAL_START + mmio::MINI_UART_OFFSET;

register_bitfields!(u32,
    AUX_IRQ[
        MINI_UART_IRQ OFFSET(0) NUMBITS(1)[],
        SPI_1_IRQ OFFSET(1) NUMBITS(1)[],
        SPI_2_IRQ OFFSET(2) NUMBITS(1)[],
    ],
    AUX_ENABLES[
        MINI_UART OFFSET(0) NUMBITS(1)[],
        SPI_1 OFFSET(1) NUMBITS(1)[],
        SPI_2 OFFSET(2) NUMBITS(1)[],
    ],
    AUX_MU_IO_REG[
        DATA OFFSET(0) NUMBITS(8) [],
    ],
    AUX_MU_IER_REG[
        TRANSMIT_INTERRUPT OFFSET(0) NUMBITS(1)[],
        RECEIVE_INTERRUPT OFFSET(1) NUMBITS(1)[],
    ],
    AUX_MU_IIR_REG[
        INTERRUPT_PENDING OFFSET(0) NUMBITS(1) [],
        INTERRUPT_ID OFFSET(1) NUMBITS(2) [],
    ],
    AUX_MU_LCR_REG[
        DATA_SIZE OFFSET(0) NUMBITS(1) [],
        BREAK OFFSET(6) NUMBITS(1) [],
        DLAB OFFSET(7) NUMBITS(1) [],
    ],
    AUX_MU_MCR_REG[
        RTS OFFSET(1) NUMBITS(1)[],
    ],
    AUX_MU_LSR_REG[
        DATA_READY OFFSET(0) NUMBITS(1)[],
        RECEIVER_OVERRUN OFFSET(0) NUMBITS(1) [],
        TRANSMIT_EMPTY OFFSET(5) NUMBITS(1) [],
        TRANSMIT_IDLE OFFSET(6) NUMBITS(1) [],
    ],
    AUX_MU_MSR_REG[
        CTS OFFSET(4) NUMBITS(1)[],
    ],
    AUX_MU_CNTL_REG[
        RECEIVER_ENABLE OFFSET(0) NUMBITS(1)[],
        TRANSMITTER_ENABLE OFFSET(1) NUMBITS(1)[],
        RECEIVER_FLOW_CONTROL OFFSET(2) NUMBITS(1)[],
        TRANSMITTER_FLOW_CONTROL OFFSET(3) NUMBITS(1)[],
        RTS_LEVEL OFFSET(4) NUMBITS(2) [],
        RTS_ASSERT_LEVEL OFFSET(6) NUMBITS(1) [],
        CTS_ASSERT_LEVEL OFFSET(7) NUMBITS(1) [],
    ],
    AUX_MU_STAT_REG[
        SYMBOL_AVAILABLE OFFSET(0) NUMBITS(1)[],
        SPACE_AVAILABLE OFFSET(1) NUMBITS(1)[],
        RECEIVER_IDLE OFFSET(2) NUMBITS(1)[],
        TRANSMITTER_IDLE OFFSET(3) NUMBITS(1)[],
        RECEIVER_OVERRUN OFFSET(4) NUMBITS(1)[],
        TRANSMIT_FIFO_FULL OFFSET(5) NUMBITS(1) [],
        RTS OFFSET(6) NUMBITS(1) [],
        CTS OFFSET(7) NUMBITS(1) [],
        TRANSMIT_FIFO_EMPTY OFFSET(8) NUMBITS(1) [],
        TRANSMITTER_DONE OFFSET(9) NUMBITS(1) [],
        RECEIVE_FIFO_FILL_LEVEL OFFSET(16) NUMBITS(4) [],
        TRANSMIT_FIFO_FILL_LEVEL OFFSET(24) NUMBITS(4) [],
    ],
    AUX_MU_BAUD_REG[
        BAUDRATE OFFSET(0) NUMBITS(16) [],
    ],
);

register_structs! {
    RegisterBlock{
        (0x00 => IRQ: ReadOnly<u32, AUX_IRQ::Register>),
        (0x04 => ENABLES: ReadWrite<u32, AUX_ENABLES::Register>),
        (0x08 => _reserved1),
        (0x40 => IO: ReadWrite<u32, AUX_MU_IO_REG::Register>),
        (0x44 => IER: ReadWrite<u32, AUX_MU_IER_REG::Register>),
        (0x48 => IIR: ReadWrite<u32, AUX_MU_IIR_REG::Register>),
        (0x4C => LCR: ReadWrite<u32, AUX_MU_LCR_REG::Register>),
        (0x50 => MCR: ReadWrite<u32, AUX_MU_MCR_REG::Register>),
        (0x54 => LSR: ReadOnly<u32, AUX_MU_LSR_REG::Register>),
        (0x58 => MSR: ReadOnly<u32, AUX_MU_MSR_REG::Register>),
        (0x5C => _reserved2),
        (0x60 => CNTL: ReadWrite<u32, AUX_MU_CNTL_REG::Register>),
        (0x64 => STAT: ReadOnly<u32, AUX_MU_STAT_REG::Register>),
        (0x68 => BAUD: ReadWrite<u32, AUX_MU_BAUD_REG::Register>),
        (0x6C => @END),
    }
}

fn mu_baud_reg(clock: u64, baud: u32) -> u32 {
    ((clock / (baud * 8) as u64) - 1) as u32
}

#[no_mangle]
pub unsafe extern "C" fn init_mini_uart() {
    // mmio_write(AUX_ENABLES - VIRTUAL_PHYSICAL_DIFF, 1); // enable UART1
    // mmio_write(AUX_MU_IER_REG - VIRTUAL_PHYSICAL_DIFF, 0);
    // mmio_write(AUX_MU_CNTL_REG - VIRTUAL_PHYSICAL_DIFF, 0);
    // mmio_write(AUX_MU_LCR_REG - VIRTUAL_PHYSICAL_DIFF, 3); // 8 bits
    // mmio_write(AUX_MU_MCR_REG - VIRTUAL_PHYSICAL_DIFF, 0);
    // mmio_write(AUX_MU_IER_REG - VIRTUAL_PHYSICAL_DIFF, 0);
    // mmio_write(AUX_MU_IIR_REG - VIRTUAL_PHYSICAL_DIFF, 0xC6); // disable interrupts
    // mmio_write(
    //     AUX_MU_BAUD_REG - VIRTUAL_PHYSICAL_DIFF,
    //     mu_baud_reg(CLOCK, BAUD_RATE),
    // );
    // mmio_write(AUX_MU_CNTL_REG - VIRTUAL_PHYSICAL_DIFF, 3); // enable RX/TX
    //                                                         //
    //                                                         // self.clear_rx();
    // return;
}

const CLOCK: u64 = 500000000;
const BAUD_RATE: u32 = 115200;

pub enum BlockingMode {
    Blocking,
    NonBlocking,
}

struct UnSafeMiniUart {
    reg: MMIOWrapper<RegisterBlock>,
    chars_read: usize,
    chars_written: usize,
}

pub struct MiniUart {
    inner: Spinlock<UnSafeMiniUart>,
}

impl UnSafeMiniUart {
    fn new(mmio_start_addr: usize) -> Self {
        Self {
            reg: MMIOWrapper::new(mmio_start_addr),
            chars_read: 0,
            chars_written: 0,
        }
    }

    fn init(&mut self) {
        self.reg.ENABLES.modify(AUX_ENABLES::MINI_UART.val(1)); // m-uart needs to be enabled for
                                                                // accessing its reg. GPIO should be set up beforehand
        self.reg
            .BAUD
            .modify(AUX_MU_BAUD_REG::BAUDRATE.val(mu_baud_reg(CLOCK, BAUD_RATE)));

        self.reg.LCR.modify(
            AUX_MU_LCR_REG::DATA_SIZE.val(1)
                + AUX_MU_LCR_REG::BREAK.val(0)
                + AUX_MU_LCR_REG::DLAB.val(0),
        ); // 8bit mode

        self.reg.IER.modify(
            AUX_MU_IER_REG::TRANSMIT_INTERRUPT.val(0) + AUX_MU_IER_REG::RECEIVE_INTERRUPT.val(0),
        ); // disable m-uart interrupts

        self.reg.MCR.modify(AUX_MU_MCR_REG::RTS.val(0)); // RTS is always high(asserted). This essentially disables receiver flow

        self.reg.CNTL.modify(
            AUX_MU_CNTL_REG::TRANSMITTER_ENABLE.val(1)
                + AUX_MU_CNTL_REG::RECEIVER_ENABLE.val(1)
                + AUX_MU_CNTL_REG::RECEIVER_FLOW_CONTROL.val(0)
                + AUX_MU_CNTL_REG::TRANSMITTER_FLOW_CONTROL.val(0),
        ); // enable rx/tx, disable auto flow-control
    }

    fn is_writeable(&self) -> bool {
        self.reg.STAT.is_set(AUX_MU_STAT_REG::SPACE_AVAILABLE)
    }

    fn is_readable(&self) -> bool {
        self.reg.STAT.is_set(AUX_MU_STAT_REG::SYMBOL_AVAILABLE)
    }

    fn is_transmitter_done(&self) -> bool {
        self.reg.STAT.is_set(AUX_MU_STAT_REG::TRANSMITTER_DONE)
    }

    fn send_byte(&mut self, b: u8) {
        while !self.is_writeable() {
            nop();
        }
        self.reg.IO.modify(AUX_MU_IO_REG::DATA.val(b as u32));
    }

    fn read_byte(&mut self) -> u8 {
        while !self.is_readable() {
            nop();
        }
        self.reg.IO.read(AUX_MU_IO_REG::DATA) as u8
    }

    fn flush(&mut self) {
        while !self.is_transmitter_done() {
            nop();
        }
    }
}

impl MiniUart {
    fn new(mmio_start_addr: usize) -> Self {
        Self {
            inner: Spinlock::new(UnSafeMiniUart::new(mmio_start_addr)),
        }
    }

    fn init(&self) {
        self.inner.lock().init()
    }
    pub fn send_byte(&self, b: u8) {
        self.inner.lock().send_byte(b)
    }

    pub fn read_byte(&self) -> u8 {
        self.inner.lock().read_byte()
    }

    pub fn flush(&self) {
        self.inner.lock().flush()
    }

    pub fn write_str(&self, s: &str) -> fmt::Result {
        self.inner.lock().write_str(s)
    }

    pub fn write_fmt(&self, args: fmt::Arguments<'_>) -> fmt::Result {
        self.inner.lock().write_fmt(args)
    }

    pub fn write_char(&self, c: char) -> fmt::Result {
        self.inner.lock().write_char(c)
    }
}

impl fmt::Write for UnSafeMiniUart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.as_bytes() {
            self.send_byte(*b);
        }

        Ok(())
    }
}

pub fn init() -> Result<(), ErrorCode> {
    MINI_UART.call_once(|| MiniUart::new(VIRTUAL_MINI_UART_START));
    MINI_UART.get().unwrap().init();
    Ok(())
}

pub static MINI_UART: Once<MiniUart> = Once::new();
