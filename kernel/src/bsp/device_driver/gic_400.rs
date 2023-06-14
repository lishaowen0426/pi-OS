//! PI4 GIC Interrupt controller

use crate::{
    bsp::mmio,
    errno::ErrorCode,
    exception,
    interrupt::IRQController,
    memory::{config, MMIOWrapper},
    println,
    utils::bitfields::Bitfields,
};
use tock_registers::{
    interfaces::{ReadWriteable, Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite, WriteOnly},
};

const IC_VIRTUAL_START: usize = config::VIRTUAL_PERIPHERAL_START + mmio::IC_OFFSET;

const ARM_LOCAL_OFFSET: usize = 0xFF80_0000 - config::PHYSICAL_PERIPHERAL_START;
const ARM_LOCAL_VIRTUAL_START: usize = config::VIRTUAL_PERIPHERAL_START + ARM_LOCAL_OFFSET;

const ARMC_OFFSET: usize = 0xFE00_B200 - config::PHYSICAL_PERIPHERAL_START;
const ARMC_VIRTUAL_START: usize = config::VIRTUAL_PERIPHERAL_START + ARMC_OFFSET;

register_bitfields!(u32,
    ARM_CONTROL[
        AXIERRIRQ_EN OFFSET(6) NUMBITS(1)[],
        PROC_CLK_TIMER OFFSET(7) NUMBITS(1)[],
        TIMER_INCREMENT OFFSET(8) NUMBITS(1)[],
    ],
    CORE_IRQ_CONTROL[
        AXI_ERR_CORE OFFSET(4) NUMBITS(6)[
            CORE0_IRQ = 0,
            CORE1_IRQ = 1,
            CORE2_IRQ = 2,
            CORE3_IRQ = 3,
            CORE0_FIQ = 4,
            CORE1_FIQ = 5,
            CORE2_FIQ = 6,
            CORE3_FIQ = 7,
        ]
    ],
    PMU_CONTROL_SET [
        PMU_IRQ OFFSET(0) NUMBITS(4)[],
        PMU_IFQ OFFSET(4) NUMBITS(4)[],
    ],
    PMU_CONTROL_CLR [
        PMU_IRQ OFFSET(0) NUMBITS(4)[],
        PMU_IFQ OFFSET(4) NUMBITS(4)[],
    ],
    PERI_IRQ_ROUTE0 [
        LOCAL_TIMER_IRQ OFFSET(0) NUMBITS(3)[
            CORE0_IRQ = 0,
            CORE1_IRQ = 1,
            CORE2_IRQ = 2,
            CORE3_IRQ = 3,
            CORE0_FIQ = 4,
            CORE1_FIQ = 5,
            CORE2_FIQ = 6,
            CORE3_FIQ = 7,
        ],
        WRITE_MASKS OFFSET(24) NUMBITS(8)[],
    ],
    AXI_QUITE_TIME[
        AXI_QUIET_TIME OFFSET(0) NUMBITS(20)[],
        AXI_QUIET_IRQ_ENB OFFSET(20) NUMBITS(1)[],
    ],
    LOCAL_TIMER_CONTROL[
        TIMER_TIMEOUT OFFSET(0) NUMBITS(28)[],
        TIMER_EN OFFSET(28) NUMBITS(1)[],
        TIMER_IRQ_EN OFFSET(28) NUMBITS(1)[],
        TIMER_IRQ_FLAG OFFSET(31) NUMBITS(1)[],
    ],
    LOCAL_TIMER_IRQ[
        RELOAD OFFSET(30) NUMBITS(1)[],
        IRQ_CLEAR OFFSET(31) NUMBITS(1)[],
    ],
    TIMER_CNTRL0[
        CNT_PS_IRQ OFFSET(0) NUMBITS(1)[],
        CNT_PNS_IRQ OFFSET(1) NUMBITS(1)[],
        CNT_HP_IRQ OFFSET(2) NUMBITS(1)[],
        CNT_V_IRQ OFFSET(3) NUMBITS(1)[],
        CNT_PS_IRQ_FIQ OFFSET(4) NUMBITS(1)[],
        CNT_PNS_IRQ_FIQ OFFSET(5) NUMBITS(1)[],
        CNT_HP_IRQ_FIQ OFFSET(6) NUMBITS(1)[],
        CNT_V_IRQ_FIQ OFFSET(7) NUMBITS(1)[],
    ],
    TIMER_CNTRL1[
        CNT_PS_IRQ OFFSET(0) NUMBITS(1)[],
        CNT_PNS_IRQ OFFSET(1) NUMBITS(1)[],
        CNT_HP_IRQ OFFSET(2) NUMBITS(1)[],
        CNT_V_IRQ OFFSET(3) NUMBITS(1)[],
        CNT_PS_IRQ_FIQ OFFSET(4) NUMBITS(1)[],
        CNT_PNS_IRQ_FIQ OFFSET(5) NUMBITS(1)[],
        CNT_HP_IRQ_FIQ OFFSET(6) NUMBITS(1)[],
        CNT_V_IRQ_FIQ OFFSET(7) NUMBITS(1)[],
    ],
    TIMER_CNTRL2[
        CNT_PS_IRQ OFFSET(0) NUMBITS(1)[],
        CNT_PNS_IRQ OFFSET(1) NUMBITS(1)[],
        CNT_HP_IRQ OFFSET(2) NUMBITS(1)[],
        CNT_V_IRQ OFFSET(3) NUMBITS(1)[],
        CNT_PS_IRQ_FIQ OFFSET(4) NUMBITS(1)[],
        CNT_PNS_IRQ_FIQ OFFSET(5) NUMBITS(1)[],
        CNT_HP_IRQ_FIQ OFFSET(6) NUMBITS(1)[],
        CNT_V_IRQ_FIQ OFFSET(7) NUMBITS(1)[],
    ],
    TIMER_CNTRL3[
        CNT_PS_IRQ OFFSET(0) NUMBITS(1)[],
        CNT_PNS_IRQ OFFSET(1) NUMBITS(1)[],
        CNT_HP_IRQ OFFSET(2) NUMBITS(1)[],
        CNT_V_IRQ OFFSET(3) NUMBITS(1)[],
        CNT_PS_IRQ_FIQ OFFSET(4) NUMBITS(1)[],
        CNT_PNS_IRQ_FIQ OFFSET(5) NUMBITS(1)[],
        CNT_HP_IRQ_FIQ OFFSET(6) NUMBITS(1)[],
        CNT_V_IRQ_FIQ OFFSET(7) NUMBITS(1)[],
    ],

    MAILBOX_CNTRL0[
        MBOX0_IRQ OFFSET(0) NUMBITS(1)[],
        MBOX1_IRQ OFFSET(1) NUMBITS(1)[],
        MBOX2_IRQ OFFSET(2) NUMBITS(1)[],
        MBOX3_IRQ OFFSET(3) NUMBITS(1)[],

        MBOX0_FIQ OFFSET(4) NUMBITS(1)[],
        MBOX1_FIQ OFFSET(5) NUMBITS(1)[],
        MBOX2_FIQ OFFSET(6) NUMBITS(1)[],
        MBOX3_FIQ OFFSET(7) NUMBITS(1)[],
    ],
    MAILBOX_CNTRL1[
        MBOX0_IRQ OFFSET(0) NUMBITS(1)[],
        MBOX1_IRQ OFFSET(1) NUMBITS(1)[],
        MBOX2_IRQ OFFSET(2) NUMBITS(1)[],
        MBOX3_IRQ OFFSET(3) NUMBITS(1)[],

        MBOX0_FIQ OFFSET(4) NUMBITS(1)[],
        MBOX1_FIQ OFFSET(5) NUMBITS(1)[],
        MBOX2_FIQ OFFSET(6) NUMBITS(1)[],
        MBOX3_FIQ OFFSET(7) NUMBITS(1)[],
    ],
    MAILBOX_CNTRL2[
        MBOX0_IRQ OFFSET(0) NUMBITS(1)[],
        MBOX1_IRQ OFFSET(1) NUMBITS(1)[],
        MBOX2_IRQ OFFSET(2) NUMBITS(1)[],
        MBOX3_IRQ OFFSET(3) NUMBITS(1)[],

        MBOX0_FIQ OFFSET(4) NUMBITS(1)[],
        MBOX1_FIQ OFFSET(5) NUMBITS(1)[],
        MBOX2_FIQ OFFSET(6) NUMBITS(1)[],
        MBOX3_FIQ OFFSET(7) NUMBITS(1)[],
    ],
    MAILBOX_CNTRL3[
        MBOX0_IRQ OFFSET(0) NUMBITS(1)[],
        MBOX1_IRQ OFFSET(1) NUMBITS(1)[],
        MBOX2_IRQ OFFSET(2) NUMBITS(1)[],
        MBOX3_IRQ OFFSET(3) NUMBITS(1)[],

        MBOX0_FIQ OFFSET(4) NUMBITS(1)[],
        MBOX1_FIQ OFFSET(5) NUMBITS(1)[],
        MBOX2_FIQ OFFSET(6) NUMBITS(1)[],
        MBOX3_FIQ OFFSET(7) NUMBITS(1)[],
    ],
    IRQ_SOURCE0[
        CNT_PS_IRQ OFFSET(0) NUMBITS(1)[],
        CNT_PNS_IRQ OFFSET(1) NUMBITS(1)[],
        CNT_HP_IRQ OFFSET(2) NUMBITS(1)[],
        CNT_V_IRQ OFFSET(3) NUMBITS(1)[],
        MAILBOX_IRQ OFFSET(4) NUMBITS(4)[],
        CORE_IRQ OFFSET(8) NUMBITS(1)[],
        PMU_IRQ OFFSET(9) NUMBITS(1)[],
        AXI_QUITE OFFSET(10) NUMBITS(1)[],
        TIMER_IRQ OFFSET(11) NUMBITS(1)[],
        AXI_IRQ OFFSET(30) NUMBITS(1)[],
    ],
    IRQ_SOURCE1[
        CNT_PS_IRQ OFFSET(0) NUMBITS(1)[],
        CNT_PNS_IRQ OFFSET(1) NUMBITS(1)[],
        CNT_HP_IRQ OFFSET(2) NUMBITS(1)[],
        CNT_V_IRQ OFFSET(3) NUMBITS(1)[],
        MAILBOX_IRQ OFFSET(4) NUMBITS(4)[],
        CORE_IRQ OFFSET(8) NUMBITS(1)[],
        PMU_IRQ OFFSET(9) NUMBITS(1)[],
        AXI_QUITE OFFSET(10) NUMBITS(1)[],
        TIMER_IRQ OFFSET(11) NUMBITS(1)[],
        AXI_IRQ OFFSET(30) NUMBITS(1)[],
    ],
    IRQ_SOURCE2[
        CNT_PS_IRQ OFFSET(0) NUMBITS(1)[],
        CNT_PNS_IRQ OFFSET(1) NUMBITS(1)[],
        CNT_HP_IRQ OFFSET(2) NUMBITS(1)[],
        CNT_V_IRQ OFFSET(3) NUMBITS(1)[],
        MAILBOX_IRQ OFFSET(4) NUMBITS(4)[],
        CORE_IRQ OFFSET(8) NUMBITS(1)[],
        PMU_IRQ OFFSET(9) NUMBITS(1)[],
        AXI_QUITE OFFSET(10) NUMBITS(1)[],
        TIMER_IRQ OFFSET(11) NUMBITS(1)[],
        AXI_IRQ OFFSET(30) NUMBITS(1)[],
    ],
    IRQ_SOURCE3[
        CNT_PS_IRQ OFFSET(0) NUMBITS(1)[],
        CNT_PNS_IRQ OFFSET(1) NUMBITS(1)[],
        CNT_HP_IRQ OFFSET(2) NUMBITS(1)[],
        CNT_V_IRQ OFFSET(3) NUMBITS(1)[],
        MAILBOX_IRQ OFFSET(4) NUMBITS(4)[],
        CORE_IRQ OFFSET(8) NUMBITS(1)[],
        PMU_IRQ OFFSET(9) NUMBITS(1)[],
        AXI_QUITE OFFSET(10) NUMBITS(1)[],
        TIMER_IRQ OFFSET(11) NUMBITS(1)[],
        AXI_IRQ OFFSET(30) NUMBITS(1)[],
    ],
    FIQ_SOURCE0[
        CNT_PS_FIQ OFFSET(0) NUMBITS(1)[],
        CNT_PNS_FIQ OFFSET(1) NUMBITS(1)[],
        CNT_HP_FIQ OFFSET(2) NUMBITS(1)[],
        CNT_V_FIQ OFFSET(3) NUMBITS(1)[],
        MAILBOX_FIQ OFFSET(4) NUMBITS(4)[],
        CORE_FIQ OFFSET(8) NUMBITS(1)[],
        PMU_FIQ OFFSET(9) NUMBITS(1)[],
        LOCAL_TIMER_FIQ OFFSET(11) NUMBITS(1)[],
        AXI_IRQ OFFSET(30) NUMBITS(1)[],
    ],
    FIQ_SOURCE1[
        CNT_PS_FIQ OFFSET(0) NUMBITS(1)[],
        CNT_PNS_FIQ OFFSET(1) NUMBITS(1)[],
        CNT_HP_FIQ OFFSET(2) NUMBITS(1)[],
        CNT_V_FIQ OFFSET(3) NUMBITS(1)[],
        MAILBOX_FIQ OFFSET(4) NUMBITS(4)[],
        CORE_FIQ OFFSET(8) NUMBITS(1)[],
        PMU_FIQ OFFSET(9) NUMBITS(1)[],
        LOCAL_TIMER_FIQ OFFSET(11) NUMBITS(1)[],
        AXI_IRQ OFFSET(30) NUMBITS(1)[],
    ],
    FIQ_SOURCE2[
        CNT_PS_FIQ OFFSET(0) NUMBITS(1)[],
        CNT_PNS_FIQ OFFSET(1) NUMBITS(1)[],
        CNT_HP_FIQ OFFSET(2) NUMBITS(1)[],
        CNT_V_FIQ OFFSET(3) NUMBITS(1)[],
        MAILBOX_FIQ OFFSET(4) NUMBITS(4)[],
        CORE_FIQ OFFSET(8) NUMBITS(1)[],
        PMU_FIQ OFFSET(9) NUMBITS(1)[],
        LOCAL_TIMER_FIQ OFFSET(11) NUMBITS(1)[],
        AXI_IRQ OFFSET(30) NUMBITS(1)[],
    ],
    FIQ_SOURCE3[
        CNT_PS_FIQ OFFSET(0) NUMBITS(1)[],
        CNT_PNS_FIQ OFFSET(1) NUMBITS(1)[],
        CNT_HP_FIQ OFFSET(2) NUMBITS(1)[],
        CNT_V_FIQ OFFSET(3) NUMBITS(1)[],
        MAILBOX_FIQ OFFSET(4) NUMBITS(4)[],
        CORE_FIQ OFFSET(8) NUMBITS(1)[],
        PMU_FIQ OFFSET(9) NUMBITS(1)[],
        LOCAL_TIMER_FIQ OFFSET(11) NUMBITS(1)[],
        AXI_IRQ OFFSET(30) NUMBITS(1)[],
    ],

    IRQ0_PENDING0[
        VC_IRQ_31_0 OFFSET(0) NUMBITS(32)[],
    ],
    IRQ1_PENDING0[
        VC_IRQ_31_0 OFFSET(0) NUMBITS(32)[],
    ],
    IRQ2_PENDING0[
        VC_IRQ_31_0 OFFSET(0) NUMBITS(32)[],
    ],
    IRQ3_PENDING0[
        VC_IRQ_31_0 OFFSET(0) NUMBITS(32)[],
    ],
    IRQ0_PENDING1[
        VC_IRQ_63_32 OFFSET(0) NUMBITS(32)[],
    ],
    IRQ1_PENDING1[
        VC_IRQ_63_32 OFFSET(0) NUMBITS(32)[],
    ],
    IRQ2_PENDING1[
        VC_IRQ_63_32 OFFSET(0) NUMBITS(32)[],
    ],
    IRQ3_PENDING1[
        VC_IRQ_63_32 OFFSET(0) NUMBITS(32)[],
    ],

    IRQ0_PENDING2[
        TIMER_IRQ OFFSET(0) NUMBITS(1)[],
        MAILBOX_IRQ OFFSET(1) NUMBITS(1)[],
        BELL_IRQ0 OFFSET(2) NUMBITS(1)[],
        BELL_IRQ1 OFFSET(3) NUMBITS(1)[],
        VPU_C0_C1_HALT OFFSET(4) NUMBITS(1)[],
        VPU_C1_HALT OFFSET(5) NUMBITS(1)[],
        ARM_ADDR_ERROR OFFSET(6) NUMBITS(1)[],
        ARM_AXI_ERROR OFFSET(7) NUMBITS(1)[],
        SW_TRIG_INT OFFSET(8) NUMBITS(8) [],
        INT31_0 OFFSET(24) NUMBITS(1) [],
        INT63_32 OFFSET(25) NUMBITS(1) [],
        IRQ OFFSET(31) NUMBITS(1) [],
    ],
    IRQ1_PENDING2[
        TIMER_IRQ OFFSET(0) NUMBITS(1)[],
        MAILBOX_IRQ OFFSET(1) NUMBITS(1)[],
        BELL_IRQ0 OFFSET(2) NUMBITS(1)[],
        BELL_IRQ1 OFFSET(3) NUMBITS(1)[],
        VPU_C0_C1_HALT OFFSET(4) NUMBITS(1)[],
        VPU_C1_HALT OFFSET(5) NUMBITS(1)[],
        ARM_ADDR_ERROR OFFSET(6) NUMBITS(1)[],
        ARM_AXI_ERROR OFFSET(7) NUMBITS(1)[],
        SW_TRIG_INT OFFSET(8) NUMBITS(8) [],
        INT31_0 OFFSET(24) NUMBITS(1) [],
        INT63_32 OFFSET(25) NUMBITS(1) [],
        IRQ OFFSET(31) NUMBITS(1) [],
    ],
    IRQ2_PENDING2[
        TIMER_IRQ OFFSET(0) NUMBITS(1)[],
        MAILBOX_IRQ OFFSET(1) NUMBITS(1)[],
        BELL_IRQ0 OFFSET(2) NUMBITS(1)[],
        BELL_IRQ1 OFFSET(3) NUMBITS(1)[],
        VPU_C0_C1_HALT OFFSET(4) NUMBITS(1)[],
        VPU_C1_HALT OFFSET(5) NUMBITS(1)[],
        ARM_ADDR_ERROR OFFSET(6) NUMBITS(1)[],
        ARM_AXI_ERROR OFFSET(7) NUMBITS(1)[],
        SW_TRIG_INT OFFSET(8) NUMBITS(8) [],
        INT31_0 OFFSET(24) NUMBITS(1) [],
        INT63_32 OFFSET(25) NUMBITS(1) [],
        IRQ OFFSET(31) NUMBITS(1) [],
    ],
    IRQ3_PENDING2[
        TIMER_IRQ OFFSET(0) NUMBITS(1)[],
        MAILBOX_IRQ OFFSET(1) NUMBITS(1)[],
        BELL_IRQ0 OFFSET(2) NUMBITS(1)[],
        BELL_IRQ1 OFFSET(3) NUMBITS(1)[],
        VPU_C0_C1_HALT OFFSET(4) NUMBITS(1)[],
        VPU_C1_HALT OFFSET(5) NUMBITS(1)[],
        ARM_ADDR_ERROR OFFSET(6) NUMBITS(1)[],
        ARM_AXI_ERROR OFFSET(7) NUMBITS(1)[],
        SW_TRIG_INT OFFSET(8) NUMBITS(8) [],
        INT31_0 OFFSET(24) NUMBITS(1) [],
        INT63_32 OFFSET(25) NUMBITS(1) [],
        IRQ OFFSET(31) NUMBITS(1) [],
    ],

    IRQ0_SET_EN_0[
        VC_IRQ_31_0 OFFSET(0) NUMBITS(32)[],
    ],
    IRQ1_SET_EN_0[
        VC_IRQ_31_0 OFFSET(0) NUMBITS(32)[],
    ],
    IRQ2_SET_EN_0[
        VC_IRQ_31_0 OFFSET(0) NUMBITS(32)[],
    ],
    IRQ3_SET_EN_0[
        VC_IRQ_31_0 OFFSET(0) NUMBITS(32)[],
    ],

    IRQ0_SET_EN_1[
        VC_IRQ_63_32 OFFSET(0) NUMBITS(32)[],
    ],
    IRQ1_SET_EN_1[
        VC_IRQ_63_32 OFFSET(0) NUMBITS(32)[],
    ],
    IRQ2_SET_EN_1[
        VC_IRQ_63_32 OFFSET(0) NUMBITS(32)[],
    ],
    IRQ3_SET_EN_1[
        VC_IRQ_63_32 OFFSET(0) NUMBITS(32)[],
    ],

    IRQ0_SET_EN_2[
        TIMER_IRQ OFFSET(0) NUMBITS(1)[],
        MAILBOX_IRQ0 OFFSET(1) NUMBITS(1)[],
        BELL_IRQ0 OFFSET(2) NUMBITS(1)[],
        BELL_IRQ1 OFFSET(3) NUMBITS(1)[],
        VPU_C0_C1_HALT OFFSET(4) NUMBITS(1)[],
        VPU_C1_HALT OFFSET(5) NUMBITS(1)[],
        ARM_ADDR_ERROR OFFSET(6) NUMBITS(1)[],
        ARM_AXI_ERROR OFFSET(7) NUMBITS(1)[],
        SW_TRIG_INT OFFSET(8) NUMBITS(8) [],
        IRQ OFFSET(31) NUMBITS(1) [],
    ],
    IRQ1_SET_EN_2[
        TIMER_IRQ OFFSET(0) NUMBITS(1)[],
        MAILBOX_IRQ0 OFFSET(1) NUMBITS(1)[],
        BELL_IRQ0 OFFSET(2) NUMBITS(1)[],
        BELL_IRQ1 OFFSET(3) NUMBITS(1)[],
        VPU_C0_C1_HALT OFFSET(4) NUMBITS(1)[],
        VPU_C1_HALT OFFSET(5) NUMBITS(1)[],
        ARM_ADDR_ERROR OFFSET(6) NUMBITS(1)[],
        ARM_AXI_ERROR OFFSET(7) NUMBITS(1)[],
        SW_TRIG_INT OFFSET(8) NUMBITS(8) [],
        IRQ OFFSET(31) NUMBITS(1) [],
    ],
    IRQ2_SET_EN_2[
        TIMER_IRQ OFFSET(0) NUMBITS(1)[],
        MAILBOX_IRQ0 OFFSET(1) NUMBITS(1)[],
        BELL_IRQ0 OFFSET(2) NUMBITS(1)[],
        BELL_IRQ1 OFFSET(3) NUMBITS(1)[],
        VPU_C0_C1_HALT OFFSET(4) NUMBITS(1)[],
        VPU_C1_HALT OFFSET(5) NUMBITS(1)[],
        ARM_ADDR_ERROR OFFSET(6) NUMBITS(1)[],
        ARM_AXI_ERROR OFFSET(7) NUMBITS(1)[],
        SW_TRIG_INT OFFSET(8) NUMBITS(8) [],
        IRQ OFFSET(31) NUMBITS(1) [],
    ],
    IRQ3_SET_EN_2[
        TIMER_IRQ OFFSET(0) NUMBITS(1)[],
        MAILBOX_IRQ0 OFFSET(1) NUMBITS(1)[],
        BELL_IRQ0 OFFSET(2) NUMBITS(1)[],
        BELL_IRQ1 OFFSET(3) NUMBITS(1)[],
        VPU_C0_C1_HALT OFFSET(4) NUMBITS(1)[],
        VPU_C1_HALT OFFSET(5) NUMBITS(1)[],
        ARM_ADDR_ERROR OFFSET(6) NUMBITS(1)[],
        ARM_AXI_ERROR OFFSET(7) NUMBITS(1)[],
        SW_TRIG_INT OFFSET(8) NUMBITS(8) [],
        IRQ OFFSET(31) NUMBITS(1) [],
    ],


    IRQ0_CLR_EN_0[
        VC_IRQ_31_0 OFFSET(0) NUMBITS(32)[],
    ],
    IRQ1_CLR_EN_0[
        VC_IRQ_31_0 OFFSET(0) NUMBITS(32)[],
    ],
    IRQ2_CLR_EN_0[
        VC_IRQ_31_0 OFFSET(0) NUMBITS(32)[],
    ],
    IRQ3_CLR_EN_0[
        VC_IRQ_31_0 OFFSET(0) NUMBITS(32)[],
    ],

    IRQ0_CLR_EN_1[
        VC_IRQ_63_32 OFFSET(0) NUMBITS(32)[],
    ],
    IRQ1_CLR_EN_1[
        VC_IRQ_63_32 OFFSET(0) NUMBITS(32)[],
    ],
    IRQ2_CLR_EN_1[
        VC_IRQ_63_32 OFFSET(0) NUMBITS(32)[],
    ],
    IRQ3_CLR_EN_1[
        VC_IRQ_63_32 OFFSET(0) NUMBITS(32)[],
    ],

    IRQ0_CLR_EN_2[
        TIMER_IRQ OFFSET(0) NUMBITS(1)[],
        MAILBOX_IRQ0 OFFSET(1) NUMBITS(1)[],
        BELL_IRQ0 OFFSET(2) NUMBITS(1)[],
        BELL_IRQ1 OFFSET(3) NUMBITS(1)[],
        VPU_C0_C1_HALT OFFSET(4) NUMBITS(1)[],
        VPU_C1_HALT OFFSET(5) NUMBITS(1)[],
        ARM_ADDR_ERROR OFFSET(6) NUMBITS(1)[],
        ARM_AXI_ERROR OFFSET(7) NUMBITS(1)[],
        SW_TRIG_INT OFFSET(8) NUMBITS(8) [],
        IRQ OFFSET(31) NUMBITS(1) [],
    ],
    IRQ1_CLR_EN_2[
        TIMER_IRQ OFFSET(0) NUMBITS(1)[],
        MAILBOX_IRQ0 OFFSET(1) NUMBITS(1)[],
        BELL_IRQ0 OFFSET(2) NUMBITS(1)[],
        BELL_IRQ1 OFFSET(3) NUMBITS(1)[],
        VPU_C0_C1_HALT OFFSET(4) NUMBITS(1)[],
        VPU_C1_HALT OFFSET(5) NUMBITS(1)[],
        ARM_ADDR_ERROR OFFSET(6) NUMBITS(1)[],
        ARM_AXI_ERROR OFFSET(7) NUMBITS(1)[],
        SW_TRIG_INT OFFSET(8) NUMBITS(8) [],
        IRQ OFFSET(31) NUMBITS(1) [],
    ],
    IRQ2_CLR_EN_2[
        TIMER_IRQ OFFSET(0) NUMBITS(1)[],
        MAILBOX_IRQ0 OFFSET(1) NUMBITS(1)[],
        BELL_IRQ0 OFFSET(2) NUMBITS(1)[],
        BELL_IRQ1 OFFSET(3) NUMBITS(1)[],
        VPU_C0_C1_HALT OFFSET(4) NUMBITS(1)[],
        VPU_C1_HALT OFFSET(5) NUMBITS(1)[],
        ARM_ADDR_ERROR OFFSET(6) NUMBITS(1)[],
        ARM_AXI_ERROR OFFSET(7) NUMBITS(1)[],
        SW_TRIG_INT OFFSET(8) NUMBITS(8) [],
        IRQ OFFSET(31) NUMBITS(1) [],
    ],
    IRQ3_CLR_EN_2[
        TIMER_IRQ OFFSET(0) NUMBITS(1)[],
        MAILBOX_IRQ0 OFFSET(1) NUMBITS(1)[],
        BELL_IRQ0 OFFSET(2) NUMBITS(1)[],
        BELL_IRQ1 OFFSET(3) NUMBITS(1)[],
        VPU_C0_C1_HALT OFFSET(4) NUMBITS(1)[],
        VPU_C1_HALT OFFSET(5) NUMBITS(1)[],
        ARM_ADDR_ERROR OFFSET(6) NUMBITS(1)[],
        ARM_AXI_ERROR OFFSET(7) NUMBITS(1)[],
        SW_TRIG_INT OFFSET(8) NUMBITS(8) [],
        IRQ OFFSET(31) NUMBITS(1) [],
    ],

    IRQ_STATUS0[
        VC_IRQ_31_0 OFFSET(0) NUMBITS(32)[],
    ],
    IRQ_STATUS1[
        VC_IRQ_31_0 OFFSET(0) NUMBITS(32)[],
    ],
    IRQ_STATUS2[
        TIMER_IRQ OFFSET(0) NUMBITS(1)[],
        MAILBOX_IRQ0 OFFSET(1) NUMBITS(1)[],
        BELL_IRQ0 OFFSET(2) NUMBITS(1)[],
        BELL_IRQ1 OFFSET(3) NUMBITS(1)[],
        VPU_C0_C1_HALT OFFSET(4) NUMBITS(1)[],
        VPU_C1_HALT OFFSET(5) NUMBITS(1)[],
        ARM_ADDR_ERROR OFFSET(6) NUMBITS(1)[],
        ARM_AXI_ERROR OFFSET(7) NUMBITS(1)[],
        SW_TRIG_INT OFFSET(8) NUMBITS(8) [],
        IRQ OFFSET(31) NUMBITS(1) [],
    ],



    FIQ0_PENDING0[
        VC_IRQ_31_0 OFFSET(0) NUMBITS(32)[],
    ],
    FIQ1_PENDING0[
        VC_IRQ_31_0 OFFSET(0) NUMBITS(32)[],
    ],
    FIQ2_PENDING0[
        VC_IRQ_31_0 OFFSET(0) NUMBITS(32)[],
    ],
    FIQ3_PENDING0[
        VC_IRQ_31_0 OFFSET(0) NUMBITS(32)[],
    ],
    FIQ0_PENDING1[
        VC_IRQ_63_32 OFFSET(0) NUMBITS(32)[],
    ],
    FIQ1_PENDING1[
        VC_IRQ_63_32 OFFSET(0) NUMBITS(32)[],
    ],
    FIQ2_PENDING1[
        VC_IRQ_63_32 OFFSET(0) NUMBITS(32)[],
    ],
    FIQ3_PENDING1[
        VC_IRQ_63_32 OFFSET(0) NUMBITS(32)[],
    ],

    FIQ0_PENDING2[
        TIMER_IRQ OFFSET(0) NUMBITS(1)[],
        MAILBOX_IRQ OFFSET(1) NUMBITS(1)[],
        BELL_IRQ0 OFFSET(2) NUMBITS(1)[],
        BELL_IRQ1 OFFSET(3) NUMBITS(1)[],
        VPU_C0_C1_HALT OFFSET(4) NUMBITS(1)[],
        VPU_C1_HALT OFFSET(5) NUMBITS(1)[],
        ARM_ADDR_ERROR OFFSET(6) NUMBITS(1)[],
        ARM_AXI_ERROR OFFSET(7) NUMBITS(1)[],
        SW_TRIG_INT OFFSET(8) NUMBITS(8) [],
        INT31_0 OFFSET(24) NUMBITS(1) [],
        INT63_32 OFFSET(25) NUMBITS(1) [],
        IRQ OFFSET(31) NUMBITS(1) [],
    ],
    FIQ1_PENDING2[
        TIMER_IRQ OFFSET(0) NUMBITS(1)[],
        MAILBOX_IRQ OFFSET(1) NUMBITS(1)[],
        BELL_IRQ0 OFFSET(2) NUMBITS(1)[],
        BELL_IRQ1 OFFSET(3) NUMBITS(1)[],
        VPU_C0_C1_HALT OFFSET(4) NUMBITS(1)[],
        VPU_C1_HALT OFFSET(5) NUMBITS(1)[],
        ARM_ADDR_ERROR OFFSET(6) NUMBITS(1)[],
        ARM_AXI_ERROR OFFSET(7) NUMBITS(1)[],
        SW_TRIG_INT OFFSET(8) NUMBITS(8) [],
        INT31_0 OFFSET(24) NUMBITS(1) [],
        INT63_32 OFFSET(25) NUMBITS(1) [],
        IRQ OFFSET(31) NUMBITS(1) [],
    ],
    FIQ2_PENDING2[
        TIMER_IRQ OFFSET(0) NUMBITS(1)[],
        MAILBOX_IRQ OFFSET(1) NUMBITS(1)[],
        BELL_IRQ0 OFFSET(2) NUMBITS(1)[],
        BELL_IRQ1 OFFSET(3) NUMBITS(1)[],
        VPU_C0_C1_HALT OFFSET(4) NUMBITS(1)[],
        VPU_C1_HALT OFFSET(5) NUMBITS(1)[],
        ARM_ADDR_ERROR OFFSET(6) NUMBITS(1)[],
        ARM_AXI_ERROR OFFSET(7) NUMBITS(1)[],
        SW_TRIG_INT OFFSET(8) NUMBITS(8) [],
        INT31_0 OFFSET(24) NUMBITS(1) [],
        INT63_32 OFFSET(25) NUMBITS(1) [],
        IRQ OFFSET(31) NUMBITS(1) [],
    ],
    FIQ3_PENDING2[
        TIMER_IRQ OFFSET(0) NUMBITS(1)[],
        MAILBOX_IRQ OFFSET(1) NUMBITS(1)[],
        BELL_IRQ0 OFFSET(2) NUMBITS(1)[],
        BELL_IRQ1 OFFSET(3) NUMBITS(1)[],
        VPU_C0_C1_HALT OFFSET(4) NUMBITS(1)[],
        VPU_C1_HALT OFFSET(5) NUMBITS(1)[],
        ARM_ADDR_ERROR OFFSET(6) NUMBITS(1)[],
        ARM_AXI_ERROR OFFSET(7) NUMBITS(1)[],
        SW_TRIG_INT OFFSET(8) NUMBITS(8) [],
        INT31_0 OFFSET(24) NUMBITS(1) [],
        INT63_32 OFFSET(25) NUMBITS(1) [],
        IRQ OFFSET(31) NUMBITS(1) [],
    ],

    FIQ0_SET_EN_0[
        VC_IRQ_31_0 OFFSET(0) NUMBITS(32)[],
    ],
    FIQ1_SET_EN_0[
        VC_IRQ_31_0 OFFSET(0) NUMBITS(32)[],
    ],
    FIQ2_SET_EN_0[
        VC_IRQ_31_0 OFFSET(0) NUMBITS(32)[],
    ],
    FIQ3_SET_EN_0[
        VC_IRQ_31_0 OFFSET(0) NUMBITS(32)[],
    ],

    FIQ0_SET_EN_1[
        VC_IRQ_63_32 OFFSET(0) NUMBITS(32)[],
    ],
    FIQ1_SET_EN_1[
        VC_IRQ_63_32 OFFSET(0) NUMBITS(32)[],
    ],
    FIQ2_SET_EN_1[
        VC_IRQ_63_32 OFFSET(0) NUMBITS(32)[],
    ],
    FIQ3_SET_EN_1[
        VC_IRQ_63_32 OFFSET(0) NUMBITS(32)[],
    ],

    FIQ0_SET_EN_2[
        TIMER_IRQ OFFSET(0) NUMBITS(1)[],
        MAILBOX_IRQ0 OFFSET(1) NUMBITS(1)[],
        BELL_IRQ0 OFFSET(2) NUMBITS(1)[],
        BELL_IRQ1 OFFSET(3) NUMBITS(1)[],
        VPU_C0_C1_HALT OFFSET(4) NUMBITS(1)[],
        VPU_C1_HALT OFFSET(5) NUMBITS(1)[],
        ARM_ADDR_ERROR OFFSET(6) NUMBITS(1)[],
        ARM_AXI_ERROR OFFSET(7) NUMBITS(1)[],
        SW_TRIG_INT OFFSET(8) NUMBITS(8) [],
        IRQ OFFSET(31) NUMBITS(1) [],
    ],
    FIQ1_SET_EN_2[
        TIMER_IRQ OFFSET(0) NUMBITS(1)[],
        MAILBOX_IRQ0 OFFSET(1) NUMBITS(1)[],
        BELL_IRQ0 OFFSET(2) NUMBITS(1)[],
        BELL_IRQ1 OFFSET(3) NUMBITS(1)[],
        VPU_C0_C1_HALT OFFSET(4) NUMBITS(1)[],
        VPU_C1_HALT OFFSET(5) NUMBITS(1)[],
        ARM_ADDR_ERROR OFFSET(6) NUMBITS(1)[],
        ARM_AXI_ERROR OFFSET(7) NUMBITS(1)[],
        SW_TRIG_INT OFFSET(8) NUMBITS(8) [],
        IRQ OFFSET(31) NUMBITS(1) [],
    ],
    FIQ2_SET_EN_2[
        TIMER_IRQ OFFSET(0) NUMBITS(1)[],
        MAILBOX_IRQ0 OFFSET(1) NUMBITS(1)[],
        BELL_IRQ0 OFFSET(2) NUMBITS(1)[],
        BELL_IRQ1 OFFSET(3) NUMBITS(1)[],
        VPU_C0_C1_HALT OFFSET(4) NUMBITS(1)[],
        VPU_C1_HALT OFFSET(5) NUMBITS(1)[],
        ARM_ADDR_ERROR OFFSET(6) NUMBITS(1)[],
        ARM_AXI_ERROR OFFSET(7) NUMBITS(1)[],
        SW_TRIG_INT OFFSET(8) NUMBITS(8) [],
        IRQ OFFSET(31) NUMBITS(1) [],
    ],
    FIQ3_SET_EN_2[
        TIMER_IRQ OFFSET(0) NUMBITS(1)[],
        MAILBOX_IRQ0 OFFSET(1) NUMBITS(1)[],
        BELL_IRQ0 OFFSET(2) NUMBITS(1)[],
        BELL_IRQ1 OFFSET(3) NUMBITS(1)[],
        VPU_C0_C1_HALT OFFSET(4) NUMBITS(1)[],
        VPU_C1_HALT OFFSET(5) NUMBITS(1)[],
        ARM_ADDR_ERROR OFFSET(6) NUMBITS(1)[],
        ARM_AXI_ERROR OFFSET(7) NUMBITS(1)[],
        SW_TRIG_INT OFFSET(8) NUMBITS(8) [],
        IRQ OFFSET(31) NUMBITS(1) [],
    ],


    FIQ0_CLR_EN_0[
        VC_IRQ_31_0 OFFSET(0) NUMBITS(32)[],
    ],
    FIQ1_CLR_EN_0[
        VC_IRQ_31_0 OFFSET(0) NUMBITS(32)[],
    ],
    FIQ2_CLR_EN_0[
        VC_IRQ_31_0 OFFSET(0) NUMBITS(32)[],
    ],
    FIQ3_CLR_EN_0[
        VC_IRQ_31_0 OFFSET(0) NUMBITS(32)[],
    ],

    FIQ0_CLR_EN_1[
        VC_IRQ_63_32 OFFSET(0) NUMBITS(32)[],
    ],
    FIQ1_CLR_EN_1[
        VC_IRQ_63_32 OFFSET(0) NUMBITS(32)[],
    ],
    FIQ2_CLR_EN_1[
        VC_IRQ_63_32 OFFSET(0) NUMBITS(32)[],
    ],
    FIQ3_CLR_EN_1[
        VC_IRQ_63_32 OFFSET(0) NUMBITS(32)[],
    ],

    FIQ0_CLR_EN_2[
        TIMER_IRQ OFFSET(0) NUMBITS(1)[],
        MAILBOX_IRQ0 OFFSET(1) NUMBITS(1)[],
        BELL_IRQ0 OFFSET(2) NUMBITS(1)[],
        BELL_IRQ1 OFFSET(3) NUMBITS(1)[],
        VPU_C0_C1_HALT OFFSET(4) NUMBITS(1)[],
        VPU_C1_HALT OFFSET(5) NUMBITS(1)[],
        ARM_ADDR_ERROR OFFSET(6) NUMBITS(1)[],
        ARM_AXI_ERROR OFFSET(7) NUMBITS(1)[],
        SW_TRIG_INT OFFSET(8) NUMBITS(8) [],
        IRQ OFFSET(31) NUMBITS(1) [],
    ],
    FIQ1_CLR_EN_2[
        TIMER_IRQ OFFSET(0) NUMBITS(1)[],
        MAILBOX_IRQ0 OFFSET(1) NUMBITS(1)[],
        BELL_IRQ0 OFFSET(2) NUMBITS(1)[],
        BELL_IRQ1 OFFSET(3) NUMBITS(1)[],
        VPU_C0_C1_HALT OFFSET(4) NUMBITS(1)[],
        VPU_C1_HALT OFFSET(5) NUMBITS(1)[],
        ARM_ADDR_ERROR OFFSET(6) NUMBITS(1)[],
        ARM_AXI_ERROR OFFSET(7) NUMBITS(1)[],
        SW_TRIG_INT OFFSET(8) NUMBITS(8) [],
        IRQ OFFSET(31) NUMBITS(1) [],
    ],
    FIQ2_CLR_EN_2[
        TIMER_IRQ OFFSET(0) NUMBITS(1)[],
        MAILBOX_IRQ0 OFFSET(1) NUMBITS(1)[],
        BELL_IRQ0 OFFSET(2) NUMBITS(1)[],
        BELL_IRQ1 OFFSET(3) NUMBITS(1)[],
        VPU_C0_C1_HALT OFFSET(4) NUMBITS(1)[],
        VPU_C1_HALT OFFSET(5) NUMBITS(1)[],
        ARM_ADDR_ERROR OFFSET(6) NUMBITS(1)[],
        ARM_AXI_ERROR OFFSET(7) NUMBITS(1)[],
        SW_TRIG_INT OFFSET(8) NUMBITS(8) [],
        IRQ OFFSET(31) NUMBITS(1) [],
    ],
    FIQ3_CLR_EN_2[
        TIMER_IRQ OFFSET(0) NUMBITS(1)[],
        MAILBOX_IRQ0 OFFSET(1) NUMBITS(1)[],
        BELL_IRQ0 OFFSET(2) NUMBITS(1)[],
        BELL_IRQ1 OFFSET(3) NUMBITS(1)[],
        VPU_C0_C1_HALT OFFSET(4) NUMBITS(1)[],
        VPU_C1_HALT OFFSET(5) NUMBITS(1)[],
        ARM_ADDR_ERROR OFFSET(6) NUMBITS(1)[],
        ARM_AXI_ERROR OFFSET(7) NUMBITS(1)[],
        SW_TRIG_INT OFFSET(8) NUMBITS(8) [],
        IRQ OFFSET(31) NUMBITS(1) [],
    ],

    SWIRQ_SET[
        SW_INT OFFSET(0) NUMBITS(8) [],
    ],
    SWIRQ_CLEAR[
        SW_INT OFFSET(0) NUMBITS(8) [],
    ],
);

