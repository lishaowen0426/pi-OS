use core::{marker::PhantomData, ops::Deref};

#[cfg(target_arch = "aarch64")]
#[path = "_arch/aarch64/mmu.rs"]
mod arch_mmu;

pub use arch_mmu::*;

pub struct MMIOWrapper<T> {
    start_addr: usize,
    _marker: PhantomData<T>,
}

impl<T> MMIOWrapper<T> {
    pub const fn new(start_addr: usize) -> Self {
        Self {
            start_addr,
            _marker: PhantomData,
        }
    }
}

impl<T> Deref for MMIOWrapper<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.start_addr as *const _) }
    }
}
