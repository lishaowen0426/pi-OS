// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2022 Andre Richter <andre.o.richter@gmail.com>

#![feature(asm_const)]
#![feature(format_args_nl)]
#![feature(panic_info_message)]
#![feature(custom_test_frameworks)]
#![allow(dead_code)]
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

use core::fmt::Write;

use bsp::device_driver::MiniUart;

unsafe fn kernel_init(el: u32) -> ! {
    #[cfg(feature = "bsp_rpi3")]
    println_qemu!("res {}", el);

    console::init_console();
    println!("I am console!");

    kernel_main()
}

fn kernel_main() -> ! {
    println!(
        "[0] {} version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    println!("[1] Booting on: {}", bsp::board_name());

    println!("[2] Drivers loaded:");
    // driver::driver_manager().enumerate();

    println!("[4] Echoing input now");

    // Discard any spurious received characters before going into echo mode.
    // unsafe {
    // console::CONSOLE.clear_rx();
    // }
    // loop {
    // unsafe {
    // let c = console::CONSOLE.read_char();
    // println!("receive: {}\n", c);
    // }
    // }

    loop {}
}