register_structs!(
    ARMLocal{
        (0x00 => ARMControl: ReadWrite<u32, ARM_CONTROL::Register>),
        (0x04 => _reserved1),
        (0x0C => CoreIRQControl: ReadWrite<u32, CORE_IRQ_CONTROL::Register>),
        (0x10 => PMUControlSet: ReadWrite<u32, PMU_CONTROL_SET::Register>),
        (0x14 => PMUControlClr: ReadWrite<u32, PMU_CONTROL_CLR::Register>),
        (0x18 => _reserved2),
        (0x24 => PERIIRQRoute0:ReadWrite<u32, PERI_IRQ_ROUTE0::Register>),
        (0x28 => _reserved3),
        (0x30 => AXIQuiteTime: ReadWrite<u32, AXI_QUITE_TIME::Register>),
        (0x34 => LocalTimerControl: ReadWrite<u32, LOCAL_TIMER_CONTROL::Register>),
        (0x38 => LocalTimerIRQ: ReadWrite<u32, LOCAL_TIMER_IRQ::Register>),
        (0x3C => _reserved4),
        (0x40 => TimerCntrl0: ReadWrite<u32, TIMER_CNTRL0::Register>),
        (0x44 => TimerCntrl1: ReadWrite<u32, TIMER_CNTRL0::Register>),
        (0x48 => TimerCntrl2: ReadWrite<u32, TIMER_CNTRL0::Register>),
        (0x4C => TimerCntrl3: ReadWrite<u32, TIMER_CNTRL0::Register>),
        (0x50 => MailboxCntrl0: ReadWrite<u32, MAILBOX_CNTRL0::Register>),
        (0x54 => MailboxCntrl1: ReadWrite<u32, MAILBOX_CNTRL0::Register>),
        (0x58 => MailboxCntrl2: ReadWrite<u32, MAILBOX_CNTRL0::Register>),
        (0x5C => MailboxCntrl3: ReadWrite<u32, MAILBOX_CNTRL0::Register>),
        (0x60 => IRQSource0: ReadWrite<u32, IRQ_SOURCE0::Register>),
        (0x64 => IRQSource1: ReadWrite<u32, IRQ_SOURCE0::Register>),
        (0x68 => IRQSource2: ReadWrite<u32, IRQ_SOURCE0::Register>),
        (0x6C => IRQSource3: ReadWrite<u32, IRQ_SOURCE0::Register>),
        (0x70 => FIQSource0: ReadWrite<u32, FIQ_SOURCE0::Register>),
        (0x74 => FIQSource1: ReadWrite<u32, FIQ_SOURCE0::Register>),
        (0x78 => FIQSource2: ReadWrite<u32, FIQ_SOURCE0::Register>),
        (0x7C => FIQSource3: ReadWrite<u32, FIQ_SOURCE0::Register>),
        (0x80 => @END),
    }
);

