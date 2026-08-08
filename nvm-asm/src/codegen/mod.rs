//! # Code generation
//!
//! The code generator turns an [`AST`] into a program of instructions
//! ([`Vec<Instruction>`]) — the next stage after [`parser`](crate::parser).
//!
//! ## Labels
//!
//! Labels are an assembler abstraction: they do not exist in bytecode.
//! The code generator resolves label references into **instruction indices**:
//! a label points to the instruction following its declaration,
//! and a jump to a label is a jump to that instruction.
//!
//! Resolution happens in two passes:
//! 1. labels are collected and instructions are emitted; label references are temporarily
//!    replaced with a zero immediate and remembered as "fixups";
//! 2. fixups are replaced with real instruction indices.
//!
//! At the very first error (a duplicate or undefined label)
//! generation stops — the code generator works on the
//! *fail-fast* principle.
//!
//! Encoding a program into the NVM Bytecode binary format (`.nb`)
//! — in the [`encoder`] submodule.
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

/// Generates a program from an [`AST`].
///
/// Labels are resolved into instruction indices (see the module documentation).
/// On a duplicate label or a reference to a nonexistent label, the
/// very first error is returned.
pub fn generate(ast: &AST, str_pool: &StrPool) -> Result<Vec<Instruction>, CodegenError> {
    let mut labels: HashMap<StrId, usize> = HashMap::new();
    let mut instructions = Vec::new();
    let mut fixups = Vec::new();

    // ====== Pass 1: labels and instructions ======

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

    // ====== Pass 2: resolving label references ======

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

/// A label reference awaiting resolution in the second pass.
struct Fixup {
    /// Index of the instruction with this reference.
    instr_index: usize,

    /// The operand slot (0, 1, or 2) that holds the reference.
    slot: usize,

    /// The label name.
    label: StrId,

    /// Position of the instruction (for the "undefined label" error).
    position: Position,
}

/// Converts an AST operand into an instruction operand.
///
/// A label reference is replaced with a zero immediate, and the reference
/// itself is recorded in `fixups`.
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
