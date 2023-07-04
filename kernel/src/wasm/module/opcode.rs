extern crate alloc;
use super::*;
use crate::{opcode_0, opcode_1};
use alloc::{boxed::Box, vec::Vec};
use core::marker::{Send, Sync};
use paste::paste;
use test_macros::display_sync_send;

pub const LOCAL_SET: u8 = 0x21;
pub const LOCAL_TEE: u8 = 0x22;

type OpParseFn = fn(input: &[u8]) -> ParserResult<'_, InstPtr>;

const fn parse_arr() -> [OpParseFn; 0xFF] {
    let mut arr = [parse_UndefinedOp as OpParseFn; 0xFF];
    arr[UNREACHABLE as usize] = parse_Unreachable as OpParseFn;
    arr[LOCALGET as usize] = parse_LocalGet as OpParseFn;
    arr[I64ADD as usize] = parse_I64Add as OpParseFn;
    arr[CALL as usize] = parse_Call as OpParseFn;
    arr
}

pub static OP_PARSER: [OpParseFn; 0xFF] = parse_arr();

opcode_0!(UndefinedOp, {});
opcode_0!(0x00, Unreachable, {});
opcode_0!(0x7C, I64Add, {});

opcode_1!(0x10, Call, U32, {});
opcode_1!(0x20, LocalGet, U32, {});
