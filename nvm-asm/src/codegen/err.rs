// nvm-asm/src/codegen/err.rs
//
//! Ошибки кодогенерации.
use std::fmt::{self, Display, Formatter};

use crate::position::Position;

/// Виды ошибок, которые может обнаружить кодогенератор.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodegenErrorKind {
    /// Метка объявлена более одного раза.
    DuplicateLabel {
        /// Имя метки.
        name: String,
    },

    /// Ссылка на несуществующую метку.
    UndefinedLabel {
        /// Имя метки.
        name: String,
    },
}

impl Display for CodegenErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateLabel { name } => write!(f, "duplicate label '{name}'"),
            Self::UndefinedLabel { name } => write!(f, "undefined label '{name}'"),
        }
    }
}

/// Ошибка кодогенерации вместе с позицией в исходном коде.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodegenError {
    /// Вид ошибки.
    pub kind: CodegenErrorKind,

    /// Позиция ошибки в исходном коде.
    pub position: Position,
}

impl CodegenError {
    /// Создаёт ошибку кодогенерации.
    pub fn new(kind: CodegenErrorKind, position: Position) -> Self {
        Self { kind, position }
    }
}

impl Display for CodegenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}
