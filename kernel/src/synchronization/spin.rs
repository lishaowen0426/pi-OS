use core::sync::atomic::AtomicU8;
use lock_api::{GuardSend, RawMutex};

pub struct RawSpinlock(pub AtomicU8);

/// certain conditioins need to occur before
/// atomics can really work on aarch64
/// see https://stackoverflow.com/questions/68785276/bare-metal-spinlock-implementation-in-rust
unsafe impl RawMutex for RawSpinlock {
    const INIT: RawSpinlock = RawSpinlock(AtomicU8::new(0));

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
