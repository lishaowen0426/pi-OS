use crate::console::{CONSOLE, UNSAFE_CONSOLE};
use core::fmt;

#[cfg(not(feature = "build_qemu"))]
pub fn _print(args: fmt::Arguments) {
    CONSOLE.get().unwrap().write_fmt(args).unwrap();
}
#[cfg(not(feature = "build_qemu"))]
pub fn _unsafe_print(args: fmt::Arguments) {
    UNSAFE_CONSOLE.write_fmt(args).unwrap();
}
#[cfg(feature = "build_qemu")]
pub fn _unsafe_print(args: fmt::Arguments) {
    _print(args)
}

#[cfg(feature = "build_qemu")]
pub fn _print(args: fmt::Arguments) {
    use crate::console::QEMU_CONSOLE;
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
macro_rules! unsafe_print {
    ($($arg:tt)*) => ($crate::print::_unsafe_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print::_print("\n"));
    ($($arg:tt)*) => ({
        $crate::print::_print(format_args_nl!($($arg)*));
    })
}

#[macro_export]
macro_rules! unsafe_println {
    () => ($crate::print::_unsafe_print("\n"));
    ($($arg:tt)*) => ({
        $crate::print::_unsafe_print(format_args_nl!($($arg)*));
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
#[macro_export]
macro_rules! unsafe_println_0 {
    () => ($crate::print::_unsafe_print("\n"));
    ($($arg:tt)*) => ({
        $crate::print::_unsafe_print(format_args_nl!("{}",format_args!($($arg)*)));
    })
}
#[macro_export]
macro_rules! unsafe_println_1 {
    () => ($crate::print::_unsafe_print("\n"));
    ($($arg:tt)*) => ({
        $crate::print::_unsafe_print(format_args_nl!("     {}",format_args!($($arg)*)));
    })
}
#[macro_export]
macro_rules! unsafe_println_2 {
    () => ($crate::print::_unsafe_print("\n"));
    ($($arg:tt)*) => ({
        $crate::print::_unsafe_print(format_args_nl!("         {}",format_args!($($arg)*)));
    })
}
