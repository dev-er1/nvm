// nvm-core/src/loader/err.rs
//
//! Loader errors.
use std::fmt::{self, Display, Formatter};

/// Error kinds.
#[derive(Debug)]
pub enum LoaderErrorKind {
    /// The file is not in NVM Bytecode format.
    FileIsNotInNVMBytecodeFormat {
        /// The full reason of the error.
        reason: String,
    },

    /// An unsupported NVM version.
    UnsupportedVersion {
        /// The version required by the file.
        file_version: String,

        /// The current VM version.
        vm_version: String,
    },

    /// An unknown opcode.
    ///
    /// ## Error example
    /// ```text
    /// [0x00] [0x01] [0x37]
    ///               ^^^^^^
    /// ```
    /// The byte `0x37` does not match any known opcode.
    UnknownOpcode {
        /// The byte that could not be recognized as an opcode.
        byte: u8,
    },

    /// An unknown operand tag.
    ///
    /// ## Error example
    /// ```text
    /// [0x05] [0x01] [0xFF] [0x2A]
    ///               ^^^^^^
    /// ```
    /// The byte `0xFF` is not a valid operand tag (`0x00` and `0x01` are allowed).
    UnknownOperandTag {
        /// The byte that could not be recognized as a tag.
        byte: u8,
    },

    /// Unexpected end of file.
    ///
    /// ## Error example
    /// ```text
    /// [0x07] [0x02]
    ///        ^^^^^^
    /// ```
    /// There are not enough bytes for a full instruction: the `IADD` opcode expects
    /// 3 operands, but the file ended earlier.
    UnexpectedEndOfFile {
        /// How many more bytes are needed.
        needed: usize,

        /// How many bytes remain.
        remaining: usize,
    },
}

impl Display for LoaderErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileIsNotInNVMBytecodeFormat { reason } => {
                write!(f, "the file is not in NVM Bytecode format: {reason}")
            }
            Self::UnsupportedVersion {
                file_version,
                vm_version,
            } => write!(
                f,
                "the file requires NVM version {file_version} or newer, but the VM version is {vm_version}"
            ),
            Self::UnknownOpcode { byte } => write!(f, "unknown opcode byte: {byte}"),
            Self::UnknownOperandTag { byte } => write!(f, "unknown operand tag: {byte}"),
            Self::UnexpectedEndOfFile { needed, remaining } => write!(
                f,
                "unexpected end of file: needed {needed} more bytes, but only {remaining} remain"
            ),
        }
    }
}

#[derive(Debug)]
pub struct LoaderError {
    pub kind: LoaderErrorKind,
}

impl LoaderError {
    pub fn new(kind: LoaderErrorKind) -> Self {
        Self { kind }
    }
}

impl Display for LoaderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}
