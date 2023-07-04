extern crate alloc;
use super::*;
use alloc::{boxed::Box, vec::Vec};
use core::mem::transmute;

pub type OpCode = u8;

// How to guard this?
#[repr(transparent)]
struct Stack {
    s: *mut u8,
}

impl Stack {
    pub fn new(s: *mut u8) -> Self {
        Self { s }
    }

    pub fn push<T>(&mut self, value: T) {
        unsafe {
            let top = transmute::<*mut u8, *mut T>(self.s);
            top.write(value);
            let sz = core::mem::size_of::<T>();
            self.s = self.s.byte_add(sz);
        }
    }

    pub fn pop<T>(&mut self) -> T {
        unsafe {
            let sz = core::mem::size_of::<T>();
            self.s = self.s.byte_sub(sz);
            let top = transmute::<*mut u8, *mut T>(self.s);
            top.read()
        }
    }
}

pub struct ExecContext {
    s: Stack,
    module_idx: Idx,
    func_idx: Idx,
}

pub trait WasmInst: fmt::Display {
    fn execute(&self, ctx: &ExecContext);
}

pub struct Instruction {}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "wasm inst")
    }
}

pub type InstPtr = Box<dyn WasmInst + Sync + Send>;
pub struct Expr {
    instr: Vec<InstPtr>,
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

    pub fn push_instruction(&mut self, p: InstPtr) {
        self.instr.push(p);
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
