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

const PERIPHERAL_BASE: usize = 0xFE000000;
const GPFSEL0: usize = PERIPHERAL_BASE + 0x200000;
const GPSET0: usize = PERIPHERAL_BASE + 0x20001C;
const GPCLR0: usize = PERIPHERAL_BASE + 0x200028;
const GPPUPPDN0: usize = PERIPHERAL_BASE + 0x2000E;
const GPIO_MAX_PIN: u32 = 53;
const GPIO_FUNCTION_ALT5: u32 = 2;
const PULL_NONE: u32 = 0;
const AUX_BASE: usize = PERIPHERAL_BASE + 0x215000;
const AUX_ENABLES: usize = AUX_BASE + 4;
const AUX_MU_IO_REG: usize = AUX_BASE + 64;
const AUX_MU_IER_REG: usize = AUX_BASE + 68;
const AUX_MU_IIR_REG: usize = AUX_BASE + 72;
const AUX_MU_LCR_REG: usize = AUX_BASE + 76;
const AUX_MU_MCR_REG: usize = AUX_BASE + 80;
const AUX_MU_LSR_REG: usize = AUX_BASE + 84;
const AUX_MU_CNTL_REG: usize = AUX_BASE + 96;
const AUX_MU_BAUD_REG: usize = AUX_BASE + 104;

fn mmio_write(reg: usize, val: u32) {
    unsafe {
        core::ptr::write_volatile(reg as *mut u32, val);
    }
}

fn mmio_read(reg: usize) -> u32 {
    unsafe { core::ptr::read_volatile(reg as *const u32) }
}

fn gpio_call(pin_number: u32, value: u32, base: usize, field_size: u32, field_max: u32) -> u32 {
    let field_mask = (1 << field_size) - 1;

    if pin_number > field_max {
        return 0;
    }
    if value > field_mask {
        return 0;
    }

    let num_fields = 32 / field_size;
    let reg = base + ((pin_number / num_fields) * 4) as usize;
    let shift = (pin_number % num_fields) * field_size;

    let mut curval = mmio_read(reg as usize);
    curval &= !(field_mask << shift);
    curval |= value << shift;
    mmio_write(reg as usize, curval);

    return 1;
}

fn gpio_set(pin_number: u32, value: u32) -> u32 {
    return gpio_call(pin_number, value, GPSET0, 1, GPIO_MAX_PIN);
}
fn gpio_clear(pin_number: u32, value: u32) -> u32 {
    return gpio_call(pin_number, value, GPCLR0, 1, GPIO_MAX_PIN);
}
fn gpio_pull(pin_number: u32, value: u32) -> u32 {
    return gpio_call(pin_number, value, GPPUPPDN0, 2, GPIO_MAX_PIN);
}
fn gpio_function(pin_number: u32, value: u32) -> u32 {
    return gpio_call(pin_number, value, GPFSEL0, 3, GPIO_MAX_PIN);
}

fn gpio_useAsAlt5(pin_number: u32) {
    gpio_pull(pin_number, PULL_NONE);
    gpio_function(pin_number, GPIO_FUNCTION_ALT5);
}

fn aux_mu_baud(baud: u32, clock: u64) -> u32 {
    ((clock / (baud * 8) as u64) - 1) as u32
}
fn uart_isWriteByteReady() -> u32 {
    return mmio_read(AUX_MU_LSR_REG) & 0x20;
}
fn uart_writeByteBlockingActual(ch: char) {
    while uart_isWriteByteReady() == 0 {}
    mmio_write(AUX_MU_IO_REG, ch as u32);
}

fn uart_writeText(s: &str) {
    for c in s.chars() {
        if c == '\n' {
            uart_writeByteBlockingActual('\r');
        }
        uart_writeByteBlockingActual(c);
    }
}

unsafe fn kernel_init(el: u64) -> ! {
    // if let Err(x) = bsp::driver::init() {
    // panic!("Error initializing BSP driver subsystem: {}", x);
    // }
    //
    // driver::driver_manager().init_drivers();
    mmio_write(AUX_ENABLES, 1); // enable UART1
    mmio_write(AUX_MU_IER_REG, 0);
    mmio_write(AUX_MU_CNTL_REG, 0);
    mmio_write(AUX_MU_LCR_REG, 3); // 8 bits
    mmio_write(AUX_MU_MCR_REG, 0);
    mmio_write(AUX_MU_IER_REG, 0);
    mmio_write(AUX_MU_IIR_REG, 0xC6); // disable interrupts
    mmio_write(AUX_MU_BAUD_REG, aux_mu_baud(115200, 500000000));
    //    gpio_useAsAlt5(14);
    //  gpio_useAsAlt5(15);
    mmio_write(AUX_MU_CNTL_REG, 3); // enable RX/TX
                                    //
                                    //
    uart_writeText("hello world\n");
    uart_writeText("what??\n");

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
