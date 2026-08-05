// Sequence tests for bitwise operations.
use nvm_core::{
    isa::{instruction::Instruction, opcode::OperationCode, register::Register},
    vm::{NVM, err::VMErrorKind},
};

use crate::vm_tests::helpers::*;

#[test]
fn bitwise_chain_and_or_xor() {
    let mut nvm = NVM::new(0);
    nvm.registers[Register(1)] = 0xFF00;
    nvm.registers[Register(2)] = 0x0FF0;

    nvm.program = vec![
        Instruction {
            opcode: OperationCode::AND,
            operand1: Some(reg(0)),
            operand2: Some(reg(1)),
            operand3: Some(reg(2)),
        },
        Instruction {
            opcode: OperationCode::OR,
            operand1: Some(reg(0)),
            operand2: Some(reg(0)),
            operand3: Some(imm(0x00FF)),
        },
        Instruction {
            opcode: OperationCode::XOR,
            operand1: Some(reg(0)),
            operand2: Some(reg(0)),
            operand3: Some(imm(0xFFFF)),
        },
    ];

    nvm.run().expect("execution failed");
    assert_eq!(nvm.registers[Register(0)], 0x0FFF ^ 0xFFFF);
}

#[test]
fn bitwise_not_then_and() {
    let mut nvm = NVM::new(0);
    nvm.registers[Register(1)] = 0xFFFF_0000_FFFF_0000;

    nvm.program = vec![
        Instruction {
            opcode: OperationCode::NOT,
            operand1: Some(reg(0)),
            operand2: Some(reg(1)),
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::AND,
            operand1: Some(reg(2)),
            operand2: Some(reg(0)),
            operand3: Some(reg(1)),
        },
    ];

    nvm.run().expect("execution failed");
    assert_eq!(nvm.registers[Register(2)], 0);
}

#[test]
fn bitwise_shift_chain() {
    let mut nvm = NVM::new(0);
    nvm.registers[Register(1)] = 1;

    nvm.program = vec![
        Instruction {
            opcode: OperationCode::SHL,
            operand1: Some(reg(0)),
            operand2: Some(reg(1)),
            operand3: Some(imm(4)),
        },
        Instruction {
            opcode: OperationCode::SAR,
            operand1: Some(reg(0)),
            operand2: Some(reg(0)),
            operand3: Some(imm(2)),
        },
        Instruction {
            opcode: OperationCode::SHR,
            operand1: Some(reg(0)),
            operand2: Some(reg(0)),
            operand3: Some(imm(1)),
        },
    ];

    nvm.run().expect("execution failed");
    assert_eq!(nvm.registers[Register(0)], 2);
}

#[test]
fn bitwise_wrong_operand_count_on_three_op() {
    let err = match run_with_result(vec![Instruction {
        opcode: OperationCode::AND,
        operand1: Some(reg(0)),
        operand2: Some(reg(1)),
        operand3: None,
    }]) {
        Err(err) => err,
        Ok(_) => panic!("expected execution error"),
    };

    assert!(matches!(
        err.kind,
        VMErrorKind::IncorrectNumberOfOperands {
            expected: 3,
            got: 2
        }
    ));
}

#[test]
fn bitwise_immediate_destination_error() {
    let err = match run_with_result(vec![Instruction {
        opcode: OperationCode::XOR,
        operand1: Some(imm(0)),
        operand2: Some(imm(1)),
        operand3: Some(imm(2)),
    }]) {
        Err(err) => err,
        Ok(_) => panic!("expected execution error"),
    };

    assert!(matches!(
        err.kind,
        VMErrorKind::IncorrectTypeOfOperand { .. }
    ));
}

#[test]
fn bitwise_sar_negative_then_not() {
    let mut nvm = NVM::new(0);

    nvm.program = vec![
        Instruction {
            opcode: OperationCode::SAR,
            operand1: Some(reg(0)),
            operand2: Some(imm(0x8000_0000_0000_0000)),
            operand3: Some(imm(63)),
        },
        Instruction {
            opcode: OperationCode::NOT,
            operand1: Some(reg(1)),
            operand2: Some(reg(0)),
            operand3: None,
        },
    ];

    nvm.run().expect("execution failed");
    assert_eq!(nvm.registers[Register(0)], u64::MAX);
    assert_eq!(nvm.registers[Register(1)], 0);
}

#[test]
fn bitwise_xor_triple() {
    let mut nvm = NVM::new(0);
    nvm.registers[Register(1)] = 0xAAAA_AAAA_AAAA_AAAA;
    nvm.registers[Register(2)] = 0xBBBB_BBBB_BBBB_BBBB;
    nvm.registers[Register(3)] = 0xCCCC_CCCC_CCCC_CCCC;

    nvm.program = vec![
        Instruction {
            opcode: OperationCode::XOR,
            operand1: Some(reg(0)),
            operand2: Some(reg(1)),
            operand3: Some(reg(2)),
        },
        Instruction {
            opcode: OperationCode::XOR,
            operand1: Some(reg(0)),
            operand2: Some(reg(0)),
            operand3: Some(reg(3)),
        },
        Instruction {
            opcode: OperationCode::XOR,
            operand1: Some(reg(0)),
            operand2: Some(reg(0)),
            operand3: Some(reg(1)),
        },
        Instruction {
            opcode: OperationCode::XOR,
            operand1: Some(reg(0)),
            operand2: Some(reg(0)),
            operand3: Some(reg(2)),
        },
    ];

    nvm.run().expect("execution failed");
    assert_eq!(nvm.registers[Register(0)], 0xCCCC_CCCC_CCCC_CCCC);
}
