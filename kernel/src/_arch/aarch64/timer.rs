use crate::{errno::ErrorCode, println, scheduler::*, synchronization::Spinlock};
use aarch64_cpu::{asm::barrier, registers::*};
use core::{num::NonZeroU64, ops::Div, time::Duration};
use spin::once::Once;
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

// this will be overwritten in boot.s
// the frequency is in Hz
#[no_mangle]
static SYSTEM_COUNTER_FREQUENCY: NonZeroU64 = NonZeroU64::MIN;

const NANOSEC_PER_SEC: NonZeroU64 = NonZeroU64::new(1_000_000_000).unwrap();

pub fn system_counter_frequency() -> NonZeroU64 {
    unsafe { core::ptr::read_volatile(&SYSTEM_COUNTER_FREQUENCY) }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub struct GenericPhysicalCounter(u64);

impl GenericPhysicalCounter {
    pub const MAX: Self = GenericPhysicalCounter(u64::MAX);

    #[inline(always)]
    fn read_cntpct() -> u64 {
        barrier::isb(barrier::SY);
        CNTPCT_EL0.get()
    }

    pub fn read() -> Self {
        GenericPhysicalCounter(Self::read_cntpct())
    }
}

impl From<GenericPhysicalCounter> for Duration {
    fn from(value: GenericPhysicalCounter) -> Self {
        if value.0 == 0 {
            return Duration::ZERO;
        }

        let frequency = system_counter_frequency().get();

        let secs = value.0.div_floor(frequency);

        let subsec = value.0 % frequency;

        let nanos = subsec
            .checked_mul(NANOSEC_PER_SEC.get())
            .unwrap()
            .div_floor(frequency) as u32;

        Duration::new(secs, nanos)
    }
}

pub struct Timer {
    frequency: u64,
}

impl Timer {
    fn new() -> Self {
        Self {
            frequency: system_counter_frequency().get(),
        }
    }

    pub fn now(&self) -> Duration {
        Duration::from(GenericPhysicalCounter::read())
    }

    pub fn enable(&self) {
        CNTP_TVAL_EL0.set(self.frequency);
        CNTP_CTL_EL0.modify(CNTP_CTL_EL0::IMASK.val(0) + CNTP_CTL_EL0::ENABLE.val(1));
        barrier::isb(barrier::SY);
    }

    pub fn reset(&self) {
        CNTP_TVAL_EL0.set(2 * self.frequency);
        barrier::isb(barrier::SY);
    }

    pub fn disable(&self) {
        CNTP_CTL_EL0.modify(CNTP_CTL_EL0::IMASK.val(1) + CNTP_CTL_EL0::ENABLE.val(0));
        barrier::isb(barrier::SY);
    }
}

pub fn init() -> Result<(), ErrorCode> {
    TIMER.call_once(|| Timer::new());

    Ok(())
}

pub fn handle_interrupt() -> Result<(), ErrorCode> {
    println!("handle timer");
    TIMER.get().unwrap().disable();
    let task = SCHEDULER.get().unwrap().schedule();
    let buf = [0u64; 13];
    unsafe {
        __cpu_switch_to(buf.as_ptr() as *mut Task, task);
    }
    Ok(())
}

pub static TIMER: Once<Timer> = Once::new();
