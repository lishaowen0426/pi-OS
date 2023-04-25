use crate::{bsp::device_driver::MINI_UART, synchronization::Spinlock};

pub mod interface {
    pub use core::fmt;

    pub trait Console {
        fn init(&self);
        fn _write_str(&self, s: &str) -> fmt::Result;
        fn _write_char(&self, c: char) -> fmt::Result;
        fn _write_fmt(&self, args: fmt::Arguments) -> fmt::Result;
        fn _flush(&self);
        fn _read_char(&self) -> char;
        fn _clear_rx(&self);
        fn _chars_written(&self) -> usize;
        fn _chars_read(&self) -> usize;
    }
}

pub static CONSOLE: Spinlock<&'static (dyn interface::Console + Sync)> = Spinlock::new(&MINI_UART);

/// DEBUG_CONSOLE is unsafe(i.e., without lock)
/// println_debug! uses DEBUG_CONSOLE to directly write to mini_uart
pub static mut DEBUG_CONSOLE: &'static dyn interface::Console = &MINI_UART;

pub fn init_debug_console() {
    unsafe {
        DEBUG_CONSOLE.init();
    }
}

#[cfg(feature = "bsp_rpi3")]
pub struct QemuConsole;

#[cfg(feature = "bsp_rpi3")]
impl QemuConsole {
    pub const fn new() -> Self {
        Self {}
    }
}

#[cfg(feature = "bsp_rpi3")]
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

#[cfg(feature = "bsp_rpi3")]
pub static mut QEMU_CONSOLE: QemuConsole = QemuConsole::new();

#[inline(always)]
pub fn init_console() {
    CONSOLE.lock().init();
}

#[inline(always)]
pub fn console() -> &'static dyn interface::Console {
    *CONSOLE.lock()
}
