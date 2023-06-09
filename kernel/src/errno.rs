macro_rules! errno_decl {
    ($($ident:ident => $literal:literal), * $(,)?) => {
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

errno_decl!(
    EINVAL => "Invalid operation",
    EAGAIN => "Try again",
    EALIGN => "Address is not properly aligned",
    EOVERFLOW => "Overflow",
    ETYPE => "Wrong Type",
    EBOUND => "Out of bound",
    EFRAME => "Cannot allocate frame",
    EPAGE => "Cannot allocate page",
    EINIT  => "Not initialized properly",
    EPARAM => "Invalid parameter",
    ESUPPORTED => "Not supported",
    ESCHED => "Scheduler error",
    EUNKNOWN => "Unknown reason",
    EUNMAP => "Address is not mapped",
);
