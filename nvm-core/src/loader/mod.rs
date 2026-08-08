//! # Loader of files in the NVM Bytecode format
//!
//! This module implements the loader of `.nb` files,
//! with conversion into a [`Vec<Instruction>`].
pub mod err;

use crate::{
    NVM_VERSION,
    isa::{instruction::Instruction, opcode::OperationCode},
    loader::err::{LoaderError, LoaderErrorKind},
};

pub struct NVMLoader {
    /// The source code — just a byte array.
    pub src: Vec<u8>,
}

impl NVMLoader {
    pub fn new(src: Vec<u8>) -> Self {
        Self { src }
    }

    #[allow(clippy::cmp_owned)]
    pub fn transpile(&self) -> Result<Vec<Instruction>, LoaderError> {
        // ====== File validity checks ======

        // The minimum size: 5 magic bytes + 6 version bytes = 11.
        if self.src.len() < 11 {
            return Err(LoaderError::new(
                LoaderErrorKind::FileIsNotInNVMBytecodeFormat {
                    reason: "the file must be at least 11 bytes in size".to_string(),
                },
            ));
        }

        // Magic check.
        if &self.src[..5] != b"NVMBC" {
            return Err(LoaderError::new(
                LoaderErrorKind::FileIsNotInNVMBytecodeFormat {
                    reason: "incorrect magic section".to_string(),
                },
            ));
        }

        // Parse the minimum NVM version.
        let first = u16::from_le_bytes([self.src[5], self.src[6]]);
        let second = u16::from_le_bytes([self.src[7], self.src[8]]);
        let third = u16::from_le_bytes([self.src[9], self.src[10]]);

        let minimal_version = format!("{first}.{second}.{third}");

        if minimal_version > NVM_VERSION.to_string() {
            return Err(LoaderError::new(LoaderErrorKind::UnsupportedVersion {
                file_version: minimal_version,
                vm_version: NVM_VERSION.to_string(),
            }));
        }
        // ====== Instruction parsing ======

        let mut instructions = Vec::new();
        let mut offset = 11;

        while offset < self.src.len() {
            // Each instruction requires at least 2 bytes (opcode + operand count).
            if offset + 2 > self.src.len() {
                return Err(LoaderError::new(LoaderErrorKind::UnexpectedEndOfFile {
                    needed: 2,
                    remaining: self.src.len() - offset,
                }));
            }

            let opcode_byte = self.src[offset];
            if opcode_byte > OperationCode::RET as u8 {
                return Err(LoaderError::new(LoaderErrorKind::UnknownOpcode {
                    byte: opcode_byte,
                }));
            }

            let operand_count = self.src[offset + 1];
            if operand_count > 3 {
                return Err(LoaderError::new(LoaderErrorKind::UnknownOpcode {
                    byte: opcode_byte,
                }));
            }

            // Compute the full instruction size.
            let mut instr_size = 2; // opcode + operand_count
            let mut pos = offset + 2;

            for _ in 0..operand_count {
                if pos >= self.src.len() {
                    return Err(LoaderError::new(LoaderErrorKind::UnexpectedEndOfFile {
                        needed: 1,
                        remaining: self.src.len() - pos,
                    }));
                }

                let op_size = match self.src[pos] {
                    0x00 => 2, // tag + 1-byte register
                    0x01 => 9, // tag + 8-byte immediate
                    tag => {
                        return Err(LoaderError::new(LoaderErrorKind::UnknownOperandTag {
                            byte: tag,
                        }));
                    }
                };

                instr_size += op_size;
                pos += op_size;
            }

            let end = offset + instr_size;
            if end > self.src.len() {
                return Err(LoaderError::new(LoaderErrorKind::UnexpectedEndOfFile {
                    needed: instr_size,
                    remaining: self.src.len() - offset,
                }));
            }

            let instr_bytes = self.src[offset..end].to_vec();
            let instruction = Instruction::try_from(instr_bytes).map_err(|e| {
                LoaderError::new(LoaderErrorKind::FileIsNotInNVMBytecodeFormat {
                    reason: format!("failed to parse instruction at offset {offset}: {e}"),
                })
            })?;

            instructions.push(instruction);
            offset = end;
        }

        Ok(instructions)
    }
}
