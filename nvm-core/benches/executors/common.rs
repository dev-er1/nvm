// nvm-core/benches/executors/common.rs
//
// Общие для бенчмарков исполнителей данные: построение программ.
use nvm_core::isa::{
    instruction::Instruction,
    opcode::OperationCode,
    operand::{Operand, OperandKind},
    register::Register,
};

fn reg(n: u8) -> Operand {
    Operand {
        kind: OperandKind::Register(Register(n)),
    }
}

fn imm(v: u64) -> Operand {
    Operand {
        kind: OperandKind::Immediate(v),
    }
}

fn instr0(opcode: OperationCode) -> Instruction {
    Instruction {
        opcode,
        operand1: None,
        operand2: None,
        operand3: None,
    }
}

fn instr2(opcode: OperationCode, o1: Operand, o2: Operand) -> Instruction {
    Instruction {
        opcode,
        operand1: Some(o1),
        operand2: Some(o2),
        operand3: None,
    }
}

fn instr3(opcode: OperationCode, o1: Operand, o2: Operand, o3: Operand) -> Instruction {
    Instruction {
        opcode,
        operand1: Some(o1),
        operand2: Some(o2),
        operand3: Some(o3),
    }
}

fn move_(dst: u8, src: u64) -> Instruction {
    instr2(OperationCode::MOVE, reg(dst), imm(src))
}

fn exit() -> Instruction {
    instr0(OperationCode::EXIT)
}

/// Цикл с арифметикой, памятью и обратным переходом:
///
/// ```text
/// i = 0 (r0); sum = 0 (r1)
/// loop:
///   mem[i] = sum          (STORE8)
///   sum += mem[i]         (LOAD8, IADD)
///   i++                   (IADD)
///   if i < limit: goto loop  (ULT, JNZ)
/// EXIT
/// ```
///
/// Память нужна размером не меньше `limit` байт (адреса `0..limit`).
pub fn fib_loop_program(limit: u64) -> Vec<Instruction> {
    vec![
        move_(0, 0),
        move_(1, 0),
        instr2(OperationCode::STORE8, reg(0), reg(1)),
        instr2(OperationCode::LOAD8, reg(3), reg(0)),
        instr3(OperationCode::IADD, reg(1), reg(1), reg(3)),
        instr3(OperationCode::IADD, reg(0), reg(0), imm(1)),
        instr3(OperationCode::ULT, reg(2), reg(0), imm(limit)),
        instr2(OperationCode::JNZ, reg(2), imm(2)),
        exit(),
    ]
}

/// "Плотная" линейная программа без переходов: `count` инструкций
/// `IADD r0, r0, 1`, затем `EXIT`.
pub fn dense_arithmetic_program(count: usize) -> Vec<Instruction> {
    let mut program = Vec::with_capacity(count + 1);
    for _ in 0..count {
        program.push(instr3(OperationCode::IADD, reg(0), reg(0), imm(1)));
    }
    program.push(exit());
    program
}
