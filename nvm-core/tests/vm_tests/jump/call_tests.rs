// Тесты на `CALL`.
use nvm_core::{
    isa::{instruction::Instruction, opcode::OperationCode, register::Register},
    vm::NVM,
};

use crate::vm_tests::helpers::*;

#[test]
fn call_jumps_to_subroutine() {
    let vm = run(vec![
        Instruction {
            opcode: OperationCode::CALL,
            operand1: Some(imm(2)),
            operand2: None,
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::MOVE,
            operand1: Some(reg(0)),
            operand2: Some(imm(1)),
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::MOVE,
            operand1: Some(reg(1)),
            operand2: Some(imm(42)),
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::MOVE,
            operand1: Some(reg(2)),
            operand2: Some(imm(99)),
            operand3: None,
        },
    ]);
    assert_eq!(vm.registers[Register(0)], 0);
    assert_eq!(vm.registers[Register(1)], 42);
    assert_eq!(vm.registers[Register(2)], 99);
}

#[test]
fn call_with_register_address() {
    let mut vm = NVM::new(0);
    vm.registers[Register(0)] = 2;

    let vm = run_on(
        vm,
        vec![
            Instruction {
                opcode: OperationCode::CALL,
                operand1: Some(reg(0)),
                operand2: None,
                operand3: None,
            },
            Instruction {
                opcode: OperationCode::MOVE,
                operand1: Some(reg(1)),
                operand2: Some(imm(1)),
                operand3: None,
            },
            Instruction {
                opcode: OperationCode::MOVE,
                operand1: Some(reg(2)),
                operand2: Some(imm(77)),
                operand3: None,
            },
        ],
    );
    assert_eq!(vm.registers[Register(1)], 0);
    assert_eq!(vm.registers[Register(2)], 77);
}

#[test]
fn call_pushes_return_address_onto_call_stack() {
    let mut vm = NVM::new(0);
    vm.call_stack.push(99);

    let vm = run_on(
        vm,
        vec![
            Instruction {
                opcode: OperationCode::CALL,
                operand1: Some(imm(3)),
                operand2: None,
                operand3: None,
            },
            Instruction {
                opcode: OperationCode::MOVE,
                operand1: Some(reg(0)),
                operand2: Some(imm(1)),
                operand3: None,
            },
            Instruction {
                opcode: OperationCode::MOVE,
                operand1: Some(reg(1)),
                operand2: Some(imm(2)),
                operand3: None,
            },
        ],
    );
    assert_eq!(vm.call_stack.len(), 2);
    assert_eq!(vm.call_stack[0], 99);
    assert_eq!(vm.call_stack[1], 1);
}

#[test]
fn call_and_ret_round_trip() {
    let vm = run(vec![
        Instruction {
            opcode: OperationCode::MOVE,
            operand1: Some(reg(0)),
            operand2: Some(imm(21)),
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::CALL,
            operand1: Some(imm(4)),
            operand2: None,
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::IADD,
            operand1: Some(reg(0)),
            operand2: Some(reg(0)),
            operand3: Some(reg(0)),
        },
        Instruction {
            opcode: OperationCode::EXIT,
            operand1: None,
            operand2: None,
            operand3: None,
        },
        // Подпрограмма: r0 += r0; RET.
        Instruction {
            opcode: OperationCode::IADD,
            operand1: Some(reg(0)),
            operand2: Some(reg(0)),
            operand3: Some(reg(0)),
        },
        Instruction {
            opcode: OperationCode::RET,
            operand1: None,
            operand2: None,
            operand3: None,
        },
    ]);
    // 21 -> CALL -> 42 -> RET -> 84 -> EXIT.
    assert_eq!(vm.registers[Register(0)], 84);
}

#[test]
fn call_skip_instructions_after_call() {
    let vm = run(vec![
        Instruction {
            opcode: OperationCode::CALL,
            operand1: Some(imm(2)),
            operand2: None,
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::MOVE,
            operand1: Some(reg(0)),
            operand2: Some(imm(10)),
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::EXIT,
            operand1: None,
            operand2: None,
            operand3: None,
        },
    ]);
    assert_eq!(vm.registers[Register(0)], 0);
}
