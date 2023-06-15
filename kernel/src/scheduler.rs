extern crate alloc;
use crate::{
    errno::ErrorCode,
    generics::{DoublyLinkedList, Link},
    println,
    synchronization::Spinlock,
};
use alloc::boxed::Box;
use spin::{once::Once, Spin};

mod context_switch;
mod task;
pub use context_switch::*;
pub use task::*;

pub struct UnSafeScheduler {
    tasks: DoublyLinkedList<Task>,
}

impl UnSafeScheduler {
    fn new() -> Self {
        Self {
            tasks: DoublyLinkedList::new(),
        }
    }

    fn add_task(&mut self, t: Box<Task>) {
        self.tasks.push_front(Link::some(Box::into_raw(t) as usize));
    }

    fn schedule(&self) -> *mut Task {
        self.tasks.head().resolve_mut() as *mut Task
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

    pub fn schedule(&self) -> *mut Task {
        self.sched.lock().schedule()
    }
}

pub fn sched_test() -> ! {
    loop {
        println!("Hello Scheduler");
    }
}

pub fn init() -> Result<(), ErrorCode> {
    SCHEDULER.call_once(|| Scheduler::new());
    SCHEDULER
        .get()
        .unwrap()
        .add_task(Box::new(Task::new(sched_test as u64)));
    Ok(())
}

pub static SCHEDULER: Once<Scheduler> = Once::new();
