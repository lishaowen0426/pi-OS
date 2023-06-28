use super::{instructions::*, *};
use crate::{errno::ErrorCode, memory::heap, println};
use core::{fmt, ops::RemAssign};
use nom::{
    bytes::complete::*,
    error::{Error, ErrorKind},
    Finish, IResult,
};

static MAGIC_AND_VERSION: [u8; 8] = [0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];
const ORDERED_SECTION: [SectionType; 12] = [
    SectionType::Type,
    SectionType::Import,
    SectionType::Function,
    SectionType::Table,
    SectionType::Memory,
    SectionType::Global,
    SectionType::Export,
    SectionType::Start,
    SectionType::Element,
    SectionType::DataCount,
    SectionType::Code,
    SectionType::Data,
];

const fn section_type_to_order_array() -> [u8; 13] {
    let mut o: [u8; 13] = [0; 13];
    o[SectionType::Custom as usize] = u8::MAX;
    o[SectionType::Type as usize] = 0;
    o[SectionType::Import as usize] = 1;
    o[SectionType::Function as usize] = 2;
    o[SectionType::Table as usize] = 3;
    o[SectionType::Memory as usize] = 4;
    o[SectionType::Global as usize] = 5;
    o[SectionType::Export as usize] = 6;
    o[SectionType::Start as usize] = 7;
    o[SectionType::Element as usize] = 8;
    o[SectionType::Code as usize] = 10;
    o[SectionType::Data as usize] = 11;
    o[SectionType::DataCount as usize] = 9;
    o
}

static SECTION_TYPE_TO_ORDER: [u8; 13] = section_type_to_order_array();

fn take_signed(N: usize, input: &[u8]) -> ParserResult<'_, i64> {
    let (remaining, lower_bytes) = take_till(|b: u8| (b & 0b10000000) == 0)(input).finish()?;
    let total_length = lower_bytes.len() + 1;
    if total_length > N.div_ceil(7usize) {
        Err(Error::new(remaining, ErrorKind::IsNot))
    } else {
        let (remaining, top_byte) = take(1usize)(remaining).finish()?;
        let mut uN: i64 = 0;
        let mut shift = 0usize;
        let bytes = lower_bytes.iter().chain(top_byte.iter());

        let low_bits = |b: &u8| (*b & 0b01111111u8) as u64;
        let high_bit = |b: &u8| *b >> 7;
        let sign_bit = |b: &u8| (*b >> 6) & 0b1;
        let mut last_byte = 0u8;
        for b in bytes {
            uN |= (low_bits(b) as i64) << shift;
            shift += 7;

            if high_bit(b) == 0 {
                last_byte = *b;
                break;
            }
        }

        if shift < N && sign_bit(&last_byte) == 1 {
            uN |= !0 << shift;
        }

        Ok((remaining, uN))
    }
}

fn take_unsigned(N: usize, input: &[u8]) -> ParserResult<'_, u64> {
    let (remaining, lower_bytes) = take_till(|b: u8| (b & 0b10000000) == 0)(input).finish()?;
    let total_length = lower_bytes.len() + 1;
    if total_length > N.div_ceil(7usize) {
        println!("unsigned {}", N);
        Err(Error::new(remaining, ErrorKind::IsNot))
    } else {
        let (remaining, top_byte) = take(1usize)(remaining).finish()?;
        let mut uN: u64 = 0;
        let top_byte = top_byte[0];
        for (idx, b) in lower_bytes.iter().enumerate() {
            let to_or = ((*b & 0b01111111u8) as u64) << (idx * 7);
            uN = uN | to_or;
        }
        uN = uN | (top_byte << (lower_bytes.len() * 7)) as u64;
        Ok((remaining, uN))
    }
}
impl Parseable for U32 {
    fn parse<'a>(input: &'a [u8], alloc: &mut Option<heap::BumpBuffer>) -> ParserResult<'a, Self> {
        take_unsigned(32usize, input).map(|(i, u)| (i, Self(u as u32)))
    }
}
impl Parseable for U64 {
    fn parse<'a>(input: &'a [u8], alloc: &mut Option<heap::BumpBuffer>) -> ParserResult<'a, Self> {
        take_unsigned(64usize, input).map(|(i, u)| (i, Self(u as u64)))
    }
}

