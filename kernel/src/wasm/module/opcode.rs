extern crate alloc;
use super::*;
use alloc::{boxed::Box, vec::Vec};
use core::marker::{Send, Sync};
use test_macros::display_sync_send;

pub const UNREACHABLE: u8 = 0x00;
pub const CALL: u8 = 0x10;
pub const LOCAL_GET: u8 = 0x20;
pub const LOCAL_SET: u8 = 0x21;
pub const LOCAL_TEE: u8 = 0x22;
pub const I64_ADD: u8 = 0x7C;

type OpParseFn = fn(input: &[u8]) -> ParserResult<'_, InstPtr>;

const fn parse_arr() -> [OpParseFn; 0xFF] {
    let arr = [parse_undefined as OpParseFn; 0xFF];
    arr
}

#[display_sync_send]
pub struct UndefinedOP;

impl WasmInst for UndefinedOP {
    fn execute(&self, ctx: &ExecContext) {}
}
fn parse_undefined(input: &[u8]) -> ParserResult<'_, InstPtr> {
    Ok((input, Box::new(UndefinedOP)))
}

#[display_sync_send]
pub struct LocalGet(u32);

impl WasmInst for LocalGet {
    fn execute(&self, ctx: &ExecContext) {}
}

fn parse_local_get(input: &[u8]) -> ParserResult<'_, InstPtr> {
    let (remaining, x) = U32::parse(input)?;
    Ok((remaining, Box::new(LocalGet(x.0))))
}
