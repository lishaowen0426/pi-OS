extern crate alloc;
use crate::{
    bsp::device_driver::interrupt_controller,
    errno::ErrorCode,
    synchronization::{IRQSafeSpinlock, Spinlock},
};
use alloc::boxed::Box;
use spin::{Once, Spin};

pub trait IRQHandler {
    fn handle(&self) -> Result<(), ErrorCode>;
}

pub trait IRQController {
    fn init(&mut self) -> Result<(), ErrorCode>;
    fn handle(&self) -> Result<(), ErrorCode>;
}

pub struct GenericIRQController {
    controller: IRQSafeSpinlock<Box<dyn IRQController + Sync + Send>>,
}

impl GenericIRQController {
    fn new() -> Self {
        Self {
            controller: IRQSafeSpinlock::new(Box::new(interrupt_controller::create())),
        }
    }

    pub fn init(&self) -> Result<(), ErrorCode> {
        self.controller.lock().init()
    }
    pub fn handle(&self) -> Result<(), ErrorCode> {
        self.controller.lock().handle()
    }
}

pub fn init() -> Result<(), ErrorCode> {
    IRQ_CONTROLLER.call_once(|| GenericIRQController::new());
    IRQ_CONTROLLER.get().unwrap().init()
}

pub static IRQ_CONTROLLER: Once<GenericIRQController> = Once::new();