impl Parseable for S32 {
    fn parse<'a>(input: &'a [u8], alloc: &mut Option<heap::BumpBuffer>) -> ParserResult<'a, Self> {
        take_signed(32usize, input).map(|(i, s)| (i, Self(s as i32)))
    }
}
impl Parseable for S64 {
    fn parse<'a>(input: &'a [u8], alloc: &mut Option<heap::BumpBuffer>) -> ParserResult<'a, Self> {
        take_signed(64usize, input).map(|(i, s)| (i, Self(s as i64)))
    }
}

impl Parseable for S33 {
    fn parse<'a>(input: &'a [u8], alloc: &mut Option<heap::BumpBuffer>) -> ParserResult<'a, Self> {
        let (remaining, s) = take_signed(33, input)?;
        if s.is_negative() {
            Err(Error::new(remaining, ErrorKind::Fail))
        } else {
            Ok((remaining, S33(s as u32)))
        }
    }
}

impl Parseable for SectionType {
    fn parse<'a>(input: &'a [u8], alloc: &mut Option<heap::BumpBuffer>) -> ParserResult<'a, Self> {
        let (remaining, output) = take(1usize)(input).finish()?;
        let section_type = SectionType::try_from(output[0])
            .map_err(|error_kind| Error::new(remaining, error_kind))?;
        Ok((remaining, section_type))
    }
}

impl Parseable for SectionHeader {
    fn parse<'a>(input: &'a [u8], alloc: &mut Option<heap::BumpBuffer>) -> ParserResult<'a, Self> {
        let (remaining, section_type) = SectionType::parse(input, alloc)?;
        let (remaining, section_size) = U32::parse(remaining, alloc)?;
        Ok((
            remaining,
            Self {
                id: section_type,
                size: section_size,
            },
        ))
    }
}

impl Parseable for MutType {
    fn parse<'a>(input: &'a [u8], alloc: &mut Option<heap::BumpBuffer>) -> ParserResult<'a, Self> {
        let (remaining, b) = take(1usize)(input).finish()?;
        let b = b[0];

        if let Ok(m) = MutType::try_from(b) {
            Ok((remaining, m))
        } else {
            Err(Error::new(remaining, ErrorKind::IsNot))
        }
    }
}

impl Parseable for GlobalType {
    fn parse<'a>(input: &'a [u8], alloc: &mut Option<heap::BumpBuffer>) -> ParserResult<'a, Self> {
        let (remaining, t) = ValType::parse(input, alloc)?;
        let (remaining, m) = MutType::parse(remaining, alloc)?;

        Ok((remaining, GlobalType { t, m }))
    }
}

impl<T: Parseable + fmt::Display> Parseable for WasmVector<T> {
    fn parse<'a>(input: &'a [u8], alloc: &mut Option<heap::BumpBuffer>) -> ParserResult<'a, Self> {
        let (mut remaining, vec_len) = U32::parse(input, alloc)?;
        if let Some(a) = alloc.as_mut() {
            let buffer: *mut T =
                a.alloc_n(vec_len.0 as usize * core::mem::size_of::<T>())
                    .map_err(|_| Error::new(remaining, ErrorKind::Fail))? as *mut T;
            unsafe {
                for i in 0..vec_len.0 {
                    let (r, t) = T::parse(remaining, alloc)?;
                    remaining = r;

                    buffer.offset(i as isize).write(t);
                }
                Ok((remaining, Self::new(vec_len, buffer)))
            }
        } else {
            Err(Error::new(remaining, ErrorKind::Fail))
        }
    }
}

impl Parseable for ValType {
    fn parse<'a>(input: &'a [u8], alloc: &mut Option<heap::BumpBuffer>) -> ParserResult<'a, Self> {
        let (remaining, b) = take(1usize)(input).finish()?;
        let b = b[0];

        if let Ok(n) = NumType::try_from(b) {
            Ok((remaining, ValType::Num(n)))
        } else if let Ok(v) = VecType::try_from(b) {
            Ok((remaining, ValType::Vec(v)))
        } else if let Ok(r) = RefType::try_from(b) {
            Ok((remaining, ValType::Ref(r)))
        } else {
            Err(Error::new(remaining, ErrorKind::IsNot))
        }
    }
}

