pub mod gpio;
pub mod mini_uart;
mod utils;

#[cfg(feature = "bsp_rpi3")]
pub mod bcm_ic;
#[cfg(feature = "bsp_rpi3")]
pub use bcm_ic as interrupt_controller;

#[cfg(any(feature = "bsp_rpi4", feature = "build_chainloader"))]
pub mod gic_400;
#[cfg(any(feature = "bsp_rpi4", feature = "build_chainloader"))]
pub use gic_400 as interrupt_controller;
