// nvm-asm/src/lexer/token.rs
use nvm_core::isa::{opcode::OperationCode, register::Register};

use crate::{position::Position, str_pool::StrId};

#[derive(Debug, Clone)]
pub enum TokenKind {
    /// Мнемоника инструкции.
    Mnemonic(OperationCode),

    /// Регистр.
    Register(Register),

    /// Целочисленный литерал.
    Integer(i64),

    /// Литерал числа с плавающей точкой.
    Float(f64),

    /// Идентификатор.
    Ident(StrId),

    Comma,
    Colon,
    Dot,
    OpeningSquareBracket,
    EndingSquareBracket,
    Plus,
    Minus,
    Asterisk,
    Newline,
    End,
}

impl TokenKind {
    /// Нужен только для удобного создания [`TokenKind`].
    pub fn tokenkind(self) -> Self {
        self
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    /// Позиция токена в исходном коде.
    pub position: Position,

    /// Вид токена.
    pub kind: TokenKind,
}

impl Token {
    pub fn new(position: Position, kind: TokenKind) -> Self {
        Self { position, kind }
    }
}
