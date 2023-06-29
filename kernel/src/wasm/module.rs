extern crate alloc;
use crate::{
    errno::{ErrorCode, EINVAL},
    memory::heap,
    type_enum, type_enum_with_error,
};
use alloc::vec::Vec;
use core::{
    fmt,
    iter::Iterator,
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
        write!(f, "(")?;
        for t in self.values.iter() {
            write!(f, "{},", t)?;
        }
        write!(f, ")")
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
        write!(f, "Table{{lim: {}, et: {}}}", self.lim, self.et)
    }
}

#[repr(C)]
struct GlobalType {
    t: ValType,
    m: MutType,
}

impl fmt::Display for GlobalType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "global {} {}", self.m, self.t)
    }
}

#[repr(C)]
struct MemType {
    lim: Limits,
}

impl fmt::Display for MemType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "mem {}", self.lim)
    }
}

#[repr(C)]
struct Name {
    b: WasmVector<Byte>,
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            let s = core::slice::from_raw_parts(self.b.elements as *const u8, self.b.n.0 as usize);
            write!(f, "{}", core::str::from_utf8_unchecked(s))
        }
    }
}

#[repr(C)]
enum ImportDesc {
    Func(U32),
    Table(TableType),
    Mem(MemType),
    Global(GlobalType),
}

impl fmt::Display for ImportDesc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Func(ref idx) => write!(f, "func {}", idx.0),
            Self::Table(ref t) => write!(f, "{}", t),
            Self::Mem(ref m) => write!(f, "{}", m),
            Self::Global(ref g) => write!(f, "{}", g),
        }
    }
}

#[repr(C)]
struct Import {
    module: Name,
    nm: Name,
    desc: ImportDesc,
}

impl fmt::Display for Import {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "module: {}, name: {}, desc: {}",
            self.module, self.nm, self.desc
        )
    }
}

#[repr(C)]
enum ExportDesc {
    Func(U32),
    Table(U32),
    Mem(U32),
    Global(U32),
}

impl fmt::Display for ExportDesc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Func(ref idx) => write!(f, "func {}", idx.0),
            Self::Table(ref t) => write!(f, "table {}", t),
            Self::Mem(ref m) => write!(f, "mem {}", m),
            Self::Global(ref g) => write!(f, "global {}", g),
        }
    }
}
#[repr(C)]
struct Export {
    nm: Name,
    desc: ExportDesc,
}

impl fmt::Display for Export {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "name: {}, desc: {},", self.nm, self.desc)
    }
}

#[repr(C)]
struct Expr {}

#[repr(C)]
struct Global {
    gt: GlobalType,
    e: Expr,
}

struct WasmVectorIter<'a, T> {
    v: &'a WasmVector<T>,
    next: usize,
}

impl<'a, T> Iterator for WasmVectorIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.next >= self.v.n.0 as usize {
            None
        } else {
            self.next += 1;
            Some(&self.v[self.next - 1])
        }
    }
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
            for e in s.iter() {
                writeln!(f, "    {} ", e)?;
            }
        }
        Ok(())
    }
}
impl<T> fmt::Debug for WasmVector<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "len = {}", self.n)?;
        let buffer = self.elements as *const u8;
        unsafe {
            for i in 0..self.n.0 {
                write!(f, "{:#04x} ", buffer.offset(i as isize).read());
                if i % 10 == 0 {
                    writeln!(f, "")?;
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

    fn iter(&self) -> WasmVectorIter<T> {
        WasmVectorIter { v: self, next: 0 }
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
impl<T> Index<U32> for WasmVector<T> {
    type Output = T;
    fn index(&self, index: U32) -> &Self::Output {
        unsafe { self.elements.offset(index.0 as isize).as_ref().unwrap() }
    }
}

impl<T> IndexMut<U32> for WasmVector<T> {
    fn index_mut(&mut self, index: U32) -> &mut Self::Output {
        unsafe { self.elements.offset(index.0 as isize).as_mut().unwrap() }
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
type ImportSection = WasmSection<WasmVector<Import>>;
type ExportSection = WasmSection<WasmVector<Export>>;

impl<T: fmt::Display> fmt::Display for WasmSection<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.header)?;
        write!(f, "{}", self.cont)
    }
}

struct GlobalStore {}

pub struct WasmModule {
    buffer: Option<heap::BumpBuffer>,
    type_section: Option<TypeSection>,
    func_section: Option<FuncSection>,
    table_section: Option<TableSection>,
    import_section: Option<ImportSection>,
    export_section: Option<ExportSection>,
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
        if let Some(ref s) = self.import_section {
            writeln!(f, "{}", s)?;
        }
        if let Some(ref s) = self.export_section {
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
            import_section: None,
            export_section: None,
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

    struct F {
        b: [u8; 40],
    }

    impl Default for F {
        fn default() -> Self {
            Self { b: [1; 40] }
        }
    }
    #[kernel_test]
    fn test_vector() {
        const LEN: usize = 50;
        let mut vec = Vec::new();
        for i in 0..LEN {
            println!("i = {}", i);
            vec.push(F::default());
        }
    }
}
