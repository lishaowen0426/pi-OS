#[macro_export]
macro_rules! rpi4 {
    ($($i:item)*) => {
        cfg_if::cfg_if! {
            if #[cfg(feature = "bsp_rpi4")]{
                $($i)*
            }else{
                compile_error!("RPI 3 is not supported");
            }
        }
    };
}

#[macro_export]
macro_rules! rpi3 {
    ($($i:item)*) => {
        cfg_if::cfg_if! {
            if #[cfg(feature = "bsp_rpi3")]{
                $($i)*
            }else{
                compile_error!("RPI 4 is not supported");
            }
        }
    };
}
