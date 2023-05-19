#![allow(clippy::upper_case_acronyms)]
#![allow(dead_code)]
#![allow(incomplete_features)]
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
use aarch64_cpu::registers::*;
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

extern "C" {
    static __boot_core_stack_end_exclusive: u8;
}

#[cfg(not(test))]
#[no_mangle]
unsafe fn kernel_main() -> ! {
    #[cfg(not(feature = "build_qemu"))]
    console::init_console();

    println!("SPSel = {}", SPSel.read(SPSel::SP));

    let (_, el) = exception::current_privilege_level();
    println!("Current privilege level: {}", el);

    let exception_handler = exception::ExceptionHandler::new();
    exception_handler.init().unwrap();

    if ID_AA64MMFR2_EL1.read(ID_AA64MMFR2_EL1::CnP) == 1 {
        println!("CnP is supported");
        TTBR0_EL1.modify(TTBR0_EL1::CnP.val(1));
    } else {
        println!("CnP is not supported");
    }

    let mmu = memory::MemoryManagementUnit::new();
    mmu.init().unwrap();
    println!("Working!");
    println!("Trying to trigger an exception..");
    let mut big_addr: u64 = 8 * 1024 * 1024 * 1024;
    unsafe {
        core::ptr::read_volatile(big_addr as *mut u64);
    }

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
unsafe fn kernel_main() -> ! {
    // exception::handling_init();
    // bsp::driver::qemu_bring_up_console();

    test_main();

    cpu::qemu_exit_success()
}
