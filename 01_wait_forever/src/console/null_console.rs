use super::interface;
use core::fmt;

pub struct NullConsole;

pub static NULL_CONSOLE: NullConsole = NullConsole {};

impl interface::Write for NullConsole {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        Ok(())
    }
}

impl interface::Read for NullConsole {
    fn clear_rx(&mut self) {}
}

impl interface::Statistics for NullConsole {}
impl interface::All for NullConsole {}
