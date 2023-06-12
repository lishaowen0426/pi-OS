use crate::{
    bsp::{device_driver, mmio},
    driver as generic_driver,
    memory::config,
};
use core::sync::atomic::{AtomicBool, Ordering};

static GPIO: device_driver::GPIO =
    unsafe { device_driver::GPIO::new(config::VIRTUAL_PERIPHERAL_START + mmio::GPIO_OFFSET) };

/// This must be called only after successful init of the GPIO driver.
fn post_init_gpio() -> Result<(), &'static str> {
    // GPIO.map_pl011_uart();
    //    GPIO.map_mini_uart();
    Ok(())
}

fn driver_gpio() -> Result<(), &'static str> {
    let gpio_descriptor = generic_driver::DeviceDriverDescriptor::new(&GPIO, Some(post_init_gpio));
    generic_driver::driver_manager().register_driver(gpio_descriptor);

    Ok(())
}

pub unsafe fn init() -> Result<(), &'static str> {
    static INIT_DONE: AtomicBool = AtomicBool::new(false);
    if INIT_DONE.load(Ordering::Relaxed) {
        return Err("Init already done");
    }

    driver_gpio()?;

    INIT_DONE.store(true, Ordering::Relaxed);
    Ok(())
}
