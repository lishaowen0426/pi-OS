#[cfg(target_arch = "aarch64")]
#[path = "_arch/aarch64/mmu.rs"]
mod arch_mmu;

pub use arch_mmu::*;
