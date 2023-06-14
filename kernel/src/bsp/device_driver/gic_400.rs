//! PI4 GIC Interrupt controller
extern crate alloc;
use crate::{
    bsp::mmio,
    cpu::{timer, timer::TIMER},
    errno::ErrorCode,
    exception,
    interrupt::IRQController,
    memory::{config, MMIOWrapper},
    println,
    utils::bitfields::Bitfields,
};
use alloc::boxed::Box;
use tock_registers::{
    interfaces::{ReadWriteable, Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite, WriteOnly},
};

register_bitfields!(
    u32,
    GICD_CTLR[
        EnableGrp0 OFFSET(0) NUMBITS(1)[
            forwarded = 1,
            not_forwarded = 0,
        ],
        EnableGrp1 OFFSET(1) NUMBITS(1)[
            forwarded = 1,
            not_forwarded = 0,
        ],
    ],
    GICD_TYPER[
        ITLinesNumber OFFSET(0) NUMBITS(5)[],
        CPUNumber OFFSET(5) NUMBITS(3)[],
        SecurityExtn OFFSET(10) NUMBITS(1)[],
        LSPI OFFSET(11) NUMBITS(5)[],
    ],
    GICD_IGROUPR[
        GroupStatus OFFSET(0) NUMBITS(32)[],
    ],
    GICD_ISENABLER[
        SetEnable OFFSET(0) NUMBITS(32) [],
    ],
    GICD_ICENABLER[
        ClearEnable OFFSET(0) NUMBITS(32) [],
    ],
    GICD_ISPENDR[
        SetPending OFFSET(0) NUMBITS(32) [],
    ],
    GICD_ICPENDR[
        ClearPending OFFSET(0) NUMBITS(32) [],
    ],
    GICD_ISACTIVER[
        SetActive OFFSET(0) NUMBITS(32) [],
    ],
    GICD_ICACTIVER[
        ClearActive OFFSET(0) NUMBITS(32) [],
    ],

    GICD_IPRIORITYR[
        Priority0 OFFSET(0) NUMBITS(8)[],
        Priority1 OFFSET(8) NUMBITS(8)[],
        Priority2 OFFSET(16) NUMBITS(8)[],
        Priority3 OFFSET(24) NUMBITS(8)[],
    ],
    GICD_ITARGETSR[
        Target0 OFFSET(0) NUMBITS(8)[
            INTF0 = 0b1,
            INTF1 = 0b10,
            INTF2 = 0b100,
            INTF3 = 0b1000,
        ],
        Target1 OFFSET(8) NUMBITS(8)[
            INTF0 = 0b1,
            INTF1 = 0b10,
            INTF2 = 0b100,
            INTF3 = 0b1000,
        ],
        Target2 OFFSET(16) NUMBITS(8)[
            INTF0 = 0b1,
            INTF1 = 0b10,
            INTF2 = 0b100,
            INTF3 = 0b1000,
        ],
        Target3 OFFSET(24) NUMBITS(8)[
            INTF0 = 0b1,
            INTF1 = 0b10,
            INTF2 = 0b100,
            INTF3 = 0b1000,
        ],
    ],
    GICD_ICFGR[
        IntConfig OFFSET(0) NUMBITS(32)[],
    ]
);

register_structs!(
    GICDRegisterBlock {
        (0x000 => Ctlr: ReadWrite<u32, GICD_CTLR::Register>),
        (0x004 => Typer: ReadOnly<u32, GICD_TYPER::Register>),
        (0x008 => _reserved1),
        (0x080 => Group: [ReadWrite<u32, GICD_IGROUPR::Register>;8]),
        (0x0A0 => _reserved2),

        (0x100 => ISEnable: [ReadWrite<u32, GICD_ISENABLER::Register>;8]),
        (0x120 => _reserved3),
        (0x180 => ICEnable: [ReadWrite<u32, GICD_ICENABLER::Register>;8]),
        (0x1A0 => _reserved4),

        (0x200 => ISPend: [ReadWrite<u32, GICD_ISPENDR::Register>;8]),
        (0x220 => _reserved5),
        (0x280 => ICPend: [ReadWrite<u32, GICD_ICPENDR::Register>;8]),
        (0x2A0 => _reserved6),

        (0x300 => ISActive: [ReadWrite<u32, GICD_ISACTIVER::Register>;8]),
        (0x320 => _reserved7),
        (0x380 => ICActive: [ReadWrite<u32, GICD_ICACTIVER::Register>;8]),
        (0x3A0 => _reserved8),

        (0x400 => Priority: [ReadWrite<u32, GICD_IPRIORITYR::Register>;64]),
        (0x500 => _reserved9),

        (0x800 => Target: [ReadWrite<u32, GICD_ITARGETSR::Register>;64]),
        (0x900 => _reserved10),

        (0xC00 => Cfg: [ReadWrite<u32, GICD_ICFGR::Register>;16]),
        (0xC40 => @END),
    }
);
const P0: u8 = 0b0000_0000;
const P1: u8 = 0b0001_0000;
const P2: u8 = 0b0010_0000;
const P3: u8 = 0b0011_0000;
const P4: u8 = 0b0100_0000;
const P5: u8 = 0b0101_0000;
const P6: u8 = 0b0110_0000;
const P7: u8 = 0b0111_0000;
const P8: u8 = 0b1000_0000;
const P9: u8 = 0b1001_0000;
const P10: u8 = 0b1010_0000;
const P11: u8 = 0b1011_0000;
const P12: u8 = 0b1100_0000;
const P13: u8 = 0b1101_0000;
const P14: u8 = 0b1110_0000;
const P15: u8 = 0b1111_0000;

