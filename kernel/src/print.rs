use crate::bsp::device_driver::mini_uart::*;
use core::{fmt, fmt::Write};
use spin::once::Once;

#[cfg(feature = "build_qemu")]
pub fn _print_qemu(args: fmt::Arguments) {
    use crate::console::QEMU_CONSOLE;
    unsafe {
        QEMU_CONSOLE.write_fmt(args).unwrap();
    }
}

pub fn _print(args: fmt::Arguments) {
    #[cfg(feature = "build_qemu")]
    {
        use crate::console::QEMU_CONSOLE;
        unsafe {
            QEMU_CONSOLE.write_fmt(args).unwrap();
        }
    }
    #[cfg(feature = "bsp_rpi4")]
    {
        use crate::bsp::device_driver::mini_uart::MINI_UART;
        MINI_UART.get().unwrap().write_fmt(args).unwrap();
    }
    #[cfg(feature = "build_chainloader")]
    {
        let mut muart = UnSafeMiniUart::new(VIRTUAL_MINI_UART_START);
        muart.write_fmt(args).unwrap();
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
#[cfg(feature = "build_qemu")]
#[macro_export]
macro_rules! println_qemu {
    () => ($crate::print::_print_qemu("\n"));
    ($($arg:tt)*) => ({
        $crate::print::_print_qemu(format_args_nl!($($arg)*));
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
