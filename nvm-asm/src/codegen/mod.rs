//! # Кодогенерация
//!
//! Кодогенератор превращает [`AST`] в программу из инструкций
//! ([`Vec<Instruction>`]) — следующий этап после [`parser`](crate::parser).
//!
//! ## Метки
//!
//! Метки — абстракция ассемблера: в байт-коде их не существует.
//! Кодогенератор разрешает ссылки на метки в **индексы инструкций**:
//! метка указывает на инструкцию, следующую за её объявлением,
//! переход на метку — это переход на эту инструкцию.
//!
//! Разрешение происходит в два прохода:
//! 1. собираются метки и эмитятся инструкции, ссылки на метки временно
//!    заменяются нулевым immediate и запоминаются как "фиксапы";
//! 2. фиксапы заменяются реальными индексами инструкций.
//!
//! При первой же ошибке (дубликат или неопределённая метка)
//! генерация останавливается — кодогенератор работает по принципу
//! *fail-fast*.
//!
//! Кодирование программы в байтовый формат NVM Bytecode (`.nb`)
//! — в подмодуле [`encoder`].
pub mod encoder;
pub mod err;

use std::collections::HashMap;

use nvm_core::isa::{
    instruction::Instruction,
    operand::{Operand as CoreOperand, OperandKind},
};

use crate::{
    parser::ast::{AST, Operand, Statement},
    position::Position,
    str_pool::{StrId, StrPool},
};

use self::err::{CodegenError, CodegenErrorKind};

/// Генерирует программу из [`AST`].
///
/// Метки разрешаются в индексы инструкций (см. документацию модуля).
/// При дубликате метки или ссылке на несуществующую метку возвращается
/// первая же ошибка.
pub fn generate(ast: &AST, str_pool: &StrPool) -> Result<Vec<Instruction>, CodegenError> {
    let mut labels: HashMap<StrId, usize> = HashMap::new();
    let mut instructions = Vec::new();
    let mut fixups = Vec::new();

    // ====== Проход 1: метки и инструкции ======

    for statement in &ast.program {
        match statement {
            Statement::Label { position, name } => {
                if labels.contains_key(name) {
                    return Err(CodegenError::new(
                        CodegenErrorKind::DuplicateLabel {
                            name: str_pool.get(*name).to_string(),
                        },
                        *position,
                    ));
                }

                labels.insert(*name, instructions.len());
            }

            Statement::Instruction {
                position,
                instruction,
            } => {
                let instr_index = instructions.len();

                let operand1 =
                    flatten(instruction.operand1, instr_index, 0, *position, &mut fixups);
                let operand2 =
                    flatten(instruction.operand2, instr_index, 1, *position, &mut fixups);
                let operand3 =
                    flatten(instruction.operand3, instr_index, 2, *position, &mut fixups);

                instructions.push(Instruction {
                    opcode: instruction.opcode,
                    operand1,
                    operand2,
                    operand3,
                });
            }
        }
    }

    // ====== Проход 2: разрешение ссылок на метки ======

    for fixup in fixups {
        let Some(&target) = labels.get(&fixup.label) else {
            return Err(CodegenError::new(
                CodegenErrorKind::UndefinedLabel {
                    name: str_pool.get(fixup.label).to_string(),
                },
                fixup.position,
            ));
        };

        let operand = Some(CoreOperand {
            kind: OperandKind::Immediate(target as u64),
        });

        match fixup.slot {
            0 => instructions[fixup.instr_index].operand1 = operand,
            1 => instructions[fixup.instr_index].operand2 = operand,
            2 => instructions[fixup.instr_index].operand3 = operand,
            _ => unreachable!("slot is always 0..=2"),
        }
    }

    Ok(instructions)
}

/// Ссылка на метку, ожидающая разрешения во втором проходе.
struct Fixup {
    /// Индекс инструкции с этой ссылкой.
    instr_index: usize,

    /// Слот операнда (0, 1 или 2), в котором стоит ссылка.
    slot: usize,

    /// Имя метки.
    label: StrId,

    /// Позиция инструкции (для ошибки "undefined label").
    position: Position,
}

/// Превращает операнд AST в операнд инструкции.
///
/// Ссылка на метку заменяется нулевым immediate, а сам факт ссылки
/// записывается в `fixups`.
fn flatten(
    operand: Option<Operand>,
    instr_index: usize,
    slot: usize,
    position: Position,
    fixups: &mut Vec<Fixup>,
) -> Option<CoreOperand> {
    match operand {
        None => None,
        Some(Operand::Register(register)) => Some(CoreOperand {
            kind: OperandKind::Register(register),
        }),
        Some(Operand::Immediate(value)) => Some(CoreOperand {
            kind: OperandKind::Immediate(value),
        }),
        Some(Operand::Label(label)) => {
            fixups.push(Fixup {
                instr_index,
                slot,
                label,
                position,
            });

            Some(CoreOperand {
                kind: OperandKind::Immediate(0),
            })
        }
    }
}
