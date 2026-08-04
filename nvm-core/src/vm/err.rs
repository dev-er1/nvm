// nvm-core/src/vm/err.rs
//
//! Ошибки ВМ.
use std::fmt::{self, Display, Formatter};

use crate::isa::operand::OperandKind;

/// Виды ошибок.
#[derive(Debug)]
pub enum VMErrorKind {
    /// Неправильное количество операндов.
    ///
    /// ## Пример ошибки
    /// ```text
    /// MOVE R1, R2, R3
    ///              ^^
    /// ```
    /// В `MOVE` инструкции может использоваться только 2 операнда.
    IncorrectNumberOfOperands { expected: u8, got: u8 },

    /// Неправильный тип операнда.
    ///
    /// ## Пример ошибки
    /// ```text
    /// MOVE 0, R0
    ///      ^
    /// ```
    /// В `MOVE` инструкции первый операнд должен быть всегда регистром.
    IncorrectTypeOfOperand {
        expected: OperandKind,
        got: OperandKind,
    },

    /// Неправильный адрес.
    ///
    /// Ошибка выдаётся если инструкция (например, `LOAD8`) пытается получить
    /// значение в памяти по несуществующему адресу.
    InvalidAddress {
        /// В какой адрес хотели "заглянуть".
        got: usize,

        /// Длина памяти.
        memory_length: usize,
    },

    /// Деление на ноль.
    DivisionByZero,

    /// Стек вызовов пуст (RET без CALL).
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
