// nvm-core/src/isa/opcode.rs
//
//! # Опкоды NVM
//!
//! В этом модуле определены опкоды виртуальной машины NVM.
//!
//! ## Что такое "опкод"
//!
//! опкод (opcode, Operation Code) — это часть машинной инструкции,
//! которая указывает процессору или виртуальной машине, какое именно
//! действие нужно выполнить. (взято из: <https://ru.wikipedia.org/wiki/Код_операции>)
//!
//! В рамках NVM, опкод, это байт, описывающий операцию, которую нужно выполнить
//! виртуальной машиной.
//!
//! ## Что такое "операнд"
//!
//! Операнд — это аргумент инструкции. В зависимости от опкода операндом
//! может быть регистр, непосредственная константа или адрес памяти.
//!
//! ## Обозначения
//!
//! В документации используются следующие обозначения:
//!
//! - "dst" — операнд назначения (destination);
//! - "src1" — первый исходный операнд;
//! - "src2" — второй исходный операнд.
use std::str::FromStr;

use crate::isa::err::{ISAError, ISAErrorKind};

/// Перечисление опкодов для ВМ.
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum OperationCode {
    /// Ничего не делает.
    NOP,

    /// Остановление ВМ.
    ///
    /// ```text
    /// EXIT
    /// ```
    EXIT,

    /// Копирование значения из второго операнда в первый операнд.
    ///
    /// ```text
    /// MOVE <dst>, <src1>
    /// ```
    ///
    /// В dst запишется значение из src1.
    MOVE,

    /// Загрузка 8 бит из памяти в операнд.
    ///
    /// ```text
    /// LOAD8 <dst>, <src1>
    /// ```
    ///
    /// В dst запишется 1 байт, считанный по адресу из src1.
    LOAD8,

    /// Загрузка 16 бит из памяти в операнд.
    ///
    /// ```text
    /// LOAD16 <dst>, <src1>
    /// ```
    ///
    /// В dst запишется 2 байта, считанные по адресу из src1.
    LOAD16,

    /// Загрузка 32 бит из памяти в операнд.
    ///
    /// ```text
    /// LOAD32 <dst>, <src1>
    /// ```
    ///
    /// В dst запишется 4 байта, считанные по адресу из src1.
    LOAD32,

    /// Загрузка 64 бит из памяти в операнд.
    ///
    /// ```text
    /// LOAD64 <dst>, <src1>
    /// ```
    ///
    /// В dst запишется 8 байт, считанные по адресу из src1.
    LOAD64,

    /// Запись 8 бит из операнда в память.
    ///
    /// ```text
    /// STORE8 <dst>, <src1>
    /// ```
    ///
    /// По адресу из dst будет записан 1 байт из src1.
    STORE8,

    /// Запись 16 бит из операнда в память.
    ///
    /// ```text
    /// STORE16 <dst>, <src1>
    /// ```
    ///
    /// По адресу из dst будет записано 2 байта из src1.
    STORE16,

    /// Запись 32 бит из операнда в память.
    ///
    /// ```text
    /// STORE32 <dst>, <src1>
    /// ```
    ///
    /// По адресу из dst будет записано 4 байта из src1.
    STORE32,

    /// Запись 64 бит из операнда в память.
    ///
    /// ```text
    /// STORE64 <dst>, <src1>
    /// ```
    ///
    /// По адресу из dst будет записано 8 байта из src1.
    STORE64,

    /// Сложение двух целочисленных значений.
    ///
    /// Складывает src1 и src2, после чего записывает результат в dst.
    ///
    /// ```text
    /// IADD <dst>, <src1>, <src2>
    /// ```
    IADD,

    /// Вычитание двух целочисленных значений.
    ///
    /// Вычитает src2 из src1 и записывает результат в dst.
    ///
    /// ```text
    /// ISUB <dst>, <src1>, <src2>
    /// ```
    ISUB,

    /// Умножение двух целочисленных значений.
    ///
    /// Перемножает src1 и src2, после чего записывает результат в dst.
    ///
    /// ```text
    /// IMUL <dst>, <src1>, <src2>
    /// ```
    IMUL,

    /// Целочисленное знаковое деление.
    ///
    /// Делит src1 на src2 и записывает результат в dst.
    ///
    /// ```text
    /// SDIV <dst>, <src1>, <src2>
    /// ```
    SDIV,

    /// Целочисленное беззнаковое деление.
    ///
    /// Делит src1 на src2 как беззнаковые значения и записывает результат в dst.
    ///
    /// ```text
    /// UDIV <dst>, <src1>, <src2>
    /// ```
    UDIV,

    /// Остаток от знакового деления.
    ///
    /// Вычисляет src1 % src2 и записывает результат в dst.
    ///
    /// ```text
    /// SREM <dst>, <src1>, <src2>
    /// ```
    SREM,

    /// Остаток от беззнакового деления.
    ///
    /// Вычисляет src1 % src2 как беззнаковые значения и записывает результат в dst.
    ///
    /// ```text
    /// UREM <dst>, <src1>, <src2>
    /// ```
    UREM,

    /// Смена знака целочисленного значения.
    ///
    /// ```text
    /// INEG <dst>, <src1>
    /// ```
    INEG,

    /// Сложение двух чисел с плавающей точкой.
    ///
    /// Складывает src1 и src2, после чего записывает результат в dst.
    ///
    /// ```text
    /// FADD <dst>, <src1>, <src2>
    /// ```
    FADD,

    /// Вычитание двух чисел с плавающей точкой.
    ///
    /// Вычитает src2 из src1 и записывает результат в dst.
    ///
    /// ```text
    /// FSUB <dst>, <src1>, <src2>
    /// ```
    FSUB,

    /// Умножение двух чисел с плавающей точкой.
    ///
    /// Перемножает src1 и src2, после чего записывает результат в dst.
    ///
    /// ```text
    /// FMUL <dst>, <src1>, <src2>
    /// ```
    FMUL,

    /// Деление двух чисел с плавающей точкой.
    ///
    /// Делит src1 на src2 и записывает результат в dst.
    ///
    /// ```text
    /// FDIV <dst>, <src1>, <src2>
    /// ```
    FDIV,

    /// Остаток от деления двух чисел с плавающей точкой.
    ///
    /// Вычисляет остаток от деления src1 на src2 и записывает результат в dst.
    ///
    /// ```text
    /// FREM <dst>, <src1>, <src2>
    /// ```
    FREM,

    /// Смена знака числа с плавающей точкой.
    ///
    /// ```text
    /// FNEG <dst>, <src1>
    /// ```
    FNEG,

    /// Побитовое И.
    ///
    /// ```text
    /// AND <dst>, <src1>, <src2>
    /// ```
    AND,

    /// Побитовое ИЛИ.
    ///
    /// ```text
    /// OR <dst>, <src1>, <src2>
    /// ```
    OR,

    /// Побитовое исключающее ИЛИ.
    ///
    /// ```text
    /// XOR <dst>, <src1>, <src2>
    /// ```
    XOR,

    /// Побитовое НЕ.
    ///
    /// ```text
    /// NOT <dst>, <src1>
    /// ```
    NOT,

    /// Логический сдвиг влево.
    ///
    /// ```text
    /// SHL <dst>, <src1>, <src2>
    /// ```
    SHL,

    /// Логический сдвиг вправо.
    ///
    /// ```text
    /// SHR <dst>, <src1>, <src2>
    /// ```
    SHR,

    /// Арифметический сдвиг вправо.
    ///
    /// ```text
    /// SAR <dst>, <src1>, <src2>
    /// ```
    SAR,

    /// Проверка на равенство.
    ///
    /// Записывает 1 в dst, если src1 == src2, иначе 0.
    ///
    /// ```text
    /// IEQ <dst>, <src1>, <src2>
    /// ```
    IEQ,

    /// Проверка на неравенство.
    ///
    /// ```text
    /// INE <dst>, <src1>, <src2>
    /// ```
    INE,

    /// Знаковое меньше.
    ///
    /// ```text
    /// SLT <dst>, <src1>, <src2>
    /// ```
    SLT,

    /// Знаковое меньше либо равно.
    ///
    /// ```text
    /// SLE <dst>, <src1>, <src2>
    /// ```
    SLE,

    /// Знаковое больше.
    ///
    /// ```text
    /// SGT <dst>, <src1>, <src2>
    /// ```
    SGT,

    /// Знаковое больше либо равно.
    ///
    /// ```text
    /// SGE <dst>, <src1>, <src2>
    /// ```
    SGE,

    /// Беззнаковое меньше.
    ///
    /// ```text
    /// ULT <dst>, <src1>, <src2>
    /// ```
    ULT,

    /// Беззнаковое меньше либо равно.
    ///
    /// ```text
    /// ULE <dst>, <src1>, <src2>
    /// ```
    ULE,

    /// Беззнаковое больше.
    ///
    /// ```text
    /// UGT <dst>, <src1>, <src2>
    /// ```
    UGT,

    /// Беззнаковое больше либо равно.
    ///
    /// ```text
    /// UGE <dst>, <src1>, <src2>
    /// ```
    UGE,

    /// Проверка на равенство.
    ///
    /// ```text
    /// FEQ <dst>, <src1>, <src2>
    /// ```
    FEQ,

    /// Проверка на неравенство.
    ///
    /// ```text
    /// FNE <dst>, <src1>, <src2>
    /// ```
    FNE,

    /// Меньше.
    ///
    /// ```text
    /// FLT <dst>, <src1>, <src2>
    /// ```
    FLT,

    /// Меньше либо равно.
    ///
    /// ```text
    /// FLE <dst>, <src1>, <src2>
    /// ```
    FLE,

    /// Больше.
    ///
    /// ```text
    /// FGT <dst>, <src1>, <src2>
    /// ```
    FGT,

    /// Больше либо равно.
    ///
    /// ```text
    /// FGE <dst>, <src1>, <src2>
    /// ```
    FGE,

    /// Безусловный переход.
    ///
    /// ```text
    /// JMP <offset>
    /// ```
    JMP,

    /// Переход, если src1 == 0.
    ///
    /// ```text
    /// JZ <src1>, <offset>
    /// ```
    JZ,

    /// Переход, если src1 != 0.
    ///
    /// ```text
    /// JNZ <src1>, <offset>
    /// ```
    JNZ,

    /// Вызов подпрограммы.
    ///
    /// ```text
    /// CALL <offset>
    /// ```
    CALL,

    /// Возврат из подпрограммы.
    ///
    /// ```text
    /// RET
    /// ```
    RET,
}

