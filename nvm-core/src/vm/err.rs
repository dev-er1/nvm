// nvm-core/src/vm/err.rs
//
//! VM errors.
use std::fmt::{self, Display, Formatter};

use crate::isa::operand::OperandKind;

/// Error kinds.
#[derive(Debug)]
pub enum VMErrorKind {
    /// Incorrect number of operands.
    ///
    /// ## Example error
    /// ```text
    /// MOVE R1, R2, R3
    ///              ^^
    /// ```
    /// The `MOVE` instruction can use only 2 operands.
    IncorrectNumberOfOperands { expected: u8, got: u8 },

    /// Incorrect operand type.
    ///
    /// ## Example error
    /// ```text
    /// MOVE 0, R0
    ///      ^
    /// ```
    /// In the `MOVE` instruction, the first operand must always be a register.
    IncorrectTypeOfOperand {
        expected: OperandKind,
        got: OperandKind,
    },

    /// Invalid address.
    ///
    /// The error is raised if an instruction (for example, `LOAD8`) tries to get
    /// a value from memory at a nonexistent address.
    InvalidAddress {
        /// The address that was attempted to "look into".
        got: usize,

        /// The memory length.
        memory_length: usize,
    },

    /// Division by zero.
    DivisionByZero,

    /// The call stack is empty (RET without CALL).
    EmptyCallStack,
}

impl Display for VMErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::IncorrectNumberOfOperands { expected, got } => {
                write!(f, "expected {expected} operands, but got {got} operands")
            }
            Self::IncorrectTypeOfOperand { expected, got } => write!(
                f,
                "expected type {}, but got {} type",
                expected.kind(),
                got.kind()
            ),
            Self::InvalidAddress { got, memory_length } => write!(
                f,
                "memory access out of bounds: address {got} is outside memory (size: {memory_length} bytes)"
            ),
            Self::DivisionByZero => write!(f, "division by zero"),
            Self::EmptyCallStack => write!(f, "empty call stack"),
        }
    }
}

#[derive(Debug)]
pub struct VMError {
    pub kind: VMErrorKind,
}

impl Display for VMError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl VMError {
    pub fn new(kind: VMErrorKind) -> Self {
        Self { kind }
    }
}
