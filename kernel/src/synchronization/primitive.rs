//! From https://stackoverflow.com/questions/68785276/bare-metal-spinlock-implementation-in-rust
//!
//! Here's a non-exhaustive list of conditions that I'm certain (I've tested them both ways) need
//! to occur in order for atomics to work on RPi 4 aarch64 in EL1 (ARMv8).   
//!     1. This will probably be very similar to RPi 3 (ARMv7).   
//!     2. MMU must be enabled (SCTLR_EL1 bit [0] set to0b1)   
//!     3. Data caching must be enabled (SCTLR_EL1 bit [2] set to 0b1)
//!     4. The page on which the lock resides has to be marked as normal, cachable memory via MAIR
//!
//! The means at the very beginning of kernel boot, we CANNOT use locks relying on these atomics

use crate::println;
use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, AtomicU8, Ordering},
};
use lock_api::{GuardSend, RawMutex};

pub struct PhantomSpinlock(pub AtomicU8);

unsafe impl RawMutex for PhantomSpinlock {
    const INIT: PhantomSpinlock = PhantomSpinlock(AtomicU8::new(0));

    type GuardMarker = GuardSend;

    fn lock(&self) {
        while !self.try_lock() {}
    }
    #[no_mangle]
    #[inline(never)]
    fn try_lock(&self) -> bool {
        true
    }

    #[no_mangle]
    unsafe fn unlock(&self) {}
}

pub struct SpinLock<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}

pub struct SpinLockGuard<'a, T> {
    lock: &'a SpinLock<T>,
}

unsafe impl<T> Sync for SpinLock<T> where T: Send {}

impl<T> SpinLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    pub fn lock(&self) -> SpinLockGuard<T> {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {}
        println!("Locked the spin");
        SpinLockGuard { lock: self }
    }
}

impl<T> Deref for SpinLockGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.value.get() }
    }
}

impl<T> DerefMut for SpinLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.value.get() }
    }
}

impl<T> Drop for SpinLockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release);
        println!("drop the lock...");
    }
}

#[cfg(test)]
#[allow(unused_imports, unused_variables, dead_code)]
mod tests {
    use super::*;
    use test_macros::kernel_test;
    #[kernel_test]
    fn test_spinlock() {
        {
            let l = SpinLock::new(32 as u32);
            let a = l.lock().pow(2);
        }
    }
}
