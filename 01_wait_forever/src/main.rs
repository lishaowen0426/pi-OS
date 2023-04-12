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
mod panic_wait;
mod print;
mod synchronization;

unsafe fn kernel_init(el: u64) -> ! {
    use console::console;
    println!("[0]Hello from Pi!");
    println!("[1] Chars written:{}", console().chars_written());
    println!("[2]EL: {}", el);
    println!("[3]Stopping here.");
    cpu::wait_forever()
}
