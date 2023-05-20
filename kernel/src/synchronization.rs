pub mod primitive;

pub type Spinlock<T> = lock_api::Mutex<primitive::PhantomSpinlock, T>;

#[allow(dead_code)]
pub type SpinlockGuard<'a, T> = lock_api::MutexGuard<'a, primitive::PhantomSpinlock, T>;
