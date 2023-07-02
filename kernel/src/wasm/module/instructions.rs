extern crate alloc;
use super::*;
use alloc::{boxed::Box, vec::Vec};

pub type OpCode = u8;

// How to guard this?
#[repr(transparent)]
struct Stack {
    s: *mut u8,
}

type InstFn = fn(stack: &Stack) -> ();

trait WasmInst {
    type Output;
    fn execute(&self) -> Self::Output;
}

pub struct Instruction {}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "wasm inst")
    }
}

pub struct Expr {
    instr: Vec<Instruction>,
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in self.instr.iter() {
            writeln!(f, "{}", i)?;
        }
        Ok(())
    }
}

impl Expr {
    pub fn new() -> Self {
        Self { instr: Vec::new() }
    }
}

// Control Instructions
pub enum BlockType {
    Empty,
    T(ValType),
    X(S33),
}

impl fmt::Display for BlockType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Empty => write!(f, "Empty block"),
            Self::T(v) => write!(f, "{}", v),
            Self::X(s) => write!(f, "{}", s),
        }
    }
}

// Variable Instructions
