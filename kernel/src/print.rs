use crate::console::QEMU_CONSOLE;

use crate::console::{CONSOLE, DEBUG_CONSOLE};
use core::fmt;

pub fn _print_debug(args: fmt::Arguments) {
    unsafe {
        DEBUG_CONSOLE._write_fmt(args).unwrap();
    }
}

#[cfg(not(feature = "build_qemu"))]
pub fn _print(args: fmt::Arguments) {
    let locked = CONSOLE.lock();
    locked._write_fmt(args).unwrap();
}
#[cfg(feature = "build_qemu")]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
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
    () => ($crate::print::_print("\n"));
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

#[macro_export]
macro_rules! println_0 {
    () => ($crate::print::_print("\n"));
    ($($arg:tt)*) => ({
        $crate::print::_print(format_args_nl!("{}",format_args!($($arg)*)));
    })
}
#[macro_export]
macro_rules! println_1 {
    () => ($crate::print::_print("\n"));
    ($($arg:tt)*) => ({
        $crate::print::_print(format_args_nl!("     {}",format_args!($($arg)*)));
    })
}
#[macro_export]
macro_rules! println_2 {
    () => ($crate::print::_print("\n"));
    ($($arg:tt)*) => ({
        $crate::print::_print(format_args_nl!("         {}",format_args!($($arg)*)));
    })
}
