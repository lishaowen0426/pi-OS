extern crate alloc;
use crate::{
    bsp::device_driver::interrupt_controller, errno::ErrorCode, synchronization::Spinlock,
};
use alloc::boxed::Box;
use spin::{Once, Spin};

pub struct IRQDescriptor {
    num: interrupt_controller::IRQNum,
    name: &'static str,
}

pub trait IRQHandler {
    fn handle(&self) -> Result<(), ErrorCode>;
}

pub trait IRQController {
    fn init(&mut self) -> Result<(), ErrorCode>;
    fn enable_timer(&self);
}

pub struct GenericIRQController {
    controller: Spinlock<Box<dyn IRQController + Sync + Send>>,
}

impl GenericIRQController {
    fn new() -> Self {
        Self {
            controller: Spinlock::new(Box::new(interrupt_controller::create())),
        }
    }
    pub fn enable_timer(&self) {
        self.controller.lock().enable_timer()
    }

    pub fn init(&self) -> Result<(), ErrorCode> {
        self.controller.lock().init()
    }
}

pub fn init() -> Result<(), ErrorCode> {
    IRQ_CONTROLLER.call_once(|| GenericIRQController::new());
    IRQ_CONTROLLER.get().unwrap().init()
}

pub static IRQ_CONTROLLER: Once<GenericIRQController> = Once::new();
