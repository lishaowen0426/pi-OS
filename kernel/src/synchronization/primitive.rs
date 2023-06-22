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

use crate::{exception, println};
use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, AtomicU8, Ordering},
};
use lock_api::{GuardSend, RawMutex};

pub struct RawSpinlock {
    locked: AtomicBool,
}

impl RawSpinlock {
    const fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
        }
    }
}

unsafe impl RawMutex for RawSpinlock {
    const INIT: Self = RawSpinlock::new();

    type GuardMarker = GuardSend;

    fn lock(&self) {
        while !self.try_lock() {}
    }

    fn try_lock(&self) -> bool {
        self.locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    unsafe fn unlock(&self) {
        self.locked.store(false, Ordering::Release)
    }
}

pub struct IRQSafeSpinlock {
    locked: RawSpinlock,
}

impl IRQSafeSpinlock {
    const fn new() -> Self {
        Self {
            locked: RawSpinlock::new(),
        }
    }
}

unsafe impl RawMutex for IRQSafeSpinlock {
    const INIT: Self = IRQSafeSpinlock::new();

    type GuardMarker = GuardSend;

    fn lock(&self) {
        while !self.try_lock() {}
    }

    fn try_lock(&self) -> bool {
        exception::local_irq_mask();
        if self.locked.try_lock() {
            true
        } else {
            exception::local_irq_unmask();
            false
        }
    }

    unsafe fn unlock(&self) {
        self.locked.unlock();
        exception::local_irq_unmask();
    }
}

#[cfg(test)]
#[allow(unused_imports, unused_variables, dead_code)]
mod tests {
    use super::*;
    use test_macros::kernel_test;
}