register_bitfields!(u32,
    GICC_CTLR [
        EnableGrp0 OFFSET(0) NUMBITS(1) [
            forwarded = 1,
            not_forwarded = 0,
        ],
        EnableGrp1 OFFSET(1) NUMBITS(1) [
            forwarded = 1,
            not_forwarded = 0,
        ],
        EOImodeS OFFSET(9) NUMBITS(1) [
            combined = 0b0,
            separated = 0b1,
        ]
    ],
    GICC_PMR[
        Priority OFFSET(0) NUMBITS(8)[
            P0  = 0b0000_0000,
            P1  = 0b0001_0000,
            P2  = 0b0010_0000,
            P3  = 0b0011_0000,
            P4  = 0b0100_0000,
            P5  = 0b0101_0000,
            P6  = 0b0110_0000,
            P7  = 0b0111_0000,
            P8  = 0b1000_0000,
            P9  = 0b1001_0000,
            P10 = 0b1010_0000,
            P11 = 0b1011_0000,
            P12 = 0b1100_0000,
            P13 = 0b1101_0000,
            P14 = 0b1110_0000,
            P15 = 0b1111_0000,
        ],
    ],
    GICC_IAR[
        InterruptID OFFSET(0) NUMBITS(10)[],
        CPUID OFFSET(10) NUMBITS(3)[],
    ],
    GICC_EOIR[
        EOIINTID OFFSET(0) NUMBITS(10) [],
        CPUID OFFSET(10) NUMBITS(3) [],
    ],
    GICC_RPR[
        Priority OFFSET(0)  NUMBITS(8)[],
    ],
    GICC_IIDR[
        Implementer OFFSET(0) NUMBITS(12)[],
        Revision OFFSET(12) NUMBITS(4)[],
        ArchVersion OFFSET(16) NUMBITS(4)[],
        ProdID OFFSET(20) NUMBITS(12)[],
    ],
);
register_structs!(GICCRegisterBlock {
    (0x000 => Ctlr: ReadWrite<u32, GICC_CTLR::Register>),
    (0x004 => Pmr: ReadWrite<u32, GICC_PMR::Register>),
    (0x008 => _reserved1),
    (0x00C => Iar: ReadOnly<u32, GICC_IAR::Register>),
    (0x010 => Eoir: WriteOnly<u32, GICC_EOIR::Register>),
    (0x014 => Rpr: ReadOnly<u32, GICC_RPR::Register>),
    (0x018 => _reserved2),
    (0x0FC => Iidr: ReadOnly<u32, GICC_IIDR::Register>),
    (0x100 => @END),
});

const GICD_OFFSET: usize = 0xFF841000 - config::PHYSICAL_PERIPHERAL_START;
const GICC_OFFSET: usize = 0xFF842000 - config::PHYSICAL_PERIPHERAL_START;
const GICD_VIRTUAL_START: usize = config::VIRTUAL_PERIPHERAL_START + GICD_OFFSET;
const GICC_VIRTUAL_START: usize = config::VIRTUAL_PERIPHERAL_START + GICC_OFFSET;

#[derive(Copy, Clone)]
pub enum IRQNum {
    PPI(u32),
    SPI(u32),
}

impl From<u32> for IRQNum {
    fn from(value: u32) -> Self {
        if value < 32 {
            Self::PPI(value)
        } else {
            Self::SPI(value)
        }
    }
}

impl IRQNum {
    fn value(&self) -> u32 {
        match *self {
            IRQNum::PPI(u) => u,
            IRQNum::SPI(u) => u,
        }
    }
}

static CORE_PS_TIMER_IRQ: IRQNum = IRQNum::PPI(30);
static VC_TIMER_IRQ: IRQNum = IRQNum::SPI(96);

