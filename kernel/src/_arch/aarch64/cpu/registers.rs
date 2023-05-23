use crate::{__read_raw, __write_raw, sys_coproc_read_raw, sys_coproc_write_raw};
use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields,
};
register_bitfields! {u64,
    pub CTR_EL0 [
       TminLine OFFSET(32) NUMBITS(6) [],
       DIC OFFSET(29) NUMBITS(1) [
            PoURequired = 0b0,
            PoUNotRequired = 0b1,

        ],
       IDC OFFSET(28) NUMBITS(1) [
            PoURequired = 0b0,
            PoUNotRequired = 0b1,

        ],
        CWG OFFSET(24) NUMBITS(4) [],
        ERG OFFSET(20) NUMBITS(4) [],
        DminLine OFFSET(16) NUMBITS(4) [],
        L1Ip OFFSET(14) NUMBITS(2) [
            VPIPT = 0b00,
            AIVIVT = 0b01,
            VIPT = 0b10,
            PIPT = 0b11,
        ],
        IminLine OFFSET(0) NUMBITS(4) [],
    ]
}

pub struct CTR_EL0_Reg;

impl Readable for CTR_EL0_Reg {
    type T = u64;
    type R = CTR_EL0::Register;

    sys_coproc_read_raw!(u64, "CTR_EL0", "x");
}

impl Writeable for CTR_EL0_Reg {
    type T = u64;
    type R = CTR_EL0::Register;

    sys_coproc_write_raw!(u64, "CTR_EL0", "x");
}
pub const CTR_EL0: CTR_EL0_Reg = CTR_EL0_Reg;

register_bitfields! {u64,
pub ID_AA64PFR1_EL1[
        MTE OFFSET(8) NUMBITS(4) [
            NotImplemented = 0b0000,
            InstOnlyMTE = 0b0001,
            FullMTE = 0b0010,
            MTEWithAsymTagCheckFaultHandling = 0b0011,
        ]
    ]
}

pub struct ID_AA64PFR1_EL1_Reg;

impl Readable for ID_AA64PFR1_EL1_Reg {
    type T = u64;
    type R = ID_AA64PFR1_EL1::Register;

    sys_coproc_read_raw!(u64, "ID_AA64PFR1_EL1", "x");
}

impl ID_AA64PFR1_EL1_Reg {
    pub fn is_mte2_supported(&self) -> bool {
        self.matches_all(ID_AA64PFR1_EL1::MTE::FullMTE)
    }
}

pub const ID_AA64PFR1_EL1: ID_AA64PFR1_EL1_Reg = ID_AA64PFR1_EL1_Reg;
