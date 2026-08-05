// nvm-core/benches/vm/fib_recursive.rs
//
// Рекурсивный Фибоначчи (fib(22)) со стеком значений в памяти и CALL/RET.
use criterion::Criterion;

use nvm_core::isa::{instruction::Instruction, opcode::OperationCode as Op};

use super::*;

/// `fib(n)` в `r0`; база — `n < 2`; временно сохраняет `n` на стеке.
fn program(n: u64) -> Vec<Instruction> {
    let mut asm = Asm::new();
    asm.jump("main");
    asm.label("fib");
    asm.push(i3(Op::SLT, reg(1), reg(0), imm(2)));
    asm.jnz(reg(1), "base_fib");
    asm.push(i2(Op::STORE64, reg(SP), reg(0)));
    asm.push(i3(Op::IADD, reg(SP), reg(SP), imm(8)));
    asm.push(i3(Op::ISUB, reg(1), reg(0), imm(1)));
    asm.push(i2(Op::MOVE, reg(0), reg(1)));
    asm.call("fib");
    asm.push(i3(Op::ISUB, reg(SP), reg(SP), imm(8)));
    asm.push(i2(Op::LOAD64, reg(1), reg(SP)));
    asm.push(i2(Op::STORE64, reg(SP), reg(0)));
    asm.push(i3(Op::IADD, reg(SP), reg(SP), imm(8)));
    asm.push(i3(Op::ISUB, reg(1), reg(1), imm(1)));
    asm.push(i2(Op::MOVE, reg(0), reg(1)));
    asm.call("fib");
    asm.push(i3(Op::ISUB, reg(SP), reg(SP), imm(8)));
    asm.push(i2(Op::LOAD64, reg(1), reg(SP)));
    asm.push(i3(Op::IADD, reg(0), reg(0), reg(1)));
    asm.push(i0(Op::RET));
    asm.label("base_fib");
    asm.push(i0(Op::RET));
    asm.label("main");
    asm.push(i2(Op::MOVE, reg(SP), imm(0)));
    asm.push(i2(Op::MOVE, reg(0), imm(n)));
    asm.call("fib");
    asm.push(i0(Op::EXIT));
    asm.finish()
}

pub fn fib_recursive(c: &mut Criterion) {
    let bytes = encode_to_nb(&program(22));
    c.bench_function("nvm/fib_recursive", |b| {
        b.iter(|| load_and_run(&bytes, MEMORY))
    });
}
