#[cfg(feature = "bsp_rpi3")]
use crate::console::QEMU_CONSOLE;

use crate::console::{CONSOLE, DEBUG_CONSOLE};
use core::fmt;

use crate::console::interface::Write;

pub fn _print(args: fmt::Arguments) {
    let mut locked = CONSOLE.lock();
    locked.write_fmt(args).unwrap();
}

pub fn _print_debug(args: fmt::Arguments) {
    unsafe {
        DEBUG_CONSOLE.write_fmt(args).unwrap();
    }
}

#[cfg(feature = "bsp_rpi3")]
pub fn _print_qemu(args: fmt::Arguments) {
    unsafe {
        QEMU_CONSOLE.write_fmt(args).unwrap();
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::print::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ({
        $crate::print::_print(format_args_nl!($($arg)*));
    })
}

#[macro_export]
macro_rules! println_debug {
    () => ($crate::print::_print_debug(format_args_nl!($($arg)*)));
    ($($arg:tt)*) => ({
        $crate::print::_print_debug(format_args_nl!($($arg)*));
    })
}

#[cfg(feature = "bsp_rpi3")]
#[macro_export]
macro_rules! println_qemu {
    () => ($crate::print::_print_qemu("\n"));
    ($($arg:tt)*) => ({
        $crate::print::_print_qemu(format_args_nl!($($arg)*));
    })
}
