use crate::{
    bsp::{device_driver, mmio},
    driver as generic_driver,
    memory::config,
};
use core::sync::atomic::{AtomicBool, Ordering};
