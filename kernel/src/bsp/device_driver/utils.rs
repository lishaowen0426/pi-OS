#[inline(always)]
pub fn mmio_write(reg: usize, val: u32) {
    unsafe {
        core::ptr::write_volatile(reg as *mut u32, val);
    }
}

#[inline(always)]
pub fn mmio_read(reg: usize) -> u32 {
    unsafe { core::ptr::read_volatile(reg as *const u32) }
}

#[inline(always)]
pub fn mmio_is_set(reg: usize, nbit: u32) -> bool {
    (mmio_read(reg) & (0b1 << nbit)) != 0
}
