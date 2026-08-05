// Тесты на целочисленную арифметику.
pub mod iadd_tests;
pub mod imul_tests;
pub mod isub_tests;
pub mod sdiv_tests;
pub mod sequence_tests;
pub mod srem_tests;
pub mod udiv_tests;
pub mod urem_tests;

use nvm_core::{
    isa::{
        instruction::Instruction,
        opcode::OperationCode,
        operand::{Operand, OperandKind},
        register::Register,
    },
    vm::NVM,
};

// Вспомогательная функция для тестирования.
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
    nvm.run().expect("d");
    nvm.registers[Register(0)]
}
