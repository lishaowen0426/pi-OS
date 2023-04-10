use crate::{console, synchronization::Spinlock};
use core::fmt;

struct QEMUOutputInner {
    chars_written: usize,
}

struct QEMUOutput {
    inner: Spinlock<QEMUOutputInner>,
}

static QEMU_OUTPUT: QEMUOutput = QEMUOutput::new();

impl QEMUOutputInner {
    const fn new() -> Self {
        Self { chars_written: 0 }
    }

    fn write_char(&mut self, c: char) {
        unsafe {
            core::ptr::write_volatile(0x3F20_1000 as *mut u8, c as u8);
        }
        self.chars_written += 1;
    }
}

impl fmt::Write for QEMUOutputInner {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            if c == '\n' {
                self.write_char('\r')
            }

            self.write_char(c);
        }

        Ok(())
    }
}

impl QEMUOutput {
    pub const fn new() -> Self {
        Self {
            inner: Spinlock::new(QEMUOutputInner::new()),
        }
    }
}

pub fn console() -> &'static dyn console::interface::All {
    &QEMU_OUTPUT
}

impl console::interface::Write for QEMUOutput {
    fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result {
        let mut locked = self.inner.lock();
        fmt::Write::write_fmt(&mut *locked, args)
    }
}

impl console::interface::Statistics for QEMUOutput {
    fn chars_written(&self) -> usize {
        let locked = self.inner.lock();
        locked.chars_written
    }
}

impl console::interface::All for QEMUOutput {}