impl FromStr for OperationCode {
    type Err = ISAError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "nop" => Ok(Self::NOP),
            "exit" => Ok(Self::EXIT),
            "move" => Ok(Self::MOVE),

            "load8" => Ok(Self::LOAD8),
            "load16" => Ok(Self::LOAD16),
            "load32" => Ok(Self::LOAD32),
            "load64" => Ok(Self::LOAD64),

            "store8" => Ok(Self::STORE8),
            "store16" => Ok(Self::STORE16),
            "store32" => Ok(Self::STORE32),
            "store64" => Ok(Self::STORE64),

            "iadd" => Ok(Self::IADD),
            "isub" => Ok(Self::ISUB),
            "imul" => Ok(Self::IMUL),
            "sdiv" => Ok(Self::SDIV),
            "udiv" => Ok(Self::UDIV),
            "srem" => Ok(Self::SREM),
            "urem" => Ok(Self::UREM),
            "ineg" => Ok(Self::INEG),

            "fadd" => Ok(Self::FADD),
            "fsub" => Ok(Self::FSUB),
            "fmul" => Ok(Self::FMUL),
            "fdiv" => Ok(Self::FDIV),
            "frem" => Ok(Self::FREM),
            "fneg" => Ok(Self::FNEG),

            "and" => Ok(Self::AND),
            "or" => Ok(Self::OR),
            "xor" => Ok(Self::XOR),
            "not" => Ok(Self::NOT),
            "shl" => Ok(Self::SHL),
            "shr" => Ok(Self::SHR),
            "sar" => Ok(Self::SAR),