register_structs!(ARMC {
    (0x0 => IRQ0Pending0: ReadWrite<u32, IRQ0_PENDING0::Register>),
    (0x4 => IRQ0Pending1: ReadWrite<u32, IRQ0_PENDING1::Register>),
    (0x8 => IRQ0Pending2: ReadWrite<u32, IRQ0_PENDING2::Register>),
    (0xC => _reserved1),

    (0x10 => IRQ0SetEN0: ReadWrite<u32, IRQ0_SET_EN_0::Register>),
    (0x14 => IRQ0SetEN1: ReadWrite<u32, IRQ0_SET_EN_1::Register>),
    (0x18 => IRQ0SetEN2: ReadWrite<u32, IRQ0_SET_EN_2::Register>),
    (0x1C=> _reserved2),

    (0x20 => IRQ0ClrEN0: ReadWrite<u32, IRQ0_CLR_EN_0::Register>),
    (0x24 => IRQ0ClrEN1: ReadWrite<u32, IRQ0_CLR_EN_1::Register>),
    (0x28 => IRQ0ClrEN2: ReadWrite<u32, IRQ0_CLR_EN_2::Register>),
    (0x2C=> _reserved3),

    (0x30 => IRQStatus0: ReadWrite<u32, IRQ_STATUS0::Register>),
    (0x34 => IRQStatus1: ReadWrite<u32, IRQ_STATUS1::Register>),
    (0x38 => IRQStatus2: ReadWrite<u32, IRQ_STATUS2::Register>),
    (0x3C=> _reserved4),

    (0x40 => IRQ1Pending0: ReadWrite<u32, IRQ1_PENDING0::Register>),
    (0x44 => IRQ1Pending1: ReadWrite<u32, IRQ1_PENDING1::Register>),
    (0x48 => IRQ1Pending2: ReadWrite<u32, IRQ1_PENDING2::Register>),
    (0x4C=> _reserved5),


    (0x50 => IRQ1SetEN0: ReadWrite<u32, IRQ1_SET_EN_0::Register>),
    (0x54 => IRQ1SetEN1: ReadWrite<u32, IRQ1_SET_EN_1::Register>),
    (0x58 => IRQ1SetEN2: ReadWrite<u32, IRQ1_SET_EN_2::Register>),
    (0x5C=> _reserved6),


    (0x60 => IRQ1ClrEN0: ReadWrite<u32, IRQ1_CLR_EN_0::Register>),
    (0x64 => IRQ1ClrEN1: ReadWrite<u32, IRQ1_CLR_EN_1::Register>),
    (0x68 => IRQ1ClrEN2: ReadWrite<u32, IRQ1_CLR_EN_2::Register>),
    (0x6C=> _reserved7),


    (0x80 => IRQ2Pending0: ReadWrite<u32, IRQ2_PENDING0::Register>),
    (0x84 => IRQ2Pending1: ReadWrite<u32, IRQ2_PENDING1::Register>),
    (0x88 => IRQ2Pending2: ReadWrite<u32, IRQ2_PENDING2::Register>),
    (0x8C => _reserved8),

    (0x90 => IRQ2SetEN0: ReadWrite<u32, IRQ2_SET_EN_0::Register>),
    (0x94 => IRQ2SetEN1: ReadWrite<u32, IRQ2_SET_EN_1::Register>),
    (0x98 => IRQ2SetEN2: ReadWrite<u32, IRQ2_SET_EN_2::Register>),
    (0x9C=> _reserved9),

    (0xA0 => IRQ2ClrEN0: ReadWrite<u32, IRQ2_CLR_EN_0::Register>),
    (0xA4 => IRQ2ClrEN1: ReadWrite<u32, IRQ2_CLR_EN_1::Register>),
    (0xA8 => IRQ2ClrEN2: ReadWrite<u32, IRQ2_CLR_EN_2::Register>),
    (0xAC=> _reserved10),


    (0xC0 => IRQ3Pending0: ReadWrite<u32, IRQ3_PENDING0::Register>),
    (0xC4 => IRQ3Pending1: ReadWrite<u32, IRQ3_PENDING1::Register>),
    (0xC8 => IRQ3Pending2: ReadWrite<u32, IRQ3_PENDING2::Register>),
    (0xCC => _reserved11),

    (0xD0 => IRQ3SetEN0: ReadWrite<u32, IRQ3_SET_EN_0::Register>),
    (0xD4 => IRQ3SetEN1: ReadWrite<u32, IRQ3_SET_EN_1::Register>),
    (0xD8 => IRQ3SetEN2: ReadWrite<u32, IRQ3_SET_EN_2::Register>),
    (0xDC=> _reserved12),

    (0xE0 => IRQ3ClrEN0: ReadWrite<u32, IRQ3_CLR_EN_0::Register>),
    (0xE4 => IRQ3ClrEN1: ReadWrite<u32, IRQ3_CLR_EN_1::Register>),
    (0xE8 => IRQ3ClrEN2: ReadWrite<u32, IRQ3_CLR_EN_2::Register>),
    (0xEC=> _reserved13),

    (0xF0 => @END),


});

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

