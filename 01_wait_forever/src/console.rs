use crate::{bsp, synchronization::Spinlock};

mod null_console;

pub mod interface {
    use core::fmt;
    pub trait Write {
        fn write_char(&self, c: char);

        fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result;

        fn flush(&self);
    }

    pub trait Read {
        fn read_char(&self) -> char {
            ' '
        }

        fn clear_rx(&self);
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

static CUR_CONSOLE: Spinlock<&'static (dyn interface::All + Sync)> =
    Spinlock::new(&null_console::NULL_CONSOLE);

pub fn register_console(new_console: &'static (dyn interface::All + Sync)) {
    let mut locked = CUR_CONSOLE.lock();
    *locked = new_console;
}

pub fn console() -> &'static dyn interface::All {
    let locked = CUR_CONSOLE.lock();
    *locked
}
