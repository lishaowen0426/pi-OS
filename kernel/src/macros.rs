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

#[macro_export]
macro_rules! type_enum {

    ($vis:vis enum $name:ident {
        $($variant:ident = $dis:expr ),*,
    }) => {
        #[derive(Eq, PartialEq, Copy, Clone)]
        #[repr(u8)]
        $vis enum $name {
            $($variant = $dis ),*,
            Undefined,
        }

        impl Default for $name{
            fn default() -> Self{
                Self::Undefined
            }
        }



        impl fmt::Display for $name{
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result{
                match self{
                    $($name::$variant => write!(f,"{}", stringify!($variant))?,)*
                    _ => write!(f,"{}:{}", stringify!($name), "Undefined")?,
                };
                Ok(())
            }
        }
        impl fmt::Debug for $name{
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result{
                match self{
                    $($name::$variant => write!(f,"{}", stringify!($variant))?,)*
                    _ => write!(f,"{}:{}", stringify!($name), "Undefined")?,
                };
                Ok(())
            }
        }

        impl From<u8> for $name {
            fn from(val: u8) -> Self{
                match val{
                    $(
                      $dis  => Self::$variant,

                    )*
                    _ => Self::Undefined,
                }
            }
        }
    };
}