impl Parseable for ResultType {
    fn parse<'a>(input: &'a [u8], alloc: &mut Option<heap::BumpBuffer>) -> ParserResult<'a, Self> {
        let (remaining, values) = WasmVector::<ValType>::parse(input, alloc)?;
        Ok((remaining, ResultType { values }))
    }
}

impl Parseable for FuncType {
    fn parse<'a>(input: &'a [u8], alloc: &mut Option<heap::BumpBuffer>) -> ParserResult<'a, Self> {
        let (remaining, tag) = take(1usize)(input).finish()?;
        if tag[0] != 0x60u8 {
            return Err(Error::new(remaining, ErrorKind::IsNot));
        }
        let (remaining, input) = ResultType::parse(remaining, alloc)?;
        let (remaining, output) = ResultType::parse(remaining, alloc)?;
        Ok((remaining, FuncType { input, output }))
    }
}

impl Parseable for Limits {
    fn parse<'a>(input: &'a [u8], alloc: &mut Option<heap::BumpBuffer>) -> ParserResult<'a, Self> {
        let (remaining, tag) = take(1usize)(input).finish()?;
        if tag[0] == 0x00u8 {
            let (remaining, min) = U32::parse(remaining, alloc)?;
            Ok((remaining, Limits { min, max: None }))
        } else if tag[0] == 0x01u8 {
            let (remaining, min) = U32::parse(remaining, alloc)?;
            let (remaining, max) = U32::parse(remaining, alloc)?;
            Ok((
                remaining,
                Limits {
                    min,
                    max: Some(max),
                },
            ))
        } else {
            Err(Error::new(remaining, ErrorKind::IsNot))
        }
    }
}

impl Parseable for RefType {
    fn parse<'a>(input: &'a [u8], alloc: &mut Option<heap::BumpBuffer>) -> ParserResult<'a, Self> {
        let (remaining, b) = take(1usize)(input).finish()?;
        let b = b[0];

        if let Ok(n) = RefType::try_from(b) {
            Ok((remaining, n))
        } else {
            Err(Error::new(remaining, ErrorKind::IsNot))
        }
    }
}

impl Parseable for TableType {
    fn parse<'a>(input: &'a [u8], alloc: &mut Option<heap::BumpBuffer>) -> ParserResult<'a, Self> {
        let (remaining, et) = RefType::parse(input, alloc)?;
        let (remaining, lim) = Limits::parse(remaining, alloc)?;
        Ok((remaining, TableType { et, lim }))
    }
}

impl Parseable for BlockType {
    fn parse<'a>(input: &'a [u8], alloc: &mut Option<heap::BumpBuffer>) -> ParserResult<'a, Self> {
        let b = input[0];

        if b == 0x40 {
            let (remaining, _) = take(1usize)(input).finish()?;
            return Ok((remaining, Self::Empty));
        }

        if let Ok(t) = ValType::try_from(b) {
            let (remaining, _) = take(1usize)(input).finish()?;
            return Ok((remaining, BlockType::T(t)));
        }

        let (remaining, x) = S33::parse(input, alloc)?;
        Ok((remaining, BlockType::X(x)))
    }
}

pub struct WasmParser {
    alloc: Option<heap::BumpBuffer>,
}

impl WasmParser {
    pub const fn new() -> Self {
        Self { alloc: None }
    }

