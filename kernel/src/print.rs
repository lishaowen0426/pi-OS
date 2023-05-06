use crate::console::QEMU_CONSOLE;

use crate::console::{CONSOLE, DEBUG_CONSOLE};
use core::fmt;

pub fn _print(args: fmt::Arguments) {
    let locked = CONSOLE.lock();
    locked._write_fmt(args).unwrap();
}

pub fn _print_debug(args: fmt::Arguments) {
    unsafe {
        DEBUG_CONSOLE._write_fmt(args).unwrap();
    }
}

pub fn _print_qemu(args: fmt::Arguments) {
    use core::fmt::Write;
    unsafe {
        QEMU_CONSOLE.write_fmt(args).unwrap();
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::print::_print(format_args!($($arg)*)));
}

#[cfg(feature = "build_qemu")]
#[macro_export]
macro_rules! println {
    () => ($crate::print::_print_qemu("\n"));
    ($($arg:tt)*) => ({
        $crate::print::_print_qemu(format_args_nl!($($arg)*));
    })
}

#[cfg(not(feature = "build_qemu"))]
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
