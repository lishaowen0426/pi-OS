// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2022 Andre Richter <andre.o.richter@gmail.com>

#![feature(asm_const)]
#![feature(format_args_nl)]
#![feature(panic_info_message)]
#![feature(custom_test_frameworks)]
#![feature(sync_unsafe_cell)]
#![feature(macro_metavar_expr)]
#![feature(error_in_core)]
#![no_main]
#![no_std]
#![allow(dead_code)]

use libkernel::{console, exception, memory, println};

#[no_mangle]
unsafe fn kernel_main() -> ! {
    console::init_console();

    let (_, el) = exception::current_privilege_level();
    println!("Current privilege level: {}", el);

    memory::MMU.config_tcr_el1().unwrap();

    loop {}
}
