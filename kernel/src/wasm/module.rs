use crate::{errno::ErrorCode, type_enum, type_enum_with_error};
use core::{fmt, marker::PhantomData};
use nom::error::ErrorKind;

mod parser;

type_enum!(
    enum SectionType {
        Custom = 0,
        Type = 1,
        Import = 2,
        Function = 3,
        Table = 4,
        Memory = 5,
        Global = 6,
        Export = 7,
        Start = 8,
        Element = 9,
        Code = 10,
        Data = 11,
        DataCount = 12,
    },
    ErrorKind,
    ErrorKind::IsNot
);

type_enum!(
    enum NumType {
        F64 = 0x7C,
        F32 = 0x7D,
        I64 = 0x7E,
        I32 = 0x7F,
    },
    ErrorKind,
    ErrorKind::IsNot
);

type_enum!(
    enum VecType {
        Vec = 0x7B,
    },
    ErrorKind,
    ErrorKind::IsNot
);

type_enum!(
    enum RefType {
        ExternRef = 0x6F,
        FuncRef = 0x70,
    },
    ErrorKind,
    ErrorKind::IsNot
);

#[repr(u8)]
enum ValType {
    Num(NumType),
    Vec(VecType),
    Ref(RefType),
}

#[repr(C)]
struct ResultType {
    values: WasmVector<ValType>,
}

#[repr(C)]
struct FuncType {
    tag: u8,
    input: ResultType,
    output: ResultType,
}

#[repr(C)]
struct WasmVector<T> {
    n: u32,
    elements: *const T,
}

#[repr(C)]
struct SectionHeader {
    id: SectionType,
    size: u32,
}

impl fmt::Display for SectionHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)?;
        write!(f, "(size={:#010x})", self.size)
    }
}

#[repr(C)]
struct WasmSection<T> {
    header: SectionHeader,
    cont: T,
}

pub struct WasmModule {
    type_section: Option<WasmSection<WasmVector<FuncType>>>,
}

impl<'a> WasmModule {
    pub fn new() -> Self {
        WasmModule { type_section: None }
    }
}

impl<'a> WasmModule {
    pub fn parse() -> Result<(), ErrorCode> {
        todo!()
    }
}

#[cfg(test)]
#[allow(unused_imports, unused_variables, dead_code)]
mod tests {
    use super::*;
    use crate::println;
    use test_macros::kernel_test;
    #[kernel_test]
    fn test_module() {}
}
