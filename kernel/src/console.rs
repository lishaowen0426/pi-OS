use crate::{bsp::device_driver::mini_uart, errno::ErrorCode, synchronization::Spinlock};
use core::fmt;
use spin::once::Once;

pub fn init() -> Result<(), ErrorCode> {
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
