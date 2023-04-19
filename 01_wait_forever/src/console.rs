#[cfg(feature = "bsp_rpi3")]
use crate::println_qemu;
use crate::{bsp::device_driver::MiniUart, println_debug, synchronization::Spinlock};

mod null_console;

pub mod interface {
    pub use core::fmt::Write;

    pub trait Read {
        fn read_char(&mut self) -> char {
            ' '
        }

        fn clear_rx(&mut self);
    }

    pub trait Statistics {
        fn chars_written(&self) -> usize {
            0
        }
        fn chars_read(&self) -> usize {
            0
        }
    }

    pub trait All: Write + Read + Statistics {}
}

const CLOCK: u64 = 500000000;
const BAUD_RATE: u32 = 115200;
pub static CONSOLE: Spinlock<MiniUart> = Spinlock::new(MiniUart::new());

/// DEBUG_CONSOLE is unsafe(i.e., without lock)
/// println_debug! uses DEBUG_CONSOLE to directly write to mini_uart
pub static mut DEBUG_CONSOLE: MiniUart = MiniUart::new();

pub fn init_debug_console() {
    unsafe {
        DEBUG_CONSOLE.init(CLOCK, BAUD_RATE);
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
impl interface::Write for QemuConsole {
    fn write_str(&mut self, s: &str) -> fmt::Result {
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

#[no_mangle]
#[inline(never)]
pub fn init_console() {
    CONSOLE.lock().init(CLOCK, BAUD_RATE);
}
