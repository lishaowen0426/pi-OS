#[cfg(target_arch = "aarch64")]
#[path = "_arch/aarch64/errno.rs"]
mod arch_errno;

use crate::errno_decl;

errno_decl!(
    EINVAL => "Invalid argument",
    EAGAIN => "Try again"
);
