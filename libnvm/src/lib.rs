//! # `libnvm`
//!
//! `libnvm` — крейт для использования NVM.
//!
//! Основная точка входа — [`NVMl`]. Он принимает байт-код через
//! [`BytecodeSource`] и исполняет его.
use std::path::PathBuf;

use nvm_core::{
    loader::NVMLoader,
    vm::{NVM, memory::NVMMemory},
};

// Публичный API `libnvm`: тип ошибки и инструкции — переэкспортируем из `nvm-core`.
pub use nvm_core::NVM_VERSION;
pub use nvm_core::error::{NVMError, NVMErrorKind};
pub use nvm_core::isa::instruction::Instruction;

/// Размер памяти ВМ по умолчанию (в байтах).
pub const DEFAULT_MEMORY_SIZE: usize = 64 * 1024;

/// Источник байт-кода для [`NVMl::run`].
pub enum BytecodeSource {
    /// Путь к файлу в формате NVM Bytecode (`.nb`).
    File(PathBuf),

    /// Сырые байты байт-кода (например, прочитанные из stdin).
    Bytes(Vec<u8>),

    /// Уже распарсенные инструкции.
    Instructions(Vec<Instruction>),
}

pub struct NVMl {
    /// Размер памяти ВМ в байтах.
    pub memory_size: usize,
}

impl NVMl {
    pub fn new() -> Self {
        Self {
            memory_size: DEFAULT_MEMORY_SIZE,
        }
    }

    /// Задаёт размер памяти ВМ в байтах.
    pub fn with_memory_size(memory_size: usize) -> Self {
        Self { memory_size }
    }

    /// Исполняет байт-код из переданного [`BytecodeSource`].
    pub fn run(&self, source: BytecodeSource) -> Result<(), NVMError> {
        let instructions = match source {
            BytecodeSource::File(path) => {
                let bytes = std::fs::read(&path)
                    .map_err(|e| NVMError::new(NVMErrorKind::IoError(e), None, false))?;
                NVMLoader::new(bytes)
                    .transpile()
                    .map_err(|e| NVMError::new(NVMErrorKind::LoaderError(e), None, false))?
            }
            BytecodeSource::Bytes(bytes) => NVMLoader::new(bytes)
                .transpile()
                .map_err(|e| NVMError::new(NVMErrorKind::LoaderError(e), None, false))?,
            BytecodeSource::Instructions(instructions) => instructions,
        };

        let mut vm = NVM::from_program_and_memory(instructions, NVMMemory::new(self.memory_size));

        vm.run()
            .map_err(|e| NVMError::new(NVMErrorKind::VMError(e), None, false))?;

        Ok(())
    }
}

impl Default for NVMl {
    fn default() -> Self {
        Self::new()
    }
}
