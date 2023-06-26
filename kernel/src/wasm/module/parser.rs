use super::*;
use crate::{
    errno::ErrorCode,
    memory::heap::{self, AllocBufferIter},
    println,
};
use core::fmt;
use nom::{
    bytes::complete::*,
    error::{Error, ErrorKind},
    Finish, IResult,
};

type ParserResult<'a, O> = Result<(&'a [u8], O), Error<&'a [u8]>>;

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

pub struct WasmParser {
    alloc: Option<AllocBufferIter>,
}

impl WasmParser {
    pub const fn new() -> Self {
        Self { alloc: None }
    }

    pub fn parse<'a>(&mut self, input: &'a [u8], module: &mut WasmModule) -> ParserResult<'a, ()> {
        self.check_magic_and_version_and_parse_sections(input, module)
    }
    fn take_unsigned(N: usize, input: &[u8]) -> ParserResult<'_, u64> {
        let (remaining, lower_bytes) = take_till(|b: u8| (b & 0b10000000) == 0)(input).finish()?;
        let total_length = lower_bytes.len() + 1;
        if total_length > N.div_ceil(7usize) {
            Err(Error::new(remaining, ErrorKind::IsNot))
        } else {
            let (remaining, top_byte) = take(1usize)(remaining).finish()?;
            let mut uN: u64 = 0;
            let top_byte = top_byte[0];
            for (idx, b) in lower_bytes.iter().enumerate() {
                uN = uN | ((*b & 0b01111111u8) << (idx * 7)) as u64;
            }
            uN = uN | (top_byte << (lower_bytes.len() * 7)) as u64;
            Ok((remaining, uN))
        }
    }

    fn take_unsigned_32(input: &[u8]) -> ParserResult<'_, u32> {
        Self::take_unsigned(32usize, input).map(|(i, u)| (i, u as u32))
    }
    fn take_unsigned_64(input: &[u8]) -> ParserResult<'_, u64> {
        Self::take_unsigned(64usize, input)
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
        let section_type = SectionType::try_from(output[0])
            .map_err(|error_kind| Error::new(remaining, error_kind))?;
        let (remaining, size) = Self::take_unsigned_32(remaining)?;
        Ok((
            remaining,
            SectionHeader {
                id: section_type,
                size,
            },
        ))
    }

    fn parse_section_content<'a>(
        &mut self,
        input: &'a [u8],
        section_header: SectionHeader,
        module: &mut WasmModule,
    ) -> ParserResult<'a, ()> {
        println!("{}", section_header);
        let (remaining, content) = take(section_header.size)(input).finish()?;
        match section_header.id {
            SectionType::Type => {
                self.parse_type_section(section_header, content, module)?;
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
        let alloc_buffer = heap::HEAP_ALLOCATOR
            .get()
            .unwrap()
            .alloc_buffer(buffer_pages)
            .map_err(|_| Error::new(remaining, ErrorKind::Fail))?;
        self.alloc = Some(alloc_buffer.iter());
        module.buffer = Some(alloc_buffer);

        loop {
            remaining = match Self::parse_section_header(remaining) {
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

    fn parse_vector<'a, T>(&mut self, input: &'a [u8]) -> ParserResult<'a, WasmVector<T>> {
        todo!()
    }

    fn parse_type_section<'a>(
        &mut self,
        section_header: SectionHeader,
        content: &'a [u8],
        module: &mut WasmModule,
    ) -> ParserResult<'a, ()> {
        let (mut remaining, vec_len) = Self::take_unsigned_32(content)?;
        println!("n = {}", vec_len);

        let mut type_section = TypeSection::new(section_header);

        if let Some(ref mut alloc) = self.alloc {
            let content = alloc
                .construct_n::<FuncType>(vec_len as usize)
                .map_err(|_| Error::new(remaining, ErrorKind::Fail))?;

            type_section.init(vec_len, content);
        } else {
            return Err(Error::new(remaining, ErrorKind::Fail));
        }
        // parse functype
        for i in 0..vec_len as usize {}

        Ok((remaining, ()))
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
    }
}
