// Тесты на дробную арифметику.
pub mod fadd_tests;
pub mod fdiv_tests;
pub mod fmul_tests;
pub mod frem_tests;
pub mod fsub_tests;
pub mod sequence_tests;

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
pub fn get_result(opcode: OperationCode, a: f64, b: f64) -> f64 {
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
    nvm.run().expect("execution failed");
    f64::from_bits(nvm.registers[Register(0)])
}
