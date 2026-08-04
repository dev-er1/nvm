//! # Виртуальная машина NVM
//!
//! В этом модуле определена виртуальная машина **NVM**, а также её
//! основные компоненты.
//!
//! ## Особенности
//!
//! Архитектура NVM допускает существование нескольких исполнителей
//! (executors), реализующих различные способы выполнения инструкций.
//! Все исполнители работают с одной и той же структурой [`NVM`],
//! содержащей состояние виртуальной машины.
//!
//! ## Содержимое модуля
//!
//! - [`memory`] — память виртуальной машины;
//! - [`register_file`] — банк регистров;
//! - [`err`] — ошибки ВМ.
//! - [`default`] — стандартный исполнитель инструкций на основе `match`.
pub mod default;
pub mod err;
pub mod jumptable;
pub mod memory;
pub mod register_file;

use crate::{
    isa::instruction::Instruction,
    vm::{memory::NVMMemory, register_file::RegisterFile},
};

/// # Виртуальная машина NVM
///
/// Представляет собой полное состояние виртуальной машины.
///
/// Содержит:
/// - программу, выполняемую виртуальной машиной;
/// - память;
/// - банк регистров;
/// - стек вызовов.
///
/// Структура не определяет способ выполнения инструкций.
/// За их исполнение отвечают отдельные модули-исполнители.
pub struct NVM {
    /// Выполняемая программа.
    pub program: Vec<Instruction>,

    /// Память виртуальной машины.
    pub memory: NVMMemory,

    /// Хранилище регистров.
    pub registers: RegisterFile,

    /// Стек вызовов для `CALL`/`RET`.
    pub call_stack: Vec<usize>,
}

impl NVM {
    /// Создаёт новую виртуальную машину.
    ///
    /// Программа инициализируется пустой, память имеет указанный размер,
    /// а все регистры заполняются значениями по умолчанию.
    pub fn new(memory_size: usize) -> Self {
        Self {
            program: Vec::new(),
            memory: NVMMemory::new(memory_size),
            registers: RegisterFile::new(),
            call_stack: Vec::new(),
        }
    }

    /// Создаёт виртуальную машину с указанной программой и памятью.
    ///
    /// Хранилище регистров инициализируется значениями по умолчанию.
    pub fn from_program_and_memory(program: Vec<Instruction>, memory: NVMMemory) -> Self {
        Self {
            program,
            memory,
            registers: RegisterFile::new(),
            call_stack: Vec::new(),
        }
    }
}
