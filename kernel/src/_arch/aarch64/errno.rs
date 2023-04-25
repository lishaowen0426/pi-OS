#[macro_export]
macro_rules! errno_decl {
    ($($ident:ident => $literal:literal), *) => {
        pub enum SysError{
            $($ident(u8,&'static str)),*
        }

        $(pub static $ident: &'static SysError= &SysError::$ident(${index()}, $literal);)*
        impl core::fmt::Display for SysError{
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
               match self{
                   $(SysError::$ident(c,s)=>{write!(f, "{},{}",c, s)},)*
               }
           }
        }
        impl core::fmt::Debug for SysError{
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
               match self{
                   $(SysError::$ident(_,s)=>{write!(f, "{}", s)},)*
               }
           }
        }

        impl core::error::Error for SysError{}


        pub type ErrorCode = &'static SysError;

    };
}
