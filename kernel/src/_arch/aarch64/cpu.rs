use aarch64_cpu::asm;
pub use asm::nop;

#[path = "cpu/registers.rs"]
pub mod registers;

#[cfg(feature = "build_qemu")]
#[inline(always)]
pub fn spin_for_cycles(n: usize) {
    for _ in 0..n {
        asm::nop();
    }
}

#[inline(always)]
pub fn wait_forever() -> ! {
    loop {
        asm::wfe()
    }
}

//--------------------------------------------------------------------------------------------------
// Testing
//--------------------------------------------------------------------------------------------------
#[cfg(feature = "build_qemu")]
use qemu_exit::QEMUExit;

#[cfg(feature = "build_qemu")]
const QEMU_EXIT_HANDLE: qemu_exit::AArch64 = qemu_exit::AArch64::new();

/// Make the host QEMU binary execute `exit(1)`.
#[cfg(feature = "build_qemu")]
pub fn qemu_exit_failure() -> ! {
    QEMU_EXIT_HANDLE.exit_failure()
}

/// Make the host QEMU binary execute `exit(0)`.
#[cfg(feature = "build_qemu")]
pub fn qemu_exit_success() -> ! {
    QEMU_EXIT_HANDLE.exit_success()
}
