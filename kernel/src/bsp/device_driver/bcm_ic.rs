//! PI3 BCM Interrupt controller

use crate::{
    bsp::mmio,
    cpu::timer::TIMER,
    interrupt::IRQController,
    memory::{config, MMIOWrapper},
};

use tock_registers::{
    interfaces::{ReadWriteable, Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite, WriteOnly},
};

use crate::{errno::ErrorCode, synchronization::IRQSafeSpinlock};
use aarch64_cpu::registers::*;

const IC_VIRTUAL_START: usize = config::VIRTUAL_PERIPHERAL_START + mmio::IC_OFFSET;

register_bitfields!(u32,
    FIQControl[
        SOURCE OFFSET(0) NUMBITS(7) [
            aux_int = 29,
            i2c_spi_slv_int = 43,
            pwa0 = 45,
            pwa1 = 46,
            smi = 48,
            gpio_int0 = 49,
            gpio_int1 = 50,
            gpio_int2 = 51,
            gpio_int3 = 52,
            i2c_int = 53,
            spi_int = 54,
            pcm_int = 55,
            uart_int = 57,
            arm_timer = 64,
            arm_mailbox = 65,
            arm_doorbell0 = 66,
            arm_doorbell1 = 67,
            gpu_halted0 = 68,
            gpu_halted1 = 69,
            illegal_access1 = 70,
            illegal_access0 = 71,

        ],
        ENBALE OFFSET(7) NUMBITS(1) [ON = 1, OFF = 0,]
    ],

    BasicPendingIRQ[
         ARMTimer OFFSET(0) NUMBITS(1) [],
         ARMMailBox OFFSET(1) NUMBITS(1) [],
         ARMDoorBell0 OFFSET(2) NUMBITS(1) [],
         ARMDoorBell1 OFFSET(3) NUMBITS(1) [],
         GPUHalted0 OFFSET(4) NUMBITS(1) [],
         GPUHalted1 OFFSET(5) NUMBITS(1) [],
         AccessErrorType1 OFFSET(6) NUMBITS(1) [],
         AccessErrorType0 OFFSET(7) NUMBITS(1) [],
         Pending1 OFFSET(8) NUMBITS(1) [],
         Pending2 OFFSET(9) NUMBITS(1) [],
         I2C_INT OFFSET(15) NUMBITS(1) [],
         SPI_INT OFFSET(16) NUMBITS(1) [],
         PCM_INT OFFSET(17) NUMBITS(1) [],
         UART_INT OFFSET(19) NUMBITS(1) [],
    ],

    IRQ1 [
        SOURCE OFFSET(0) NUMBITS(32) [
            aux_int = 29,
        ],
    ],
    IRQ2 [
        SOURCE OFFSET(0) NUMBITS(32) [
            i2c_spi_slv_int = 43,
            pwa0 = 45,
            pwa1 = 46,
            smi = 48,
            gpio_int0 = 49,
            gpio_int1 = 50,
            gpio_int2 = 51,
            gpio_int3 = 52,
            i2c_int = 53,
            spi_int = 54,
            pcm_int = 55,
            uart_int = 57,
        ],
    ],

    BasicIRQ [
         ARMTimer OFFSET(0) NUMBITS(1) [],
         ARMMailBox OFFSET(1) NUMBITS(1) [],
         ARMDoorBell0 OFFSET(2) NUMBITS(1) [],
         ARMDoorBell1 OFFSET(3) NUMBITS(1) [],
         GPUHalted0 OFFSET(4) NUMBITS(1) [],
         GPUHalted1 OFFSET(5) NUMBITS(1) [],
         AccessErrorType1 OFFSET(6) NUMBITS(1) [],
         AccessErrorType0 OFFSET(7) NUMBITS(1) [],
    ],
);

register_structs!(
    #[allow(non_snake_case)]
    RORegisterBlock {
        (0x000 => BasicPending: ReadOnly<u32, BasicPendingIRQ::Register>),
        (0x004 => Pending1: ReadOnly<u32, IRQ1::Register>),
        (0x008 => Pending2: ReadOnly<u32, IRQ2::Register>),
        (0x00C => @END),
    }
);

register_structs!(
    #[allow(non_snake_case)]
    RWRegisterBlock {
        (0x000 => _reserved),
        (0x00C => FIQControl: ReadWrite<u32, FIQControl::Register> ),
        (0x010 => Enable1: ReadWrite<u32, IRQ1::Register>),
        (0x014 => Enable2: ReadWrite<u32, IRQ2::Register>),
        (0x018 => EnableBasic: ReadWrite<u32, BasicIRQ::Register>),
        (0x01C => Disable1: ReadWrite<u32, IRQ1::Register>),
        (0x020 => Disable2: ReadWrite<u32, IRQ2::Register>),
        (0x024 => DisableBasic: ReadWrite<u32, BasicIRQ::Register>),
        (0x028 => @END),
    }
);

pub enum IRQNum {
    ARM(u8),
    GPU(u8),
}

pub struct BCMIC {
    ro_reg: MMIOWrapper<RORegisterBlock>,
    rw_reg: IRQSafeSpinlock<MMIOWrapper<RWRegisterBlock>>,
}

impl BCMIC {
    fn new() -> Self {
        Self {
            ro_reg: MMIOWrapper::new(IC_VIRTUAL_START),
            rw_reg: IRQSafeSpinlock::new(MMIOWrapper::new(IC_VIRTUAL_START)),
        }
    }
}

pub fn create() -> BCMIC {
    BCMIC::new()
}

impl IRQController for BCMIC {
    fn init(&mut self) -> Result<(), ErrorCode> {
        Ok(())
    }
    fn handle(&self) -> Result<(), ErrorCode> {
        Ok(())
    }
}
unsafe impl Send for BCMIC {}
unsafe impl Sync for BCMIC {}
