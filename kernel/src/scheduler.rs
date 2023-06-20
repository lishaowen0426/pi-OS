extern crate alloc;
use crate::{
    errno::*,
    generics::{DoublyLinkedList, Link},
    memory::*,
    println,
    synchronization::Spinlock,
};
use aarch64_cpu::{asm::barrier, registers::*};
use alloc::boxed::Box;
use core::arch::asm;
use spin::{once::Once, Spin};
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

mod context_switch;
mod task;
#[cfg(not(feature = "build_qemu"))]
use crate::bsp::device_driver::gic_400::IRQNum::SPI;
use crate::memory::address::AddressRange;
pub use context_switch::*;
pub use task::*;

const NUM_OF_CORES: usize = 4;
const CORE_ID: usize = 0;

#[derive(Copy, Clone)]
struct RunQueue {
    tasks: DoublyLinkedList<Task>,
    current: Option<*mut Task>,
}

unsafe impl Sync for RunQueue {}
unsafe impl Send for RunQueue {}

impl RunQueue {
    fn new() -> Self {
        Self {
            tasks: DoublyLinkedList::new(),
            current: None,
        }
    }

    fn add_task(&mut self, t: Box<Task>) {
        self.tasks.push_front(Link::some(Box::into_raw(t) as usize));
    }

    fn get_task(&mut self) -> Option<*mut Task> {
        let t = self.tasks.pop_front()?;
        Some(t.resolve_mut() as *mut Task)
    }

    fn replace_current(&mut self, t: *mut Task) {
        if let Some(c) = self.current {
            self.tasks.push_front(Link::some(c as usize));
        }

        self.current = Some(t)
    }
}

pub struct UnSafeScheduler {
    rq: [RunQueue; NUM_OF_CORES],
}

impl UnSafeScheduler {
    fn new() -> Self {
        Self {
            rq: [RunQueue::new(); NUM_OF_CORES],
        }
    }

    fn add_task(&mut self, t: Box<Task>) {
        self.rq[CORE_ID].add_task(t);
    }

    fn schedule(&mut self) -> Option<*mut Task> {
        let t = self.rq[CORE_ID].get_task()?;
        Some(t as *mut Task)
    }
    fn replace_current(&mut self, t: *mut Task) {
        self.rq[CORE_ID].replace_current(t)
    }
}

pub struct Scheduler {
    sched: Spinlock<UnSafeScheduler>,
}

impl Scheduler {
    fn new() -> Self {
        Self {
            sched: Spinlock::new(UnSafeScheduler::new()),
        }
    }

    pub fn add_task(&self, t: Box<Task>) {
        self.sched.lock().add_task(t)
    }

    pub fn schedule(&self) -> Option<*mut Task> {
        self.sched.lock().schedule()
    }

    pub fn init_task(&self) -> ! {
        let mut t = Box::new(Task::default());
        let stack = MMU.get().unwrap().allocate_stack(1).unwrap();
        t.set_sp(stack.va.start().value());
        t.set_lr(sched_test as usize);
        self.sched.lock().replace_current(Box::into_raw(t));

        SPSR_EL1.write(
            SPSR_EL1::M::EL0t
                + SPSR_EL1::D::Unmasked
                + SPSR_EL1::A::Unmasked
                + SPSR_EL1::I::Unmasked
                + SPSR_EL1::F::Unmasked,
        );
        SP_EL0.set(stack.va.start().value() as u64);
        ELR_EL1.set(sched_test as u64);

        barrier::isb(barrier::SY);
        unsafe {
            asm!("eret");
        }

        loop {}
    }
}

pub fn sched_test() -> ! {
    println!("Hello Scheduler");
    let a = 5u32;
    let b = 6u32;
    let mut c = 0u32;
    loop {
        unsafe {
            core::ptr::write_volatile(&mut c as *mut u32, a + b);
        }
    }
}

pub fn init() -> Result<(), ErrorCode> {
    SCHEDULER.call_once(|| Scheduler::new());

    Ok(())
}

pub static SCHEDULER: Once<Scheduler> = Once::new();
