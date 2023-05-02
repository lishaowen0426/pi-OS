#![allow(clippy::upper_case_acronyms)]
#![allow(incomplete_features)]
#![feature(asm_const)]
#![feature(const_option)]
#![feature(associated_type_defaults)]
#![feature(core_intrinsics)]
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
#![no_std]
// Testing
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(crate::test_runner)]

mod panic_wait;
mod synchronization;

pub mod bsp;
pub mod console;
pub mod cpu;
pub mod driver;
pub mod errno;
pub mod exception;
pub mod macros;
pub mod memory;
pub mod print;
pub mod utils;

#[cfg(not(test))]
/*
extern "Rust" {
    fn kernel_main() -> !;
}
*/
#[no_mangle]
unsafe fn kernel_main() -> ! {
    console::init_console();

    let (_, el) = exception::current_privilege_level();
    println!("Current privilege level: {}", el);

    memory::MMU.config_tcr_el1().unwrap();

    loop {}
}

#[cfg(test)]
fn test_runner(tests: &[&test_types::UnitTest]) {
    println_qemu!("Running {} tests", tests.len());
    for (i, test) in tests.iter().enumerate() {
        println_qemu!("{:>3}. {:.<58}", i + 1, test.name);
        (test.test_func)();

        println_qemu!("[ok]");
    }
}

#[cfg(test)]
#[no_mangle]
unsafe fn kernel_main() -> ! {
    // exception::handling_init();
    // bsp::driver::qemu_bring_up_console();

    test_main();

    cpu::qemu_exit_success()
}
