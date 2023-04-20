// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2022 Andre Richter <andre.o.richter@gmail.com>

//! A panic handler that infinitely waits.

use crate::{cpu, println};
use core::panic::PanicInfo;

fn panic_prevent_reenter() {
    use core::sync::atomic::{AtomicBool, Ordering};

    static PANIC_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

    if !PANIC_IN_PROGRESS.load(Ordering::Relaxed) {
        PANIC_IN_PROGRESS.store(true, Ordering::Relaxed);
        return;
    }

    cpu::wait_forever()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    panic_prevent_reenter();

    let (location, line, col) = match info.location() {
        Some(loc) => (loc.file(), loc.line(), loc.column()),
        _ => ("?", 0, 0),
    };

    println!(
        "Kernel Panic!\n\nPanic localtion:\n  File '{}', line {}, column {}\n\n {}",
        location,
        line,
        col,
        info.message().unwrap_or(&format_args!(""))
    );

    cpu::wait_forever()
}
