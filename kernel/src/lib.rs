#![allow(clippy::upper_case_acronyms)]
#![allow(dead_code)]
#![allow(incomplete_features)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_macros)]
#![feature(asm_const)]
#![feature(const_option)]
#![feature(ptr_from_ref)]
#![feature(generic_const_exprs)]
#![feature(associated_type_defaults)]
#![feature(core_intrinsics)]
#![feature(pointer_is_aligned)]
#![feature(strict_provenance)]
#![feature(format_args_nl)]
#![feature(int_roundings)]
#![feature(linkage)]
#![feature(nonzero_min_max)]
#![feature(panic_info_message)]
#![feature(trait_alias)]
#![feature(exclusive_range_pattern)]
#![feature(unchecked_math)]
#![feature(sync_unsafe_cell)]
#![feature(error_in_core)]
#![feature(macro_metavar_expr)]
#![feature(const_pointer_is_aligned)]
#![feature(const_fmt_arguments_new)]
#![no_std]
// Testing
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(crate::test_runner)]

extern crate alloc;

mod bsp;
mod console;
mod cpu;
mod driver;
mod errno;
mod exception;
mod generics;
mod interrupt;
mod macros;
mod memory;
mod panic_wait;
mod print;
mod scheduler;
mod synchronization;
mod utils;

use aarch64_cpu::{asm, registers::*};
use core::{fmt, time::Duration};
use generics::*;
use interrupt::IRQ_CONTROLLER;
use memory::address::*;
use tock_registers::interfaces::Writeable;
// 32 bytes * 4 + 16 + 16 + 16
#[derive(Copy, Clone)]
#[repr(C)]
pub struct BootInfo {
    pub code_and_ro: Mapped,
    pub bss: Mapped,
    pub stack: Mapped,
    pub peripheral: Mapped,
    pub free_frame: PaRange,
    pub lower_free_page: VaRange,
    pub higher_free_page: VaRange,
}

impl fmt::Display for BootInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "    code and ro:       {}", self.code_and_ro)?;
        writeln!(f, "    bss :              {}", self.bss)?;
        writeln!(f, "    stack:             {}", self.stack)?;
        writeln!(f, "    peripheral:        {}", self.peripheral)?;
        writeln!(f, "    free frame:        {}", self.free_frame)?;
        writeln!(f, "    lower free page:   {}", self.lower_free_page)?;
        write!(f, "    higher free page:  {}", self.higher_free_page)
    }
}

impl fmt::Debug for BootInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "    code and ro:       {:?}", self.code_and_ro)?;
        writeln!(f, "    bss :              {:?}", self.bss)?;
        writeln!(f, "    stack:             {:?}", self.stack)?;
        writeln!(f, "    peripheral:        {:?}", self.peripheral)?;
        writeln!(f, "    free frame:        {:?}", self.free_frame)?;
        writeln!(f, "    lower free page:   {:?}", self.lower_free_page)?;
        write!(f, "    higher free page:  {:?}", self.higher_free_page)
    }
}

#[cfg(not(test))]
#[no_mangle]
pub unsafe fn kernel_main(boot_info: &BootInfo) -> ! {
    console::init().unwrap();
    exception::init().unwrap();
    println!("Boot info:");
    println!("{}", boot_info);
    memory::init(boot_info).unwrap();
    cpu::timer::init().unwrap();
    // cpu::timer::TIMER.get().unwrap().enable();

    println!(
        "System counter frequency {}",
        cpu::timer::system_counter_frequency()
    );

    let boot_duration = cpu::timer::TIMER.get().unwrap().now();
    println!("boot takes {} micros", boot_duration.as_micros());

    interrupt::init().unwrap();
    cpu::timer::TIMER.get().unwrap().enable();
    scheduler::init();

    println!("Passed!");

    loop {}
}

#[cfg(test)]
fn test_runner(tests: &[&test_types::UnitTest]) {
    println!("Running {} tests", tests.len());
    for (i, test) in tests.iter().enumerate() {
        println!("{:>3}. {:.<58}", i + 1, test.name);
        (test.test_func)();

        println!("[ok]");
    }
}

#[cfg(test)]
#[no_mangle]
pub unsafe fn kernel_main(boot_info: &BootInfo) -> ! {
    // exception::handling_init();
    // bsp::driver::qemu_bring_up_console();
    println!(
        "current exception level {}",
        exception::current_privilege_level().1
    );
    println!("Boot info:");
    println!("{}", boot_info);
    memory::init(boot_info).unwrap();
    cpu::timer::init().unwrap();

    println!(
        "System counter frequency {}",
        cpu::timer::system_counter_frequency()
    );

    let boot_duration = cpu::timer::TIMER.get().unwrap().now();
    println!("boot takes {} micros", boot_duration.as_micros());

    interrupt::init().unwrap();

    test_main();

    cpu::qemu_exit_success()
}
