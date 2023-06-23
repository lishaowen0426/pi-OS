use crate::{errno::ErrorCode, type_enum, type_enum_with_error, wasm::module::WasmModule};
use core::fmt;
use nom::{
    bytes::complete::take,
    error::{Error, ErrorKind},
    Finish, IResult, ToUsize,
};

type ParserResult<'a, O> = Result<(&'a [u8], O), Error<&'a [u8]>>;

const MAGIC_AND_VERSION: [u8; 8] = [0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];

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

struct SectionHeader {
    id: SectionType,
    size: u32,
}

impl fmt::Display for SectionHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)?;
        write!(f, "(size={})", self.size)
    }
}

pub struct WasmParser {}

impl<'a> WasmParser {
    pub const fn new() -> Self {
        Self {}
    }

    pub fn parse<'b>(&'a mut self, input: &'b [u8]) -> ParserResult<'b, WasmModule> {
        let (sections, _) = check_magic_and_version(input)?;
        parse_sections(sections)
    }
}

fn take_unsigned(N: usize, input: &[u8]) -> ParserResult<'_, u64> {
    let (remaining, output) = take(1usize)(input).finish()?;
    let n = output[0];
    if n < (1 << 7) && n < (1 << N) {
        Ok((remaining, n as u64))
    } else if n >= (1 << 7) && (N > 7) {
        let (remaining, m) = take_unsigned(N - 7, remaining)?;
        Ok((remaining, (1 << 7) * m + (n as u64 - (1 << 7))))
    } else {
        Err(Error::new(remaining, ErrorKind::IsNot))
    }
}

fn take_unsigned_32(input: &[u8]) -> ParserResult<'_, u32> {
    take_unsigned(32usize, input).map(|(i, u)| (i, u as u32))
}
fn take_unsigned_64(input: &[u8]) -> ParserResult<'_, u64> {
    take_unsigned(64usize, input)
}

fn check_magic_and_version(input: &[u8]) -> ParserResult<'_, ()> {
    let (remaining, output) = take(8usize)(input).finish()?;
    if output != MAGIC_AND_VERSION {
        Err(Error::new(remaining, ErrorKind::Verify))
    } else {
        Ok((remaining, ()))
    }
}

fn parse_section_header(input: &[u8]) -> ParserResult<'_, SectionHeader> {
    let (remaining, output) = take(1usize)(input).finish()?;
    let section_type =
        SectionType::try_from(output[0]).map_err(|error_kind| Error::new(remaining, error_kind))?;
    let (remaining, size) = take_unsigned_32(remaining)?;
    Ok((
        remaining,
        SectionHeader {
            id: section_type,
            size,
        },
    ))
}

fn parse_sections(input: &[u8]) -> ParserResult<'_, WasmModule> {
    todo!()
}

#[cfg(test)]
#[allow(unused_imports, unused_variables, dead_code)]
mod tests {
    use super::*;
    use crate::println;
    use test_macros::kernel_test;
    const WASM_MODULE: &[u8; 230] = include_bytes!("module.wasm");
    #[kernel_test]
    fn test_parser() {
        let (remaining, _) = check_magic_and_version(WASM_MODULE).unwrap();
        let (remaining, section_header) = parse_section_header(remaining).unwrap();
        println!("{}", section_header);
    }
}
