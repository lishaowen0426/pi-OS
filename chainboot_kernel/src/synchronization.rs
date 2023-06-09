pub mod spin;

pub type Spinlock<T> = lock_api::Mutex<spin::RawSpinlock, T>;
pub type SpinlockGuard<'a, T> = lock_api::MutexGuard<'a, spin::RawSpinlock, T>;