            "ieq" => Ok(Self::IEQ),
            "ine" => Ok(Self::INE),
            "slt" => Ok(Self::SLT),
            "sle" => Ok(Self::SLE),
            "sgt" => Ok(Self::SGT),
            "sge" => Ok(Self::SGE),
            "ult" => Ok(Self::ULT),
            "ule" => Ok(Self::ULE),
            "ugt" => Ok(Self::UGT),
            "uge" => Ok(Self::UGE),

            "feq" => Ok(Self::FEQ),
            "fne" => Ok(Self::FNE),
            "flt" => Ok(Self::FLT),
            "fle" => Ok(Self::FLE),
            "fgt" => Ok(Self::FGT),
            "fge" => Ok(Self::FGE),

            "jmp" => Ok(Self::JMP),
            "jz" => Ok(Self::JZ),
            "jnz" => Ok(Self::JNZ),
            "call" => Ok(Self::CALL),
            "ret" => Ok(Self::RET),

            _ => Err(ISAError::new(ISAErrorKind::UnknownOperationCode(
                s.to_string(),
            ))),
        }
    }
}

impl TryFrom<u8> for OperationCode {
    type Error = ISAError;

    #[allow(clippy::missing_transmute_annotations)]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value <= OperationCode::RET as u8 {
            // SAFETY: все значения от 0 до RET являются валидными вариантами enum.
            Ok(unsafe { std::mem::transmute(value) })
        } else {
            Err(ISAError::new(ISAErrorKind::UnknownOperationCode(
                value.to_string(),
            )))
        }
    }
}
