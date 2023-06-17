use crate::{
    generics::{DoublyLink, DoublyLinkable, DoublyLinkedList, Link},
    memory,
    memory::{address::AddressRange, *},
    scheduler::context_switch::Context,
};
use test_macros::doubly_linkable;

#[doubly_linkable]
#[derive(Default)]
#[repr(C)]
pub struct Task {
    ctx: Context,
}

impl Task {
    pub fn set_sp(&mut self, sp: u64) {
        self.ctx.sp = sp;
    }
    pub fn set_lr(&mut self, lr: u64) {
        self.ctx.lr = lr;
    }
    pub fn new(lr: u64) -> Self {
        let mapped = MMU
            .get()
            .unwrap()
            .kzalloc(1, RWNORMAL, HIGHER_PAGE)
            .unwrap();
        Self {
            ctx: Context {
                gpr: [0; 11],
                sp: mapped.va.start().value() as u64,
                lr,
            },
            ..Default::default()
        }
    }
    pub fn new_with_sp(sp: u64, lr: u64) -> Self {
        Self {
            ctx: Context {
                gpr: [0; 11],
                sp,
                lr,
            },
            ..Default::default()
        }
    }
}