static CORE_PS_TIMER_IRQ: IRQNum = IRQNum::PPI(29);
static VC_TIMER_IRQ: IRQNum = IRQNum::SPI(96);

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

        println!(
            "Group 0 forwarded {}",
            self.gicd.Ctlr.read(GICD_CTLR::EnableGrp0)
        );
        println!(
            "Group 1 forwarded {}",
            self.gicd.Ctlr.read(GICD_CTLR::EnableGrp1)
        );
        println!(
            "CPU Interface number {}",
            self.gicd.Typer.read(GICD_TYPER::CPUNumber) + 1
        );
        println!(
            "ILineNumber number {}",
            self.gicd.Typer.read(GICD_TYPER::ITLinesNumber)
        );

        let sec_ext = self.gicd.Typer.read(GICD_TYPER::SecurityExtn);
        println!("Security Extn implemented {}", sec_ext);

        self.gicd.Priority[0].modify(GICD_IPRIORITYR::Priority0.val(0xFF));
        let prio = self.gicd.Priority[0].read(GICD_IPRIORITYR::Priority0);
        self.gicd.Priority[0].modify(GICD_IPRIORITYR::Priority0.val(0x00));
        println!("Implemented priority bits {:#010b}", prio);

        let arch = self.gicc.Iidr.read(GICC_IIDR::ArchVersion);
        let arch_version = |v| match v {
            0x1 => "GICv1",
            0x2 => "GICv2",
            _ => "Unrecognized",
        };

        println!("GIC architecture {}", arch_version(arch));

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
        // set VC Timer0 to the highest priority
        self.set_priority(&VC_TIMER_IRQ, P0);

        // set VC Timer0 to target cpu0
        self.set_target_cpu(&VC_TIMER_IRQ, 0);

        // enable VC Timer0
        self.enable(&VC_TIMER_IRQ);

        // mask all interrupts other than ones with the highest priority
        self.gicc.Pmr.modify(GICC_PMR::Priority::P15);

        // enable GICD and GICC
        self.gicd.Ctlr.modify(GICD_CTLR::EnableGrp0::forwarded);
        self.gicc.Ctlr.modify(GICC_CTLR::EnableGrp0::forwarded);

        exception::local_irq_unmask();
        Ok(())
    }
    fn enable_timer(&self) {}
}
unsafe impl Send for GIC400 {}
unsafe impl Sync for GIC400 {}
