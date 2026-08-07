// nvm-asm/src/codegen/encoder.rs
//
//! # Генератор NVM Bytecode (`.nb`)
//!
//! Кодирует программу из [`Instruction`] в байтовый формат NVM Bytecode —
//! формат, в котором хранятся и исполняются программы виртуальной машины
//! (см. `docs/File-Format/File-Format.md`).
//!
//! ## Использование
//!
//! Генератор принимает готовую программу — результат [`generate`](super::generate):
//!
//! ```text
//! текст -> лексер -> парсер -> кодогенератор -> encoder -> .nb
//! ```
//!
//! Кодирование не может завершиться ошибкой: любая инструкция из опкода
//! и до трёх операндов (регистр или immediate) представима в этом формате.
use nvm_core::{
    NVM_VERSION,
    isa::{
        instruction::Instruction,
        operand::{Operand, OperandKind},
        register::Register,
    },
};

/// Магическая сигнатура `.nb`-файла: `NVMBC`.
const MAGIC: [u8; 5] = *b"NVMBC";

/// Размер заголовка: 5 байт magic + 6 байт версии.
const HEADER_SIZE: usize = 11;

/// Кодирует программу в формат NVM Bytecode.
///
/// В заголовок записывается минимальная требуемая версия NVM — текущая
/// версия ядра [`NVM_VERSION`] в формате `major.minor.patch`.
pub fn encode(instructions: &[Instruction]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(HEADER_SIZE + instructions.len() * 11);

    bytes.extend_from_slice(&MAGIC);
    push_version(&mut bytes, NVM_VERSION);

    for instruction in instructions {
        bytes.push(instruction.opcode as u8);
        bytes.push(instruction.operand_count() as u8);

        for operand in [
            instruction.operand1,
            instruction.operand2,
            instruction.operand3,
        ]
        .into_iter()
        .flatten()
        {
            push_operand(&mut bytes, operand);
        }
    }

    bytes
}

/// Записывает версию `major.minor.patch` тремя `u16` в Little-Endian.
///
/// Нечисловые части (например, суффикс пререлиза) обрезаются,
/// отсутствующие части считаются нулём.
fn push_version(bytes: &mut Vec<u8>, version: &str) {
    let mut parts = version.split('.').map(version_number);

    for _ in 0..3 {
        bytes.extend_from_slice(&parts.next().unwrap_or(0).to_le_bytes());
    }
}

/// Разбирает числовую часть строки; для не-числа возвращает 0.
fn version_number(part: &str) -> u16 {
    part.chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .unwrap_or(0)
}

/// Записывает операнд: байт тега и данные.
fn push_operand(bytes: &mut Vec<u8>, operand: Operand) {
    match operand.kind {
        OperandKind::Register(Register(number)) => {
            bytes.push(0x00);
            bytes.push(number);
        }
        OperandKind::Immediate(value) => {
            bytes.push(0x01);
            bytes.extend_from_slice(&value.to_le_bytes());
        }
    }
}
