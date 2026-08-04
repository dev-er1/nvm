// nvm-core/src/isa/operand.rs
//
//! # Операнды NVM
//!
//! В этом модуле определены типы, описывающие операнды инструкций.
//!
//! Операнд — это аргумент инструкции. В зависимости от инструкции
//! операндом может быть регистр или непосредственное значение.
use std::fmt::{self, Display, Formatter};

use crate::{
    isa::register::Register,
    vm::err::{VMError, VMErrorKind},
};

/// # Вид операнда.
///
/// Видов операнда 2:
/// 1. Регистр.
/// 2. Immediate-значение.
#[derive(Debug, Clone, Copy)]
pub enum OperandKind {
    /// Регистр виртуальной машины.
    Register(Register),

    /// Непосредственное значение (immediate).
    Immediate(u64),
}

impl Display for OperandKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Immediate(v) => write!(f, "{v}"),
            Self::Register(r) => write!(f, "{r}"),
        }
    }
}

impl OperandKind {
    /// Превращает `OperandKind` в строку с типом операнда.
    ///
    /// Нужно для [`vm::err`](crate::vm::err).
    pub fn kind(&self) -> &str {
        match self {
            Self::Immediate(_) => "value",
            Self::Register(_) => "register",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Operand {
    pub kind: OperandKind,
}

impl Display for Operand {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl Operand {
    pub fn expect_register(&self) -> Result<Register, VMError> {
        match self.kind {
            OperandKind::Register(r) => Ok(r),
            got => Err(VMError::new(VMErrorKind::IncorrectTypeOfOperand {
                expected: OperandKind::Register(Register(0)),
                got,
            })),
        }
    }

    pub fn expect_immediate(&self) -> Result<u64, VMError> {
        match self.kind {
            OperandKind::Immediate(r) => Ok(r),
            got => Err(VMError::new(VMErrorKind::IncorrectTypeOfOperand {
                expected: OperandKind::Immediate(0),
                got,
            })),
        }
    }
}
