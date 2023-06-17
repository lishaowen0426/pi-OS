use crate::{
    generics::{DoublyLink, DoublyLinkable, DoublyLinkedList, Link},
    memory,
    memory::{address::AddressRange, *},
    scheduler::context_switch::Context,
};
use test_macros::doubly_linkable;

#[doubly_linkable]
#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct Task {
    ctx: Context,
}

impl Task {
    pub fn set_sp(&mut self, sp: usize) {
        self.ctx.sp = sp as u64;
    }
    pub fn set_lr(&mut self, lr: usize) {
        self.ctx.lr = lr as u64;
    }

    pub fn get_sp(&self) -> usize {
        self.ctx.sp as usize
    }
    pub fn get_lr(&self) -> usize {
        self.ctx.lr as usize
    }
}
