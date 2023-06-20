use core::{fmt, fmt::Write};
use spin::once::Once;

#[cfg(feature = "build_qemu")]
pub fn _unsafe_print(args: fmt::Arguments) {
    _print(args)
}

pub fn _print(args: fmt::Arguments) {
    #[cfg(feature = "build_qemu")]
    {
        use crate::console::QEMU_CONSOLE;
        unsafe {
            QEMU_CONSOLE.write_fmt(args).unwrap();
        }
    }
    #[cfg(not(feature = "build_qemu"))]
    {
        use crate::bsp::device_driver::mini_uart::MINI_UART;
        MINI_UART.get().unwrap().write_fmt(args).unwrap();
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
