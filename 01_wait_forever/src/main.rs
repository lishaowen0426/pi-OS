// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2022 Andre Richter <andre.o.richter@gmail.com>

#![feature(asm_const)]
#![no_main]
#![no_std]

mod bsp;
mod cpu;
mod panic_wait;

unsafe fn kernel_init() -> ! {
    panic!()
}
