// nvm-asm/src/parser/err.rs
//
//! Ошибки синтаксического анализа.
use std::fmt::{self, Display, Formatter};

use crate::position::Position;

/// Виды ошибок, которые может обнаружить парсер.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParserErrorKind {
    /// После имени метки не хватает двоеточия: `name:`.
    ExpectedLabelColon,

    /// Между операндами не хватает запятой.
    ExpectedComma,

    /// Встречен неожиданный токен.
    UnexpectedToken {
        /// Что ожидалось (например, `"end of statement"`).
        expected: &'static str,
        /// Что встречено (Debug-представление токена).
        got: String,
    },

    /// У инструкции неверное количество операндов.
    IncorrectNumberOfOperands {
        /// Сколько операндов ждёт опкод.
        expected: u8,
        /// Сколько операндов встречено.
        got: u8,
    },

    /// Операнд, который обязан быть регистром (приёмник), — не регистр.
    ExpectedRegisterOperand {
        /// Что встречено (Debug-представление операнда).
        got: String,
    },
}

impl Display for ParserErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExpectedLabelColon => write!(f, "expected ':' after the label name"),
            Self::ExpectedComma => write!(f, "expected ',' between operands"),
            Self::UnexpectedToken { expected, got } => {
                write!(f, "expected {expected}, got {got}")
            }
            Self::IncorrectNumberOfOperands { expected, got } => {
                write!(
                    f,
                    "incorrect number of operands: expected {expected}, got {got}"
                )
            }
            Self::ExpectedRegisterOperand { got } => {
                write!(f, "the operand must be a register, got {got}")
            }
        }
    }
}

/// Ошибка синтаксического анализа вместе с позицией в исходном коде.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserError {
    /// Вид ошибки.
    pub kind: ParserErrorKind,

    /// Позиция ошибки в исходном коде.
    pub position: Position,
}

impl ParserError {
    /// Создаёт ошибку синтаксического анализа.
    pub fn new(kind: ParserErrorKind, position: Position) -> Self {
        Self { kind, position }
    }
}

impl Display for ParserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}
