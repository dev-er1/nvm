//! # `libnvm`
//!
//! `libnvm` is the crate for using NVM.
//!
//! Two parts:
//! - [`NVMl`] — bytecode execution. It takes bytecode via
//!   [`BytecodeSource`] and executes it;
//! - [`NVMAssembler`] — compilation of NVM Assembly into instructions
//!   and into bytecode (`.nb`).
use std::path::PathBuf;

use nvm_asm::{codegen, lexer::Lexer, parser::Parser, src::SourceCode, str_pool::StrPool};
use nvm_core::{
    loader::NVMLoader,
    vm::{NVM, memory::NVMMemory},
};

// The public API of `libnvm`: the error type and instructions — re-exported from `nvm-core`.
pub use nvm_core::NVM_VERSION;
pub use nvm_core::error::{NVMError, NVMErrorKind};
pub use nvm_core::isa::instruction::Instruction;

// Compilation errors — re-exported from `nvm-asm`.
pub use nvm_asm::error::{NvmASMError, NvmASMErrorKind};

/// The default VM memory size (in bytes).
pub const DEFAULT_MEMORY_SIZE: usize = 64 * 1024;

/// The bytecode source for [`NVMl::run`].
pub enum BytecodeSource {
    /// A path to a file in the NVM Bytecode (`.nb`) format.
    File(PathBuf),

    /// Raw bytecode bytes (for example, read from stdin).
    Bytes(Vec<u8>),

    /// Already parsed instructions.
    Instructions(Vec<Instruction>),
}

pub struct NVMl {
    /// VM memory size in bytes.
    pub memory_size: usize,
}

impl NVMl {
    pub fn new() -> Self {
        Self {
            memory_size: DEFAULT_MEMORY_SIZE,
        }
    }

    /// Sets the VM memory size in bytes.
    pub fn with_memory_size(memory_size: usize) -> Self {
        Self { memory_size }
    }

    /// Executes the bytecode from the given [`BytecodeSource`].
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

/// Compiler from NVM Assembly into NVM Bytecode.
///
/// Builds the full compilation pipeline of the textual assembler:
///
/// ```text
/// text -> lexer -> parser -> codegen [-> encoder -> .nb]
/// ```
///
/// On error, the very first compilation error ([`NvmASMError`]) is
/// returned with a position and a fragment of the source code.
pub struct NVMAssembler;

impl NVMAssembler {
    /// Compiles NVM Assembly source text into instructions.
    ///
    /// Labels are resolved into instruction indices (see `codegen`).
    ///
    /// ## Example
    ///
    /// ```rust
    /// use libnvm::NVMAssembler;
    ///
    /// let instructions = NVMAssembler::assemble("MOVE R0, 42\nEXIT").expect("valid program");
    /// assert_eq!(instructions.len(), 2);
    /// ```
    // An error carries a fragment of the source code for pretty-printing
    // (NvmASMError::format) — this is a deliberate size.
    #[allow(clippy::result_large_err)]
    pub fn assemble(source: &str) -> Result<Vec<Instruction>, NvmASMError> {
        let source = SourceCode::new(source.to_string());
        let mut str_pool = StrPool::from_source(&source);

        // ====== Lexer ======

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

        // ====== Parser ======

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

        // ====== Code generator ======

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

    /// Compiles NVM Assembly source text into the bytes of a `.nb` file.
    ///
    /// Unlike [`Self::assemble`], this encodes the instructions into
    /// the NVM Bytecode format (see `docs/File-Format/File-Format.md`).
    #[allow(clippy::result_large_err)]
    pub fn assemble_to_bytecode(source: &str) -> Result<Vec<u8>, NvmASMError> {
        let instructions = Self::assemble(source)?;

        Ok(codegen::encoder::encode(&instructions))
    }
}
