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
mod macros;
mod panic_wait;
mod print;
mod synchronization;

unsafe fn kernel_init() -> ! {
    #[cfg(feature = "bsp_rpi3")]
    println_qemu!("I am qemu!");

    console::init_console();
    println!("");
    println!("console initialized");

    kernel_main()
}

const MINILOAD_LOGO: &str = r#"
 __  __ _      _ _                 _
|  \/  (_)_ _ (_) |   ___  __ _ __| |
| |\/| | | ' \| | |__/ _ \/ _` / _` |
|_|  |_|_|_||_|_|____\___/\__,_\__,_|
"#;

fn kernel_main() -> ! {
    use console::console;
    println!("{}", MINILOAD_LOGO);

    println!("Requesting binary");

    console()._flush();
    console()._clear_rx();

    for i in 0..3 {
        // print!("{}", 3);
        let _ = console()._write_char('4');
    }
    let mut size: u32 = u32::from(console()._read_char() as u8);
    size |= u32::from(console()._read_char() as u8) << 8;
    size |= u32::from(console()._read_char() as u8) << 16;
    size |= u32::from(console()._read_char() as u8) << 24;

    let _ = console()._write_char('O');
    let _ = console()._write_char('K');

    let kernel_addr: *mut u8 = bsp::board_default_load_addr() as *mut u8;
    unsafe {
        // Read the kernel byte by byte.
        for i in 0..size {
            core::ptr::write_volatile(kernel_addr.offset(i as isize), console()._read_char() as u8)
        }
    }

    println!("[ML] Loaded! Executing the payload now\n");
    println!("kernel size {}", size);
    console()._flush();

    let kernel: fn() -> ! = unsafe { core::mem::transmute(kernel_addr) };

    kernel()
}
