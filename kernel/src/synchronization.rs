pub mod primitive;

pub type Spinlock<T> = lock_api::Mutex<primitive::RawSpinlock, T>;
pub type SpinlockGuard<'a, T> = lock_api::MutexGuard<'a, primitive::RawSpinlock, T>;
pub type MappedSpinlockGuard<'a, T> = lock_api::MappedMutexGuard<'a, primitive::RawSpinlock, T>;

pub type IRQSafeSpinlock<T> = lock_api::Mutex<primitive::IRQSafeSpinlock, T>;
pub type IRQSafeSpinlockGuard<'a, T> = lock_api::MutexGuard<'a, primitive::IRQSafeSpinlock, T>;

pub type SpinRwLock<T> = lock_api::RwLock<primitive::RawRwSpinlock, T>;
pub type SpinRwLockReadGuard<'a, T> = lock_api::RwLockReadGuard<'a, primitive::RawRwSpinlock, T>;
pub type SpinRwLockWriteGuard<'a, T> = lock_api::RwLockWriteGuard<'a, primitive::RawRwSpinlock, T>;
pub type MappedSpinRwLockReadGuard<'a, T> =
    lock_api::MappedRwLockReadGuard<'a, primitive::RawRwSpinlock, T>;
pub type MappedSpinRwLockWriteGuard<'a, T> =
    lock_api::MappedRwLockWriteGuard<'a, primitive::RawRwSpinlock, T>;
