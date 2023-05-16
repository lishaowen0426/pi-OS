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

// Copy from aarch64_cpu
// DONT TOUCH
#[macro_export]
macro_rules! __read_raw {
    ($width:ty, $asm_instr:tt, $asm_reg_name:tt, $asm_width:tt) => {
        /// Reads the raw bits of the CPU register.
        #[inline]
        fn get(&self) -> $width {
            match () {
                #[cfg(target_arch = "aarch64")]
                () => {
                    let reg;
                    unsafe {
                        core::arch::asm!(concat!($asm_instr, " {reg:", $asm_width, "}, ", $asm_reg_name), reg = out(reg) reg, options(nomem, nostack));
                    }
                    reg
                }

                #[cfg(not(target_arch = "aarch64"))]
                () => unimplemented!(),
            }
        }
    };
}

#[macro_export]
macro_rules! __write_raw {
    ($width:ty, $asm_instr:tt, $asm_reg_name:tt, $asm_width:tt) => {
        /// Writes raw bits to the CPU register.
        #[cfg_attr(not(target_arch = "aarch64"), allow(unused_variables))]
        #[inline]
        fn set(&self, value: $width) {
            match () {
                #[cfg(target_arch = "aarch64")]
                () => {
                    unsafe {
                        core::arch::asm!(concat!($asm_instr, " ", $asm_reg_name, ", {reg:", $asm_width, "}"), reg = in(reg) value, options(nomem, nostack))
                    }
                }

                #[cfg(not(target_arch = "aarch64"))]
                () => unimplemented!(),
            }
        }
    };
}

/// Raw read from system coprocessor registers.
#[macro_export]
macro_rules! sys_coproc_read_raw {
    ($width:ty, $asm_reg_name:tt, $asm_width:tt) => {
        __read_raw!($width, "mrs", $asm_reg_name, $asm_width);
    };
}

/// Raw write to system coprocessor registers.
#[macro_export]
macro_rules! sys_coproc_write_raw {
    ($width:ty, $asm_reg_name:tt, $asm_width:tt) => {
        __write_raw!($width, "msr", $asm_reg_name, $asm_width);
    };
}

/// Raw read from (ordinary) registers.
macro_rules! read_raw {
    ($width:ty, $asm_reg_name:tt, $asm_width:tt) => {
        __read_raw!($width, "mov", $asm_reg_name, $asm_width);
    };
}
/// Raw write to (ordinary) registers.
macro_rules! write_raw {
    ($width:ty, $asm_reg_name:tt, $asm_width:tt) => {
        __write_raw!($width, "mov", $asm_reg_name, $asm_width);
    };
}
