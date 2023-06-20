pub mod gpio;
pub mod mini_uart;
mod utils;

#[cfg(feature = "bsp_rpi3")]
pub mod bcm_ic;
#[cfg(feature = "bsp_rpi3")]
pub use bcm_ic as interrupt_controller;

#[cfg(feature = "bsp_rpi4")]
pub mod gic_400;
#[cfg(feature = "bsp_rpi4")]
pub use gic_400 as interrupt_controller;
