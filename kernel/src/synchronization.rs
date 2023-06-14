pub mod primitive;

pub type Spinlock<T> = lock_api::Mutex<primitive::RawSpinlock, T>;
pub type SpinlockGuard<'a, T> = lock_api::MutexGuard<'a, primitive::RawSpinlock, T>;

pub type IRQSafeSpinlock<T> = lock_api::Mutex<primitive::IRQSafeSpinlock, T>;
pub type IRQSafeSpinlockGuard<'a, T> = lock_api::MutexGuard<'a, primitive::IRQSafeSpinlock, T>;
