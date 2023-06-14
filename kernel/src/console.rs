use crate::{bsp::device_driver::MiniUartInner, errno::ErrorCode, synchronization::Spinlock};
use core::fmt;
use spin::once::Once;

pub struct Console<T>
where
    T: fmt::Write,
{
    io: Spinlock<T>,
}

impl<T> Console<T>
where
    T: fmt::Write,
{
    pub fn new(inner: T) -> Self {
        Self {
            io: Spinlock::new(inner),
        }
    }
    pub fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result {
        write!(self.io.lock(), "{}", args)
    }
}

pub static CONSOLE: Once<Console<MiniUartInner>> = Once::new();

pub fn init() -> Result<(), ErrorCode> {
    CONSOLE.call_once(|| Console {
        io: Spinlock::new(MiniUartInner::new()),
    });
    Ok(())
}

#[cfg(feature = "build_qemu")]
pub struct QemuConsole;

#[cfg(feature = "build_qemu")]
pub static mut QEMU_CONSOLE: QemuConsole = QemuConsole;

#[cfg(feature = "build_qemu")]
impl core::fmt::Write for QemuConsole {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            unsafe {
                core::ptr::write_volatile(0x3F20_1000 as *mut u8, c as u8);
            }
        }
        Ok(())
    }
}
