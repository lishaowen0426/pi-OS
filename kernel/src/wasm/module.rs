use crate::{
    errno::{ErrorCode, EINVAL},
    memory::heap,
    type_enum, type_enum_with_error,
};
use core::{
    fmt,
    ops::{Deref, DerefMut, Index, IndexMut},
};
use nom::error::{Error, ErrorKind};
use test_macros::SingleField;

mod instructions;
mod parser;

type ParserResult<'a, O> = Result<(&'a [u8], O), Error<&'a [u8]>>;

trait Parseable
where
    Self: Sized,
{
    fn parse<'a>(input: &'a [u8], alloc: &mut Option<heap::BumpBuffer>) -> ParserResult<'a, Self>;
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
#[derive(Default)]
#[repr(C)]
struct SectionHeader {
    id: SectionType,
    size: U32,
}

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

#[derive(Clone, Copy)]
#[repr(u8)]
enum ValType {
    Num(NumType),
    Vec(VecType),
    Ref(RefType),
    Undefined,
}

impl TryFrom<u8> for ValType {
    type Error = ErrorKind;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if let Ok(n) = NumType::try_from(value) {
            return Ok(Self::Num(n));
        }

        if let Ok(v) = VecType::try_from(value) {
            return Ok(Self::Vec(v));
        }

        if let Ok(r) = RefType::try_from(value) {
            return Ok(Self::Ref(r));
        }

        return Err(ErrorKind::IsNot);
    }
}

impl Default for ValType {
    fn default() -> Self {
        Self::Undefined
    }
}

impl fmt::Display for ValType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Num(n) => write!(f, "{}", n),
            Self::Vec(v) => write!(f, "{}", v),
            Self::Ref(r) => write!(f, "{}", r),
            _ => write!(f, "Unknown value type"),
        }
    }
}

type_enum!(
    enum MutType {
        Const = 0x00,
        Var = 0x01,
    },
    ErrorKind,
    ErrorKind::IsNot
);
#[derive(Default)]
#[repr(C)]
struct ResultType {
    values: WasmVector<ValType>,
}

impl fmt::Display for ResultType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.values)
    }
}

#[derive(Default)]
#[repr(C)]
struct FuncType {
    input: ResultType,
    output: ResultType,
}

impl fmt::Display for FuncType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {}", self.input, self.output)
    }
}

#[derive(Eq, PartialEq, Default, Clone, Copy, SingleField)]
#[repr(transparent)]
struct Byte(u8);

#[derive(Eq, PartialEq, Default, Clone, Copy, SingleField)]
#[repr(transparent)]
struct U32(u32);

#[derive(Eq, PartialEq, Default, Clone, Copy, SingleField)]
#[repr(transparent)]
struct U64(u64);

#[derive(Eq, PartialEq, Default, Clone, Copy, SingleField)]
#[repr(transparent)]
struct S32(i32);

#[derive(Eq, PartialEq, Default, Clone, Copy, SingleField)]
#[repr(transparent)]
struct S64(i64);

#[derive(Eq, PartialEq, Default, Clone, Copy, SingleField)]
#[repr(transparent)]
struct I32(S32);

#[derive(Eq, PartialEq, Default, Clone, Copy, SingleField)]
#[repr(transparent)]
struct I64(S64);

// The type index in a block type
// encoded as a positive signed integer
#[derive(Eq, PartialEq, Default, Clone, Copy, SingleField)]
#[repr(transparent)]
struct S33(u32);

#[repr(C)]
struct Limits {
    min: U32,
    max: Option<U32>,
}

impl fmt::Display for Limits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(m) = self.max {
            write!(f, "{{ {}, {} }}", self.min, m)
        } else {
            write!(f, "{{ {},  }}", self.min)
        }
    }
}

#[repr(C)]
struct TableType {
    lim: Limits,
    et: RefType,
}

impl fmt::Display for TableType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.lim, self.et)
    }
}

#[repr(C)]
struct GlobalType {
    t: ValType,
    m: MutType,
}

impl fmt::Display for GlobalType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.m, self.t)
    }
}

#[repr(C)]
struct Expr {}

#[repr(C)]
struct Global {
    gt: GlobalType,
    e: Expr,
}

#[repr(C)]
struct WasmVector<T> {
    n: U32,
    elements: *mut T,
}

impl<T: fmt::Display> fmt::Display for WasmVector<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            let s = core::slice::from_raw_parts(self.elements as *const T, self.n.0 as usize);
            for i in 0..s.len() - 1 {
                write!(f, "{} ", s[i])?;
            }
            write!(f, "{}", s[s.len() - 1])
        }
    }
}
impl<T> fmt::Debug for WasmVector<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "len = {}", self.n);
        let buffer = self.elements as *const u8;
        unsafe {
            for i in 0..self.n.0 {
                write!(f, "{:#04x} ", buffer.offset(i as isize).read());
                if i % 10 == 0 {
                    writeln!(f, "");
                }
            }
        }
        writeln!(f, "")
    }
}

impl<T> Default for WasmVector<T> {
    fn default() -> Self {
        Self {
            n: 0u32.into(),
            elements: core::ptr::null_mut(),
        }
    }
}

impl<T> WasmVector<T> {
    fn new(n: U32, elements: *mut T) -> Self {
        Self { n, elements }
    }

    fn init(&mut self, n: U32, elements: *mut T) -> Result<(), ErrorCode> {
        if n != 0u32.into() || !elements.is_null() {
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

impl fmt::Display for SectionHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)?;
        write!(f, "(size={})", self.size)
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
    fn new(header: SectionHeader, cont: T) -> Self {
        Self { header, cont }
    }
}

type TypeSection = WasmSection<WasmVector<FuncType>>;
type FuncSection = WasmSection<WasmVector<U32>>;
type TableSection = WasmSection<WasmVector<TableType>>;

impl<T: fmt::Display> fmt::Display for WasmSection<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.header)?;
        write!(f, "{}", self.cont)
    }
}

pub struct WasmModule {
    buffer: Option<heap::BumpBuffer>,
    type_section: Option<TypeSection>,
    func_section: Option<FuncSection>,
    table_section: Option<TableSection>,
}

impl fmt::Display for WasmModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref s) = self.type_section {
            writeln!(f, "{}", s)?;
        }
        if let Some(ref s) = self.func_section {
            writeln!(f, "{}", s)?;
        }
        if let Some(ref s) = self.table_section {
            writeln!(f, "{}", s)?;
        }
        Ok(())
    }
}

impl<'a> WasmModule {
    pub fn new() -> Self {
        WasmModule {
            buffer: None,
            type_section: None,
            func_section: None,
            table_section: None,
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
