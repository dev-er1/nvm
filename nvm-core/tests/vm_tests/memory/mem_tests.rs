// Test for VM memory.
use nvm_core::{
    isa::{instruction::Instruction, opcode::OperationCode, register::Register},
    vm::NVM,
};

use crate::vm_tests::helpers::{imm, reg, run_on};

#[test]
fn nothing_after_exit() {
    let vm = NVM::new(0);

    let vm = run_on(
        vm,
        vec![
            Instruction {
                opcode: OperationCode::MOVE,
                operand1: Some(reg(0)),
                operand2: Some(imm(67)),
                operand3: None,
            },
            Instruction {
                opcode: OperationCode::EXIT,
                operand1: None,
                operand2: None,
                operand3: None,
            },
            Instruction {
                opcode: OperationCode::IADD,
                operand1: Some(reg(0)),
                operand2: Some(reg(0)),
                operand3: Some(imm(1)),
            },
        ],
    );

    assert_eq!(vm.registers[Register(0)], 67);
}
