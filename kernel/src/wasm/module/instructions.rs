extern crate alloc;
use super::*;
use alloc::boxed::Box;

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

// A sequence of instructions ended with 0x0B
struct Expr {}

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
