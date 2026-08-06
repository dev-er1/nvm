// nvm-asm/src/lexer/err.rs
//
//! Ошибки лексического анализа.
use std::fmt::{self, Display, Formatter};

use crate::position::Position;

/// Виды ошибок, которые может обнаружить лексер.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexerErrorKind {
    /// Встречен символ, который не является частью ни одного токена.
    ///
    /// Например, `!` или `@` в исходном коде.
    UnexpectedCharacter(char),

    /// Некорректный целочисленный литерал.
    InvalidInteger(String),

    /// Некорректный литерал числа с плавающей точкой.
    InvalidFloat(String),

    /// Некорректный регистр: номер выходит за пределы 0-255.
    InvalidRegister(String),
}

impl Display for LexerErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedCharacter(c) => write!(f, "unexpected character: '{c}'"),
            Self::InvalidInteger(text) => {
                write!(f, "invalid integer: '{text}'")
            }
            Self::InvalidFloat(text) => {
                write!(f, "invalid floating-point: '{text}'")
            }
            Self::InvalidRegister(text) => write!(
                f,
                "invalid register '{text}': register number must be in the range 0-255"
            ),
        }
    }
}

/// Ошибка лексического анализа вместе с позицией в исходном коде.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexerError {
    /// Вид ошибки.
    pub kind: LexerErrorKind,

    /// Позиция ошибки в исходном коде.
    pub pos: Position,
}

impl LexerError {
    /// Создаёт ошибку лексического анализа.
    pub fn new(kind: LexerErrorKind, pos: Position) -> Self {
        Self { kind, pos }
    }
}

impl Display for LexerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}
