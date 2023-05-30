#![allow(clippy::upper_case_acronyms)]
#![allow(dead_code)]
#![allow(incomplete_features)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_macros)]
#![feature(asm_const)]
#![feature(const_option)]
#![feature(ptr_from_ref)]
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
#![feature(unchecked_math)]
#![feature(sync_unsafe_cell)]
#![feature(error_in_core)]
#![feature(macro_metavar_expr)]
#![feature(const_pointer_is_aligned)]
#![no_std]
// Testing
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(crate::test_runner)]

mod boot_const;
mod bsp;
mod console;
mod cpu;
mod driver;
mod errno;
mod exception;
mod macros;
mod memory;
mod panic_wait;
mod print;
mod synchronization;
mod utils;
extern "C" {
    static __code_start: u8;
    static __code_end_exclusive: u8;
    static __bss_start: u8;
    static __bss_end_exclusive: u8;
    static __data_start: u8;
    static __data_end_exclusive: u8;
    static __kernel_main: u8;
    static initial_stack_top: u8;

}

use aarch64_cpu::registers::*;
use boot_const::*;
use core::{arch::asm, fmt};
use cpu::registers::*;
use memory::*;
use tock_registers::interfaces::{ReadWriteable, Readable};

// 32 bytes * 4 + 16 + 16 + 16
#[repr(C)]
pub struct BootInfo {
    code_and_ro: Mapped,
    bss: Mapped,
    stack: Mapped,
    peripheral: Mapped,

    free_frame: PaRange,
    lower_free_page: VaRange,
    higher_free_page: VaRange,
}

impl fmt::Display for BootInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "code and ro: {}", self.code_and_ro)?;
        writeln!(f, "bss : {}", self.bss)?;
        writeln!(f, "stack: {}", self.stack)?;
        writeln!(f, "peripheral: {}", self.peripheral)?;
        writeln!(f, "free frame: {}", self.free_frame)?;
        writeln!(f, "lower free page: {}", self.lower_free_page)?;
        write!(f, "higher free page: {}", self.higher_free_page)
    }
}

impl fmt::Debug for BootInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "code and ro: {:?}", self.code_and_ro)?;
        writeln!(f, "bss : {:?}", self.bss)?;
        writeln!(f, "stack: {:?}", self.stack)?;
        writeln!(f, "peripheral: {:?}", self.peripheral)?;
        writeln!(f, "free frame: {:?}", self.free_frame)?;
        writeln!(f, "lower free page: {:?}", self.lower_free_page)?;
        write!(f, "higher free page: {:?}", self.higher_free_page)
    }
}

#[cfg(not(test))]
#[no_mangle]
pub unsafe fn kernel_main(boot_info: &BootInfo) -> ! {
    // pub unsafe fn kernel_main(x0: u64) -> ! {
    exception::init().unwrap();
    console::init().unwrap();
    memory::init().unwrap();
    println!(" bootinf:\n{}", boot_info);
    // println!("\nx0 = {:#018x}", x0);
    println!(" l1 = {:#018x}", config::LOWER_L1_VIRTUAL_ADDRESS);
    let l1_va = config::LOWER_L1_VIRTUAL_ADDRESS as *mut u64;
    unsafe {
        println!(" l1[0] = {:#066b}", *l1_va.offset(0));
        //*l1_va = 0;
        // println!(" l1[0] = {:#066b}", *l1.offset(0));
        // println!("l1[511] = {:#066b}", *l1.offset(511));
    }
    let (_, el) = exception::current_privilege_level();
    println!("Current privilege level: {}", el);

    println!("Trying to trigger an exception..");
    unsafe {
        core::ptr::write_volatile(
            ((config::KERNEL_OFFSET) | (510 << 30) | (495 << 21) | (509 << 12)) as *mut u8,
            42,
        );
        // core::ptr::write_volatile(0x200000 as *mut u8, 42);
        // core::ptr::write_volatile(0x40000000 as *mut u8, 42);
    }

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
pub unsafe fn kernel_main() -> ! {
    // exception::handling_init();
    // bsp::driver::qemu_bring_up_console();

    test_main();

    cpu::qemu_exit_success()
}
