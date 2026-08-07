// nvm-asm/src/parser/ast.rs
//
//! Определение AST.
use nvm_core::isa::{opcode::OperationCode, register::Register};

use crate::{position::Position, str_pool::StrId};

/// Абстрактное синтаксическое дерево программы на NVM Assembly.
#[derive(Debug, Clone)]
pub struct AST {
    pub program: Vec<Statement>,
}

impl AST {
    pub fn new() -> Self {
        Self {
            program: Vec::new(),
        }
    }

    pub fn with_program(program: Vec<Statement>) -> Self {
        Self { program }
    }
}

impl Default for AST {
    fn default() -> Self {
        Self::new()
    }
}

/// Одно выражение AST: метка или инструкция.
#[derive(Debug, Clone)]
pub enum Statement {
    /// Объявление метки: `name:`.
    Label { position: Position, name: StrId },

    /// Инструкция.
    Instruction {
        position: Position,
        instruction: Instr,
    },
}

/// Операнд инструкции внутри AST.
///
/// Помимо операндов байт-кода (регистр, immediate) операнд может
/// ссылаться на метку — это абстракция ассемблера. Конкретное
/// смещение (offset) метки кодогенератор вычисляет уже после разбора.
#[derive(Debug, Clone, Copy)]
pub enum Operand {
    /// Регистр.
    Register(Register),

    /// Непосредственное значение.
    Immediate(u64),

    /// Ссылка на метку по имени.
    Label(StrId),
}

/// Инструкция NVM в AST.
#[derive(Debug, Clone, Copy)]
pub struct Instr {
    pub opcode: OperationCode,
    pub operand1: Option<Operand>,
    pub operand2: Option<Operand>,
    pub operand3: Option<Operand>,
}

impl Instr {
    /// Возвращает количество операндов.
    pub fn operand_count(&self) -> usize {
        [self.operand1, self.operand2, self.operand3]
            .into_iter()
            .flatten()
            .count()
    }
}
