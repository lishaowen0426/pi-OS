// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2022 Andre Richter <andre.o.richter@gmail.com>

#![feature(asm_const)]
#![feature(format_args_nl)]
#![feature(panic_info_message)]
#![feature(custom_test_frameworks)]
#![no_main]
#![no_std]

mod bsp;
mod console;
mod cpu;
mod driver;
mod macros;
mod panic_wait;
mod print;
mod synchronization;

unsafe fn kernel_init(el: u64) -> ! {
    if let Err(x) = bsp::driver::init() {
        panic!("Error initializing BSP driver subsystem: {}", x);
    }

    driver::driver_manager().init_drivers();

    kernel_main()
}

fn kernel_main() -> ! {
    use console::console;
    println!(
        "[0] {} version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    println!("[1] Booting on: {}", bsp::board_name());

    println!("[2] Drivers loaded:");
    driver::driver_manager().enumerate();

    println!("[3] Chars written: {}", console().chars_written());
    println!("[4] Echoing input now");

    // Discard any spurious received characters before going into echo mode.
    console().clear_rx();
    loop {
        let c = console().read_char();
        println!("{}", c);
        console().write_char(c);
        console().flush();
    }
}
