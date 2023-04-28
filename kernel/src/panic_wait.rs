// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2022 Andre Richter <andre.o.richter@gmail.com>

//! A panic handler that infinitely waits.

use crate::{cpu, println};
use core::panic::PanicInfo;

#[linkage = "weak"]
#[no_mangle]
fn _panic_exit() -> ! {
    #[cfg(not(feature = "test_build"))]
    {
        cpu::wait_forever()
    }

    #[cfg(feature = "test_build")]
    {
        cpu::qemu_exit_failure()
    }
}

fn panic_prevent_reenter() {
    use core::sync::atomic::{AtomicBool, Ordering};

    static PANIC_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

    if !PANIC_IN_PROGRESS.load(Ordering::Relaxed) {
        PANIC_IN_PROGRESS.store(true, Ordering::Relaxed);
        return;
    }

    _panic_exit()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    panic_prevent_reenter();

    let (location, line, col) = match info.location() {
        Some(loc) => (loc.file(), loc.line(), loc.column()),
        _ => ("?", 0, 0),
    };

    #[cfg(not(test))]
    println!(
        "Kernel Panic!\n\nPanic localtion:\n  File '{}', line {}, column {}\n\n {}",
        location,
        line,
        col,
        info.message().unwrap_or(&format_args!(""))
    );

    _panic_exit()
}
