
#[cfg(target_arch = "aarch64")]
#[path = "_arch/aarch64/synronization_primitives.rs"]
mod primitive;

pub mod spin;

pub type Spinlock<T> = lock_api::Mutex<spin::RawSpinlock, T>;

#[allow(dead_code)]
pub type SpinlockGuard<'a, T> = lock_api::MutexGuard<'a, spin::RawSpinlock, T>;
