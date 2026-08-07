//! # `libnvm`
//!
//! `libnvm` — крейт для использования NVM.
//!
//! Две части:
//! - [`NVMl`] — исполнение байт-кода. Принимает байт-код через
//!   [`BytecodeSource`] и исполняет его;
//! - [`NVMAssembler`] — компиляция NVM Assembly в инструкции
//!   и в байт-код (`.nb`).
use std::path::PathBuf;

use nvm_asm::{codegen, lexer::Lexer, parser::Parser, src::SourceCode, str_pool::StrPool};
use nvm_core::{
    loader::NVMLoader,
    vm::{NVM, memory::NVMMemory},
};

// Публичный API `libnvm`: тип ошибки и инструкции — переэкспортируем из `nvm-core`.
pub use nvm_core::NVM_VERSION;
pub use nvm_core::error::{NVMError, NVMErrorKind};
pub use nvm_core::isa::instruction::Instruction;

// Ошибки компиляции — переэкспортируем из `nvm-asm`.
pub use nvm_asm::error::{NvmASMError, NvmASMErrorKind};

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

/// Компилятор NVM Assembly в NVM Bytecode.
///
/// Собирает полный конвейер компиляции текстового ассемблера:
///
/// ```text
/// текст -> лексер -> парсер -> кодогенератор -> (encoder -> .nb)
/// ```
///
/// При ошибке возвращается первая же ошибка компиляции
/// ([`NvmASMError`]) с позицией и фрагментом исходного кода.
pub struct NVMAssembler;

impl NVMAssembler {
    /// Компилирует исходный текст NVM Assembly в инструкции.
    ///
    /// Метки разрешаются в индексы инструкций (см. `codegen`).
    ///
    /// ## Пример
    ///
    /// ```rust
    /// use libnvm::NVMAssembler;
    ///
    /// let instructions = NVMAssembler::assemble("MOVE R0, 42\nEXIT").expect("valid program");
    /// assert_eq!(instructions.len(), 2);
    /// ```
    // Ошибка несёт фрагмент исходного кода для pretty-print
    // (NvmASMError::format) — это осознанный размер.
    #[allow(clippy::result_large_err)]
    pub fn assemble(source: &str) -> Result<Vec<Instruction>, NvmASMError> {
        let source = SourceCode::new(source.to_string());
        let mut str_pool = StrPool::from_source(&source);

        // ====== Лексер ======

        let (tokens, lexer_errors, source) = {
            let mut lexer = Lexer::new(source.clone(), &mut str_pool);
            let tokens = lexer.tokenize().to_vec();
            (tokens, lexer.errors.clone(), lexer.src)
        };

        if let Some(err) = lexer_errors.first() {
            return Err(NvmASMError::error(
                err.pos,
                NvmASMErrorKind::LexerError(err.clone()),
                false,
                None,
                source,
            ));
        }

        // ====== Парсер ======

        let mut parser = Parser::new(tokens);
        let ast = parser.parse().clone();

        if let Some(err) = parser.errors.first() {
            return Err(NvmASMError::error(
                err.position,
                NvmASMErrorKind::ParserError(err.clone()),
                false,
                None,
                source,
            ));
        }

        // ====== Кодогенератор ======

        codegen::generate(&ast, &str_pool).map_err(|err| {
            NvmASMError::error(
                err.position,
                NvmASMErrorKind::CodegenError(err),
                false,
                None,
                source,
            )
        })
    }

    /// Компилирует исходный текст NVM Assembly в байты `.nb`-файла.
    ///
    /// Отличается от [`Self::assemble`] кодированием инструкций в
    /// формат NVM Bytecode (см. `docs/File-Format/File-Format.md`).
    #[allow(clippy::result_large_err)]
    pub fn assemble_to_bytecode(source: &str) -> Result<Vec<u8>, NvmASMError> {
        let instructions = Self::assemble(source)?;

        Ok(codegen::encoder::encode(&instructions))
    }
}
