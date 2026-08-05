// Тесты на сравнения.
use nvm_core::{
    isa::{instruction::Instruction, opcode::OperationCode, register::Register},
    vm::{NVM, err::VMErrorKind},
};

use crate::vm_tests::helpers::*;

#[test]
fn compare_wrong_operand_count() {
    let err = match run_with_result(vec![Instruction {
        opcode: OperationCode::IEQ,
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
fn compare_immediate_destination() {
    let err = match run_with_result(vec![Instruction {
        opcode: OperationCode::SLT,
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
fn compare_chain_ieq_slt_uge() {
    let mut nvm = NVM::new(0);
    nvm.registers[Register(1)] = 10;
    nvm.registers[Register(2)] = 20;

    nvm.program = vec![
        Instruction {
            opcode: OperationCode::IEQ,
            operand1: Some(reg(0)),
            operand2: Some(reg(1)),
            operand3: Some(reg(2)),
        },
        Instruction {
            opcode: OperationCode::SLT,
            operand1: Some(reg(3)),
            operand2: Some(reg(1)),
            operand3: Some(reg(2)),
        },
        Instruction {
            opcode: OperationCode::UGE,
            operand1: Some(reg(4)),
            operand2: Some(reg(2)),
            operand3: Some(reg(1)),
        },
    ];

    nvm.run().expect("execution failed");
    assert_eq!(nvm.registers[Register(0)], 0);
    assert_eq!(nvm.registers[Register(3)], 1);
    assert_eq!(nvm.registers[Register(4)], 1);
}

#[test]
fn compare_float_chain() {
    let mut nvm = NVM::new(0);
    let a = 1.5f64;
    let b = 2.5f64;

    nvm.registers[Register(1)] = a.to_bits();
    nvm.registers[Register(2)] = b.to_bits();

    nvm.program = vec![
        Instruction {
            opcode: OperationCode::FEQ,
            operand1: Some(reg(0)),
            operand2: Some(reg(1)),
            operand3: Some(reg(2)),
        },
        Instruction {
            opcode: OperationCode::FLT,
            operand1: Some(reg(3)),
            operand2: Some(reg(1)),
            operand3: Some(reg(2)),
        },
        Instruction {
            opcode: OperationCode::FGT,
            operand1: Some(reg(4)),
            operand2: Some(reg(2)),
            operand3: Some(reg(1)),
        },
    ];

    nvm.run().expect("execution failed");
    assert_eq!(nvm.registers[Register(0)], 0);
    assert_eq!(nvm.registers[Register(3)], 1);
    assert_eq!(nvm.registers[Register(4)], 1);
}

#[test]
fn compare_mixed_int_and_float() {
    let mut nvm = NVM::new(0);

    nvm.program = vec![
        Instruction {
            opcode: OperationCode::IEQ,
            operand1: Some(reg(0)),
            operand2: Some(imm(0)),
            operand3: Some(imm(0)),
        },
        Instruction {
            opcode: OperationCode::FEQ,
            operand1: Some(reg(1)),
            operand2: Some(imm(f64::NAN.to_bits())),
            operand3: Some(imm(f64::NAN.to_bits())),
        },
    ];

    nvm.run().expect("execution failed");
    assert_eq!(nvm.registers[Register(0)], 1);
    assert_eq!(nvm.registers[Register(1)], 0);
}
