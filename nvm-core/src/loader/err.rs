// nvm-core/src/loader/err.rs
//
//! Ошибки загрузчика.
use std::fmt::{self, Display, Formatter};

/// Виды ошибок.
#[derive(Debug)]
pub enum LoaderErrorKind {
    /// Файл не в формате NVM Bytecode.
    FileIsNotInNVMBytecodeFormat {
        /// Полная причина ошибки.
        reason: String,
    },

    /// Неподдерживаемая версия NVM.
    UnsupportedVersion {
        /// Версия, требуемая файлом.
        file_version: String,

        /// Текущая версия ВМ.
        vm_version: String,
    },

    /// Неизвестный опкод.
    ///
    /// ## Пример ошибки
    /// ```text
    /// [0x00] [0x01] [0x37]
    ///               ^^^^^^
    /// ```
    /// Байт `0x37` не соответствует ни одному известному опкоду.
    UnknownOpcode {
        /// Байт, который не удалось распознать как опкод.
        byte: u8,
    },

    /// Неизвестный тег операнда.
    ///
    /// ## Пример ошибки
    /// ```text
    /// [0x05] [0x01] [0xFF] [0x2A]
    ///               ^^^^^^
    /// ```
    /// Байт `0xFF` не является валидным тегом операнда (допустимы `0x00` и `0x01`).
    UnknownOperandTag {
        /// Байт, который не удалось распознать как тег.
        byte: u8,
    },

    /// Неожиданный конец файла.
    ///
    /// ## Пример ошибки
    /// ```text
    /// [0x07] [0x02]
    ///        ^^^^^^
    /// ```
    /// Для полной инструкции не хватает байт: опкод `IADD` ожидает 3 операнда,
    /// но файл закончился раньше.
    UnexpectedEndOfFile {
        /// Сколько байт ещё нужно.
        needed: usize,

        /// Сколько байт осталось.
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
