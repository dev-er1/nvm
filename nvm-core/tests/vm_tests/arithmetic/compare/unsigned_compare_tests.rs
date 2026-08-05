// Тесты на `ULT`, `ULE`, `UGT`, `UGE`.
use nvm_core::isa::opcode::OperationCode;

use crate::vm_tests::arithmetic::compare::get_int;

#[test]
fn ult_less() {
    assert_eq!(get_int(OperationCode::ULT, 5, 10), 1);
}

#[test]
fn ult_equal() {
    assert_eq!(get_int(OperationCode::ULT, 10, 10), 0);
}

#[test]
fn ult_greater() {
    assert_eq!(get_int(OperationCode::ULT, 10, 5), 0);
}

#[test]
fn ult_max_is_greater() {
    assert_eq!(get_int(OperationCode::ULT, u64::MAX, 1), 0);
}

#[test]
fn ult_zero_less_than_max() {
    assert_eq!(get_int(OperationCode::ULT, 0, u64::MAX), 1);
}

#[test]
fn ule_less() {
    assert_eq!(get_int(OperationCode::ULE, 3, 7), 1);
}

#[test]
fn ule_equal() {
    assert_eq!(get_int(OperationCode::ULE, 7, 7), 1);
}

#[test]
fn ule_greater() {
    assert_eq!(get_int(OperationCode::ULE, 100, 1), 0);
}

#[test]
fn ule_max_vs_zero() {
    assert_eq!(get_int(OperationCode::ULE, 0, u64::MAX), 1);
}

#[test]
fn ugt_greater() {
    assert_eq!(get_int(OperationCode::UGT, 100, 1), 1);
}

#[test]
fn ugt_equal() {
    assert_eq!(get_int(OperationCode::UGT, 5, 5), 0);
}

#[test]
fn ugt_less() {
    assert_eq!(get_int(OperationCode::UGT, 1, 100), 0);
}

#[test]
fn ugt_max_beats_all() {
    assert_eq!(get_int(OperationCode::UGT, u64::MAX, 0), 1);
}

#[test]
fn uge_greater() {
    assert_eq!(get_int(OperationCode::UGE, 10, 5), 1);
}

#[test]
fn uge_equal() {
    assert_eq!(get_int(OperationCode::UGE, 10, 10), 1);
}

#[test]
fn uge_less() {
    assert_eq!(get_int(OperationCode::UGE, 0, u64::MAX), 0);
}

#[test]
fn unsigned_register_sources() {
    let mut nvm = nvm_core::vm::NVM::new(0);
    nvm.registers[nvm_core::isa::register::Register(1)] = 5;
    nvm.registers[nvm_core::isa::register::Register(2)] = 10;
    nvm.program = vec![nvm_core::isa::instruction::Instruction {
        opcode: OperationCode::ULT,
        operand1: Some(crate::vm_tests::helpers::reg(0)),
        operand2: Some(crate::vm_tests::helpers::reg(1)),
        operand3: Some(crate::vm_tests::helpers::reg(2)),
    }];
    nvm.run().expect("execution failed");
    assert_eq!(nvm.registers[nvm_core::isa::register::Register(0)], 1);
}