    pub fn parse<'a>(&mut self, input: &'a [u8], module: &mut WasmModule) -> ParserResult<'a, ()> {
        self.check_magic_and_version_and_parse_sections(input, module)
    }

    fn check_magic_and_version(input: &[u8]) -> ParserResult<'_, ()> {
        let (remaining, output) = take(8usize)(input).finish()?;
        if output != MAGIC_AND_VERSION {
            Err(Error::new(remaining, ErrorKind::Verify))
        } else {
            Ok((remaining, ()))
        }
    }

    fn parse_section_content<'a>(
        &mut self,
        input: &'a [u8],
        section_header: SectionHeader,
        module: &mut WasmModule,
    ) -> ParserResult<'a, ()> {
        let (remaining, content) = take(section_header.size.0)(input).finish()?;
        match section_header.id {
            SectionType::Type => {
                self.parse_type_section(section_header, content, module)?;
            }
            SectionType::Function => {
                self.parse_func_section(section_header, content, module)?;
            }
            SectionType::Table => {
                self.parse_table_section(section_header, content, module)?;
            }
            _ => {}
        };
        Ok((remaining, ()))
    }

    fn check_magic_and_version_and_parse_sections<'a>(
        &mut self,
        input: &'a [u8],
        module: &mut WasmModule,
    ) -> ParserResult<'a, ()> {
        let mut next_section_order: u8 = SECTION_TYPE_TO_ORDER[SectionType::Type as usize];
        let (mut remaining, _) = Self::check_magic_and_version(input)?;

        let buffer_pages: usize = remaining.len().div_ceil(4096);
        let bump_buffer = heap::HEAP_ALLOCATOR
            .get()
            .unwrap()
            .alloc_bump_buffer(buffer_pages)
            .map_err(|_| Error::new(remaining, ErrorKind::Fail))?;

        self.alloc = Some(bump_buffer);

        loop {
            remaining = match SectionHeader::parse(remaining, &mut self.alloc) {
                Ok((r, section_header)) => {
                    if SECTION_TYPE_TO_ORDER[section_header.id as usize] < next_section_order {
                        return Err(Error::new(r, ErrorKind::IsNot));
                    } else {
                        if section_header.id != SectionType::Custom {
                            next_section_order =
                                SECTION_TYPE_TO_ORDER[section_header.id as usize] + 1;
                        }
                        let (r, _) = self.parse_section_content(r, section_header, module)?;
                        r
                    }
                }
                Err(e) => {
                    if e.code == ErrorKind::Eof {
                        return Ok((e.input, ()));
                    } else {
                        return Err(e);
                    }
                }
            };
        }
    }

    fn parse_type_section<'a>(
        &mut self,
        section_header: SectionHeader,
        content: &'a [u8],
        module: &mut WasmModule,
    ) -> ParserResult<'a, ()> {
        let (remaining, types) = WasmVector::<FuncType>::parse(content, &mut self.alloc)?;
        let type_sec = TypeSection::new(section_header, types);
        if module.type_section.is_some() {
            Err(Error::new(remaining, ErrorKind::Fail))
        } else {
            module.type_section = Some(type_sec);
            Ok((remaining, ()))
        }
    }
    fn parse_func_section<'a>(
        &mut self,
        section_header: SectionHeader,
        content: &'a [u8],
        module: &mut WasmModule,
    ) -> ParserResult<'a, ()> {
        let (remaining, idx) = WasmVector::<U32>::parse(content, &mut self.alloc)?;
        let func_sec = FuncSection::new(section_header, idx);
        if module.func_section.is_some() {
            Err(Error::new(remaining, ErrorKind::Fail))
        } else {
            module.func_section = Some(func_sec);
            Ok((remaining, ()))
        }
    }
    fn parse_table_section<'a>(
        &mut self,
        section_header: SectionHeader,
        content: &'a [u8],
        module: &mut WasmModule,
    ) -> ParserResult<'a, ()> {
        let (remaining, tables) = WasmVector::<TableType>::parse(content, &mut self.alloc)?;
        let table_sec = TableSection::new(section_header, tables);
        if module.table_section.is_some() {
            Err(Error::new(remaining, ErrorKind::Fail))
        } else {
            module.table_section = Some(table_sec);
            Ok((remaining, ()))
        }
    }
}

#[cfg(test)]
#[allow(unused_imports, unused_variables, dead_code)]
mod tests {
    use super::*;
    use test_macros::kernel_test;
    const WASM_MODULE: &[u8; 230] = include_bytes!("../module.wasm");
    #[kernel_test]
    fn test_parser() {
        let mut module = WasmModule::new();
        let mut parser = WasmParser::new();
        parser.parse(WASM_MODULE, &mut module).unwrap();
        println!("{}", module);

        let signed = [0xC0, 0xBB, 0x78] as [u8; 3];
        let (_, signed) = take_signed(32, &signed).unwrap();
        println!("{}", signed);
    }
}
