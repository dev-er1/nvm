// Tests for `SHL`.
use nvm_core::isa::opcode::OperationCode;

use crate::vm_tests::arithmetic::bitwise::get_result;

#[test]
fn shl_basic() {
    assert_eq!(get_result(OperationCode::SHL, 1, 4), 16);
}

#[test]
fn shl_by_zero() {
    assert_eq!(get_result(OperationCode::SHL, 0x1234, 0), 0x1234);
}

#[test]
fn shl_zero() {
    assert_eq!(get_result(OperationCode::SHL, 0, 10), 0);
}

#[test]
fn shl_to_msb() {
    assert_eq!(get_result(OperationCode::SHL, 1, 63), 1u64 << 63);
}

#[test]
fn shl_wrap_around() {
    assert_eq!(get_result(OperationCode::SHL, 1, 64), 1);
}

#[test]
fn shl_wrap_large() {
    assert_eq!(get_result(OperationCode::SHL, 1, 128), 1);
}

#[test]
fn shl_lose_bits() {
    assert_eq!(
        get_result(OperationCode::SHL, 0xFFFF_FFFF_FFFF_FFFF, 4),
        0xFFFF_FFFF_FFFF_FFF0
    );
}

#[test]
fn shl_max_shift_255() {
    assert_eq!(get_result(OperationCode::SHL, 1, 255), 1 << 63);
}

#[test]
fn shl_registers() {
    let mut nvm = nvm_core::vm::NVM::new(0);
    nvm.registers[nvm_core::isa::register::Register(1)] = 3;
    nvm.registers[nvm_core::isa::register::Register(2)] = 5;

    nvm.program = vec![nvm_core::isa::instruction::Instruction {
        opcode: OperationCode::SHL,
        operand1: Some(crate::vm_tests::helpers::reg(0)),
        operand2: Some(crate::vm_tests::helpers::reg(1)),
        operand3: Some(crate::vm_tests::helpers::reg(2)),
    }];

    nvm.run().expect("execution failed");
    assert_eq!(nvm.registers[nvm_core::isa::register::Register(0)], 96);
}

#[test]
fn shl_shift_by_register() {
    let mut nvm = nvm_core::vm::NVM::new(0);
    nvm.registers[nvm_core::isa::register::Register(0)] = 1;
    nvm.registers[nvm_core::isa::register::Register(1)] = 63;

    nvm.program = vec![nvm_core::isa::instruction::Instruction {
        opcode: OperationCode::SHL,
        operand1: Some(crate::vm_tests::helpers::reg(2)),
        operand2: Some(crate::vm_tests::helpers::reg(0)),
        operand3: Some(crate::vm_tests::helpers::reg(1)),
    }];

    nvm.run().expect("execution failed");
    assert_eq!(
        nvm.registers[nvm_core::isa::register::Register(2)],
        1u64 << 63
    );
}
