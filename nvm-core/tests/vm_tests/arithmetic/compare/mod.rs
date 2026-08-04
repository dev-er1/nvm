// Тесты на операции сравнения.
pub mod equality_tests;
pub mod float_equality_tests;
pub mod float_ordered_tests;
pub mod sequence_tests;
pub mod signed_compare_tests;
pub mod unsigned_compare_tests;

use nvm_core::{
    isa::{
        instruction::Instruction,
        opcode::OperationCode,
        operand::{Operand, OperandKind},
        register::Register,
    },
    vm::NVM,
};

pub fn get_int(opcode: OperationCode, a: u64, b: u64) -> u64 {
    let mut nvm = NVM::new(0);
    nvm.program = vec![Instruction {
        opcode,
        operand1: Some(Operand {
            kind: OperandKind::Register(Register(0)),
        }),
        operand2: Some(Operand {
            kind: OperandKind::Immediate(a),
        }),
        operand3: Some(Operand {
            kind: OperandKind::Immediate(b),
        }),
    }];
    nvm.match_execute().expect("execution failed");
    nvm.registers[Register(0)]
}

pub fn get_float(opcode: OperationCode, a: f64, b: f64) -> u64 {
    let mut nvm = NVM::new(0);
    nvm.program = vec![Instruction {
        opcode,
        operand1: Some(Operand {
            kind: OperandKind::Register(Register(0)),
        }),
        operand2: Some(Operand {
            kind: OperandKind::Immediate(a.to_bits()),
        }),
        operand3: Some(Operand {
            kind: OperandKind::Immediate(b.to_bits()),
        }),
    }];
    nvm.match_execute().expect("execution failed");
    nvm.registers[Register(0)]
}
