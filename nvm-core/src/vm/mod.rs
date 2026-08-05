//! # Виртуальная машина NVM
//!
//! В этом модуле определена виртуальная машина **NVM**, а также её
//! основные компоненты.
//!
//! ## Содержимое модуля
//!
//! - [`memory`] — память виртуальной машины;
//! - [`register_file`] — банк регистров;
//! - [`err`] — ошибки ВМ;
//! - [`executer`] — исполнитель инструкций на основе Direct Threading.
pub mod err;
pub mod executer;
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
