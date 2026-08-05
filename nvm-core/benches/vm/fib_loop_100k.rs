// nvm-core/benches/vm/fib_loop_100k.rs
//
// Итеративный Фибоначчи: 100 000 итераций в цикле.
use criterion::Criterion;

use nvm_core::isa::{instruction::Instruction, opcode::OperationCode as Op};

use super::*;

/// Тот же цикл, что и в `fib_loop_10k`, но с большим числом итераций.
fn program(n: u64) -> Vec<Instruction> {
    let mut asm = Asm::new();
    asm.push(i2(Op::MOVE, reg(0), imm(0)));
    asm.push(i2(Op::MOVE, reg(1), imm(1)));
    asm.push(i2(Op::MOVE, reg(2), imm(n)));
    asm.push(i2(Op::MOVE, reg(3), imm(0)));
    asm.label("loop");
    asm.push(i3(Op::ISUB, reg(2), reg(2), imm(1)));
    asm.jz(reg(2), "done");
    asm.push(i2(Op::MOVE, reg(3), reg(1)));
    asm.push(i3(Op::IADD, reg(1), reg(1), reg(0)));
    asm.push(i2(Op::MOVE, reg(0), reg(3)));
    asm.jump("loop");
    asm.label("done");
    asm.push(i0(Op::EXIT));
    asm.finish()
}

pub fn fib_loop_100k(c: &mut Criterion) {
    let bytes = encode_to_nb(&program(100_000));
    c.bench_function("nvm/fib_loop_100k", |b| {
        b.iter(|| load_and_run(&bytes, MEMORY))
    });
}
