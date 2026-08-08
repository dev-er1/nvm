//! # NVM virtual machine
//!
//! This module defines the **NVM** virtual machine and its main components.
//!
//! ## Module contents
//!
//! - [`memory`] — the virtual machine's memory;
//! - [`register_file`] — the register bank;
//! - [`err`] — VM errors;
//! - [`executer`] — the instruction executor based on Direct Threading.
pub mod err;
pub mod executer;
pub mod memory;
pub mod register_file;

use crate::{
    isa::instruction::Instruction,
    vm::{memory::NVMMemory, register_file::RegisterFile},
};

/// # NVM virtual machine
///
/// Represents the full state of the virtual machine.
///
/// Contains:
/// - the program executed by the VM;
/// - the memory;
/// - the register bank;
/// - the call stack.
pub struct NVM {
    /// The program to be executed.
    pub program: Vec<Instruction>,

    /// The virtual machine's memory.
    pub memory: NVMMemory,

    /// The register file.
    pub registers: RegisterFile,

    /// The call stack for `CALL`/`RET`.
    pub call_stack: Vec<usize>,
}

impl NVM {
    /// Creates a new virtual machine.
    ///
    /// The program is initialized empty, the memory has the given size,
    /// and all registers are filled with default values.
    pub fn new(memory_size: usize) -> Self {
        Self {
            program: Vec::new(),
            memory: NVMMemory::new(memory_size),
            registers: RegisterFile::new(),
            call_stack: Vec::new(),
        }
    }

    /// Creates a virtual machine with the given program and memory.
    ///
    /// The register file is initialized with default values.
    pub fn from_program_and_memory(program: Vec<Instruction>, memory: NVMMemory) -> Self {
        Self {
            program,
            memory,
            registers: RegisterFile::new(),
            call_stack: Vec::new(),
        }
    }
}
