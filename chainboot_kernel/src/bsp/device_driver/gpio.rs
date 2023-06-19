use crate::{
    bsp::device_driver::mmio_wrapper::MMIODerefWrapper, driver, synchronization::Spinlock,
};

use tock_registers::{
    interfaces::{ReadWriteable, Writeable},
    register_bitfields, register_structs,
    registers::ReadWrite,
};

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

type Registers = MMIODerefWrapper<RegisterBlock>;

struct GPIOInner {
    registers: Registers,
}

pub struct GPIO {
    inner: Spinlock<GPIOInner>,
}

impl GPIOInner {
    const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            registers: Registers::new(mmio_start_addr),
        }
    }

    #[cfg(feature = "bsp_rpi3")]
    fn disable_pud_14_15(&mut self) {
        use crate::cpu;

        // Make an educated guess for a good delay value (Sequence described in the BCM2837
        // peripherals PDF).
        //
        // - According to Wikipedia, the fastest RPi4 clocks around 1.5 GHz.
        // - The Linux 2837 GPIO driver waits 1 µs between the steps.
        //
        // So lets try to be on the safe side and default to 2000 cycles, which would equal 1 µs
        // would the CPU be clocked at 2 GHz.
        const DELAY: usize = 2000;

        self.registers.GPPUD.write(GPPUD::PUD::Off);
        cpu::spin_for_cycles(DELAY);

        self.registers
            .GPPUDCLK0
            .write(GPPUDCLK0::PUDCLK15::AssertClock + GPPUDCLK0::PUDCLK14::AssertClock);
        cpu::spin_for_cycles(DELAY);

        self.registers.GPPUD.write(GPPUD::PUD::Off);
        self.registers.GPPUDCLK0.set(0);
    }

    /// Disable pull-up/down on pins 14 and 15.
    #[cfg(feature = "bsp_rpi4")]
    fn disable_pud_14_15(&mut self) {
        self.registers.GPIO_PUP_PDN_CNTRL_REG0.write(
            GPIO_PUP_PDN_CNTRL_REG0::GPIO_PUP_PDN_CNTRL15::PullUp
                + GPIO_PUP_PDN_CNTRL_REG0::GPIO_PUP_PDN_CNTRL14::PullUp,
        );
    }

    #[cfg(feature = "bsp_rpi3")]
    fn pull_none_14_15(&mut self) {
        use crate::cpu;

        const DELAY: usize = 2000;

        self.registers.GPPUD.write(GPPUD::PUD::Off);
        cpu::spin_for_cycles(DELAY);

        self.registers
            .GPPUDCLK0
            .write(GPPUDCLK0::PUDCLK15::AssertClock + GPPUDCLK0::PUDCLK14::AssertClock);
        cpu::spin_for_cycles(DELAY);

        self.registers.GPPUD.write(GPPUD::PUD::Off);
        self.registers.GPPUDCLK0.set(0);
    }

    /// Disable pull-up/down on pins 14 and 15.
    #[cfg(feature = "bsp_rpi4")]
    fn pull_none_14_15(&mut self) {
        self.registers.GPIO_PUP_PDN_CNTRL_REG0.write(
            GPIO_PUP_PDN_CNTRL_REG0::GPIO_PUP_PDN_CNTRL15::NoResistor
                + GPIO_PUP_PDN_CNTRL_REG0::GPIO_PUP_PDN_CNTRL14::NoResistor,
        );
    }

    pub fn map_pl011_uart(&mut self) {
        // Select the UART on pins 14 and 15.
        self.registers
            .GPFSEL1
            .modify(GPFSEL1::FSEL15::AltFunc0 + GPFSEL1::FSEL14::AltFunc0);

        // Disable pull-up/down on pins 14 and 15.
        // self.disable_pud_14_15();
        self.pull_none_14_15();
    }
    pub fn map_mini_uart(&mut self) {
        // Select the UART on pins 14 and 15.
        self.pull_none_14_15();
        self.registers
            .GPFSEL1
            .modify(GPFSEL1::FSEL15::AltFunc5 + GPFSEL1::FSEL14::AltFunc5);
    }
}

impl GPIO {
    pub const COMPATIBLE: &'static str = "BCM GPIO";

    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            inner: Spinlock::new(GPIOInner::new(mmio_start_addr)),
        }
    }

    pub fn map_pl011_uart(&self) {
        let mut locked = self.inner.lock();
        locked.map_pl011_uart()
    }
    pub fn map_mini_uart(&self) {
        let mut locked = self.inner.lock();
        locked.map_mini_uart()
    }
}

impl driver::interface::DeviceDriver for GPIO {
    fn compatible(&self) -> &'static str {
        Self::COMPATIBLE
    }
}
