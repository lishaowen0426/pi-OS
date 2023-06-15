use crate::{
    generics::{DoublyLinkable, DoublyLinkedList, Link},
    memory,
    memory::{address::AddressRange, *},
    scheduler::context_switch::Context,
};

#[repr(C)]
pub struct Task {
    ctx: Context,
    prev_link: Link<Task>,
    next_link: Link<Task>,
}

impl DoublyLinkable for Task {
    type T = Self;
    fn set_prev(&mut self, link: Link<Self::T>) {
        self.prev_link = link;
    }
    fn set_next(&mut self, link: Link<Self::T>) {
        self.next_link = link;
    }
    fn prev(&self) -> Link<Self::T> {
        self.prev_link
    }
    fn next(&self) -> Link<Self::T> {
        self.next_link
    }
}

impl Task {
    pub fn new(lr: u64) -> Self {
        let mapped = MMU
            .get()
            .unwrap()
            .kzalloc(BLOCK_4K, RWNORMAL, HIGHER_PAGE)
            .unwrap();
        Self {
            ctx: Context {
                gpr: [0; 11],
                sp: mapped.va.start().value() as u64,
                lr,
            },
            prev_link: Link::none(),
            next_link: Link::none(),
        }
    }
    pub fn new_with_sp(sp: u64, lr: u64) -> Self {
        Self {
            ctx: Context {
                gpr: [0; 11],
                sp,
                lr,
            },
            prev_link: Link::none(),
            next_link: Link::none(),
        }
    }
}
