// nvm-core/src/error.rs
//
//! Pretty-print ошибок.
use std::fmt::{self, Display, Formatter};

use crate::{
    isa::{err::ISAError, instruction::Instruction},
    loader::err::LoaderError,
    vm::err::VMError,
};

/// Виды ошибок во всём крейте `nvm-core`.
#[derive(Debug)]
pub enum NVMErrorKind {
    ISAError(ISAError),
    VMError(VMError),
    LoaderError(LoaderError),
    IoError(std::io::Error),
}

impl Display for NVMErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ISAError(err) => write!(f, "{err}"),
            Self::VMError(err) => write!(f, "{err}"),
            Self::LoaderError(err) => write!(f, "{err}"),
            Self::IoError(err) => write!(f, "{err}"),
        }
    }
}

#[derive(Debug)]
pub struct NVMError {
    pub kind: NVMErrorKind,

    /// Инструкция (опционально).
    ///
    /// Нужно для показа ошибки.
    ///
    /// ## Как будет выглядеть вывод ошибки с этим полем (пример):
    /// ```text
    /// Error: expected type register, but got value type
    ///
    /// --> MOVE 0, R0
    ///     ^^^^^^^^^^
    /// ```
    pub instruction: Option<Instruction>,

    /// Есть ли поддержка ANSI-escape последовательностей.
    /// Нужно для того, чтобы выводить ошибку с цветами если true,
    /// и без цветов, если false.
    have_ansi: bool,
}

impl NVMError {
    pub fn new(kind: NVMErrorKind, instruction: Option<Instruction>, have_ansi: bool) -> Self {
        Self {
            kind,
            instruction,
            have_ansi,
        }
    }

    pub fn report(&self) {
        if self.have_ansi {
            println!("\x1b[1;31mError\x1b[0m: \x1b[1m{}\x1b[0m.", self.kind);
            println!();
            if let Some(instr) = self.instruction {
                println!("\x1b[36m-->\x1b[0m  {instr}");

                // Количество стрелок указывающих на инструкцию.
                let arrows = "^".repeat(format!("{instr}").len());
                println!("     \x1b[1m{arrows}\x1b[0m");
            }
        } else {
            // Всё тоже самое только без ANSI-escape последовательностей.
            println!("Error: {}.", self.kind);
            println!();
            if let Some(instr) = self.instruction {
                println!("-->  {instr}");

                let arrows = "^".repeat(format!("{instr}").len());
                println!("     {arrows}");
            }
        }
    }
}
