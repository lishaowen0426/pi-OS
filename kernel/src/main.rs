// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2022 Andre Richter <andre.o.richter@gmail.com>

#![feature(asm_const)]
#![feature(format_args_nl)]
#![feature(panic_info_message)]
#![feature(custom_test_frameworks)]
#![feature(sync_unsafe_cell)]
#![allow(dead_code)]
#![no_main]
#![no_std]

mod bsp;
mod console;
mod cpu;
mod driver;
mod exception;
mod macros;
mod panic_wait;
mod print;
mod synchronization;

unsafe fn kernel_init() -> ! {
    #[cfg(feature = "bsp_rpi3")]
    println_qemu!("I am qemu!");

    console::init_console();
    println!("");
    println!("new kernel");

    kernel_main()
}

fn kernel_main() -> ! {
    let (_, el) = exception::current_privilege_level();
    println!("Current privilege level: {}", el);

    loop {}
}