pub struct IRQDescriptor {
    num: IRQNum,
    name: &'static str,
    handleFn: fn() -> Result<(), ErrorCode>,
}

static GIC400_HANDLER: [IRQDescriptor; 1] = [IRQDescriptor {
    num: CORE_PS_TIMER_IRQ,
    name: "Core Physical Timer",
    handleFn: timer::handle_interrupt,
}];
pub struct GIC400 {
    gicd: MMIOWrapper<GICDRegisterBlock>,
    gicc: MMIOWrapper<GICCRegisterBlock>,
}

fn to_group(irq: u32) -> (usize, usize) {
    (irq as usize / 32, irq as usize % 32)
}
fn to_enable(irq: u32) -> (usize, usize) {
    (irq as usize / 32, irq as usize % 32)
}
fn to_pend(irq: u32) -> (usize, usize) {
    (irq as usize / 32, irq as usize % 32)
}
fn to_priority(irq: u32) -> (usize, usize) {
    (irq as usize / 4, irq as usize % 4)
}
fn to_target(irq: u32) -> (usize, usize) {
    (irq as usize / 4, irq as usize % 4)
}

impl GIC400 {
    fn new() -> Self {
        Self {
            gicd: MMIOWrapper::new(GICD_VIRTUAL_START),
            gicc: MMIOWrapper::new(GICC_VIRTUAL_START),
        }
    }

    fn set_priority(&mut self, irq: &IRQNum, p: u8) {
        let (idx, offset) = to_priority(irq.value());
        let mut prio = self.gicd.Priority[idx].get();
        prio.set_bits(offset * 8..(offset + 1) * 8, p as u64);
        self.gicd.Priority[idx].set(prio);
    }
    fn set_target_cpu(&mut self, irq: &IRQNum, cpu: u8) {
        let (idx, offset) = to_target(irq.value());
        let mut target = self.gicd.Target[idx].get();
        target.set_bits(offset * 8..(offset + 1) * 8, (0b1 << cpu) as u64);
        self.gicd.Target[idx].set(target);
    }
    fn enable(&mut self, irq: &IRQNum) {
        let (idx, offset) = to_enable(irq.value());
        let mut enabled = self.gicd.ISEnable[idx].get();
        enabled.set_bit(offset, 1);
        self.gicd.ISEnable[idx].set(enabled);
    }
}

pub fn create() -> GIC400 {
    GIC400::new()
}
impl IRQController for GIC400 {
    fn init(&mut self) -> Result<(), ErrorCode> {
        // mask all interrupts
        exception::local_irq_mask();

        // disable GICD and GICC
        self.gicd
            .Ctlr
            .modify(GICD_CTLR::EnableGrp0::not_forwarded + GICD_CTLR::EnableGrp1::not_forwarded);
        self.gicc
            .Ctlr
            .modify(GICC_CTLR::EnableGrp0::not_forwarded + GICC_CTLR::EnableGrp1::not_forwarded);

        // set all irqs to group0
        for igroup in self.gicd.Group.iter() {
            igroup.set(0);
        }

        // disable all irqs, clear pending, clear active
        for icenable in self.gicd.ICEnable.iter() {
            icenable.set(0xFFFF_FFFF);
        }
        for icpend in self.gicd.ICPend.iter() {
            icpend.set(0xFFFF_FFFF);
        }
        for icactive in self.gicd.ICActive.iter() {
            icactive.set(0xFFFF_FFFF);
        }

        // set all IRQs for the lowest priority
        for iprio in self.gicd.Priority.iter() {
            iprio.set(0xF0_F0_F0_F0);
        }

        self.set_priority(&CORE_PS_TIMER_IRQ, P0);

        self.enable(&CORE_PS_TIMER_IRQ);

        // mask all interrupts other than ones with the highest priority
        self.gicc.Pmr.modify(GICC_PMR::Priority::P15);

        // enable GICD and GICC
        self.gicd.Ctlr.modify(GICD_CTLR::EnableGrp0::forwarded);
        self.gicc.Ctlr.modify(GICC_CTLR::EnableGrp0::forwarded);

        exception::local_irq_unmask();
        Ok(())
    }

    fn handle(&self) -> Result<(), ErrorCode> {
        let iar = self.gicc.Iar.get();
        let interrupt_id = iar & 0b11111111111;
        let cpu_id = (iar >> 10) & 0b111;
        println!("IRQ ID{}, CPU ID{}", interrupt_id, cpu_id);
        (GIC400_HANDLER[0].handleFn)().unwrap();
        self.gicc
            .Eoir
            .write(GICC_EOIR::EOIINTID.val(interrupt_id) + GICC_EOIR::CPUID.val(cpu_id));
        Ok(())
    }
}
unsafe impl Send for GIC400 {}
unsafe impl Sync for GIC400 {}
