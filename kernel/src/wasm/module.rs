use crate::{
    errno::{ErrorCode, EINVAL},
    memory::heap::AllocBuffer,
    type_enum, type_enum_with_error,
};
use core::{
    fmt,
    ops::{Deref, DerefMut, Index, IndexMut},
};
use nom::error::ErrorKind;

mod parser;

trait Parseable {
    fn parse(input: &[u8]) -> Self;
}

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
    Undefined,
}

impl Default for ValType {
    fn default() -> Self {
        Self::Undefined
    }
}

#[derive(Default)]
#[repr(C)]
struct ResultType {
    values: WasmVector<ValType>,
}

#[derive(Default)]
#[repr(C)]
struct FuncType {
    input: ResultType,
    output: ResultType,
}

#[repr(C)]
struct WasmVector<T> {
    n: u32,
    elements: *mut T,
}

impl<T> Default for WasmVector<T> {
    fn default() -> Self {
        Self {
            n: 0u32,
            elements: core::ptr::null_mut(),
        }
    }
}

impl<T> WasmVector<T> {
    fn new(n: u32, elements: *mut T) -> Self {
        Self { n, elements }
    }

    fn init(&mut self, n: u32, elements: *mut T) -> Result<(), ErrorCode> {
        if n != 0 || !elements.is_null() {
            Err(EINVAL)
        } else {
            self.n = n;
            self.elements = elements;
            Ok(())
        }
    }
}

impl<T> Index<usize> for WasmVector<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        unsafe { self.elements.offset(index as isize).as_ref().unwrap() }
    }
}

impl<T> IndexMut<usize> for WasmVector<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe { self.elements.offset(index as isize).as_mut().unwrap() }
    }
}

#[derive(Default)]
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

impl<T> Default for WasmSection<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            header: SectionHeader::default(),
            cont: T::default(),
        }
    }
}

impl<T> Deref for WasmSection<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.cont
    }
}

impl<T> DerefMut for WasmSection<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cont
    }
}

impl<T> WasmSection<T>
where
    T: Default,
{
    fn new(header: SectionHeader) -> Self {
        Self {
            header,
            cont: T::default(),
        }
    }
}

type TypeSection = WasmSection<WasmVector<FuncType>>;

pub struct WasmModule {
    buffer: Option<AllocBuffer>,
    type_section: Option<TypeSection>,
}

impl<'a> WasmModule {
    pub fn new() -> Self {
        WasmModule {
            buffer: None,
            type_section: None,
        }
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
