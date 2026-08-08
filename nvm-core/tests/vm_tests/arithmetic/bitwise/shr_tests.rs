// Tests for `SHR`.
use nvm_core::isa::opcode::OperationCode;

use crate::vm_tests::arithmetic::bitwise::get_result;

#[test]
fn shr_basic() {
    assert_eq!(get_result(OperationCode::SHR, 16, 4), 1);
}

#[test]
fn shr_by_zero() {
    assert_eq!(get_result(OperationCode::SHR, 0x1234, 0), 0x1234);
}

#[test]
fn shr_zero() {
    assert_eq!(get_result(OperationCode::SHR, 0, 10), 0);
}

#[test]
fn shr_logical_not_arithmetic() {
    assert_eq!(get_result(OperationCode::SHR, 0x8000_0000_0000_0000, 63), 1);
}

#[test]
fn shr_lsb_to_zero() {
    assert_eq!(get_result(OperationCode::SHR, 1, 1), 0);
}

#[test]
fn shr_wrap_around() {
    assert_eq!(get_result(OperationCode::SHR, 1, 64), 1);
}

#[test]
fn shr_wrap_large() {
    assert_eq!(get_result(OperationCode::SHR, 1, 128), 1);
}

#[test]
fn shr_lose_low_bits() {
    assert_eq!(
        get_result(OperationCode::SHR, 0xFFFF_FFFF_FFFF_FFFF, 4),
        0x0FFF_FFFF_FFFF_FFFF
    );
}

#[test]
fn shr_high_bit_becomes_zero() {
    assert_eq!(
        get_result(OperationCode::SHR, 0x8000_0000_0000_0000, 1),
        0x4000_0000_0000_0000
    );
}

#[test]
fn shr_registers() {
    let mut nvm = nvm_core::vm::NVM::new(0);
    nvm.registers[nvm_core::isa::register::Register(1)] = 256;
    nvm.registers[nvm_core::isa::register::Register(2)] = 8;

    nvm.program = vec![nvm_core::isa::instruction::Instruction {
        opcode: OperationCode::SHR,
        operand1: Some(crate::vm_tests::helpers::reg(0)),
        operand2: Some(crate::vm_tests::helpers::reg(1)),
        operand3: Some(crate::vm_tests::helpers::reg(2)),
    }];

    nvm.run().expect("execution failed");
    assert_eq!(nvm.registers[nvm_core::isa::register::Register(0)], 1);
}
