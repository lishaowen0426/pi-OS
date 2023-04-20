#[no_mangle]
#[link_section = ".text._start_arguments"]
pub static BOOT_CORE_ID: u64 = 0;

#[no_mangle]
#[link_section = ".data"]
pub static TEST_DATA: u32 = 0;
