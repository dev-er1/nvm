// Tests for bitwise operations.
pub mod and_tests;
pub mod not_tests;
pub mod or_tests;
pub mod sar_tests;
pub mod sequence_tests;
pub mod shl_tests;
pub mod shr_tests;
pub mod xor_tests;

use nvm_core::{
    isa::{
        instruction::Instruction,
        opcode::OperationCode,
        operand::{Operand, OperandKind},
        register::Register,
    },
    vm::NVM,
};

pub fn get_result(opcode: OperationCode, a: u64, b: u64) -> u64 {
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
    nvm.run().expect("execution failed");
    nvm.registers[Register(0)]
}

pub fn get_not_result(a: u64) -> u64 {
    let mut nvm = NVM::new(0);
    nvm.program = vec![Instruction {
        opcode: OperationCode::NOT,
        operand1: Some(Operand {
            kind: OperandKind::Register(Register(0)),
        }),
        operand2: Some(Operand {
            kind: OperandKind::Immediate(a),
        }),
        operand3: None,
    }];
    nvm.run().expect("execution failed");
    nvm.registers[Register(0)]
}
