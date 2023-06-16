use crate::scheduler::task::Task;
use core::arch::asm;

extern "C" {
    pub fn __cpu_switch_to(prev: *mut Task, next: *mut Task);
}

#[derive(Default)]
#[repr(C)]
pub struct Context {
    pub gpr: [u64; 11], // x19 - x29
    pub sp: u64,
    pub lr: u64,
}
