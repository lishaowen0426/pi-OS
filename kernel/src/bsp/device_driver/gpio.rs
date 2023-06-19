use crate::{
    bsp::mmio,
    driver,
    errno::ErrorCode,
    memory::{config, MMIOWrapper},
    synchronization::Spinlock,
};
use tock_registers::{
    interfaces::{ReadWriteable, Writeable},
    register_bitfields, register_structs,
    registers::ReadWrite,
};

use spin::once::Once;

const GPIO_VIRTUAL_START: usize = config::VIRTUAL_PERIPHERAL_START + mmio::GPIO_OFFSET;

register_bitfields! {
    u32,


    GPFSEL1 [
        FSEL15 OFFSET(15) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AltFunc0 = 0b100,
            AltFunc5 = 0b010
        ],
        FSEL14 OFFSET(12) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AltFunc0 = 0b100,
            AltFunc5 = 0b010
        ],

    ],



    GPIO_PUP_PDN_CNTRL_REG0 [
        GPIO_PUP_PDN_CNTRL15 OFFSET(30) NUMBITS(2)[
            NoResistor = 0b00,
            PullUp = 0b01,
            PullDown = 0b10
        ],
        GPIO_PUP_PDN_CNTRL14 OFFSET(28) NUMBITS(2)[
            NoResistor = 0b00,
            PullUp = 0b01,
            PullDown = 0b10
        ],

    ],

}

register_structs! {
    #[allow(non_snake_case)]
    RegisterBlock{

        (0x00 => _reserved1),
        (0x04 => GPFSEL1: ReadWrite<u32, GPFSEL1::Register>),
        (0x08 => _reserved2),
        (0xE4 => GPIO_PUP_PDN_CNTRL_REG0: ReadWrite<u32, GPIO_PUP_PDN_CNTRL_REG0::Register>),
        (0xE8 => @END),


    }
}

type Registers = MMIOWrapper<RegisterBlock>;

struct UnsafeGPIO {
    registers: Registers,
}

pub struct GPIOController {
    inner: Spinlock<UnsafeGPIO>,
}

impl UnsafeGPIO {
    const fn new(mmio_start_addr: usize) -> Self {
        Self {
            registers: Registers::new(mmio_start_addr),
        }
    }

    fn init(&self) {
        self.registers.GPIO_PUP_PDN_CNTRL_REG0.modify(
            GPIO_PUP_PDN_CNTRL_REG0::GPIO_PUP_PDN_CNTRL14::PullDown
                + GPIO_PUP_PDN_CNTRL_REG0::GPIO_PUP_PDN_CNTRL15::PullDown,
        );
        self.registers
            .GPFSEL1
            .modify(GPFSEL1::FSEL14::AltFunc5 + GPFSEL1::FSEL15::AltFunc5);
    }

    /// Disable pull-up/down on pins 14 and 15.
    #[cfg(feature = "bsp_rpi4")]
    #[allow(dead_code)]
    fn disable_pud_14_15(&mut self) {
        self.registers.GPIO_PUP_PDN_CNTRL_REG0.write(
            GPIO_PUP_PDN_CNTRL_REG0::GPIO_PUP_PDN_CNTRL15::PullUp
                + GPIO_PUP_PDN_CNTRL_REG0::GPIO_PUP_PDN_CNTRL14::PullUp,
        );
    }
}

impl GPIOController {
    const fn new(mmio_start_addr: usize) -> Self {
        Self {
            inner: Spinlock::new(UnsafeGPIO::new(mmio_start_addr)),
        }
    }

    fn init(&self) {
        self.inner.lock().init()
    }
}

pub static GPIO: Once<GPIOController> = Once::new();
pub fn init() -> Result<(), ErrorCode> {
    GPIO.call_once(|| GPIOController::new(GPIO_VIRTUAL_START));
    GPIO.get().unwrap().init();
    Ok(())
}
