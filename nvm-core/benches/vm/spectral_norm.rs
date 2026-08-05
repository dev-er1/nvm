// nvm-core/benches/vm/spectral_norm.rs
//
// Спектральная норма: y = A·x итеративно, 10 проходов (80×80).
// Матрица A(i,j) = 1 / ((i+j)(i+j+1)/2 + i + 1) считается на лету
// только вещественной арифметикой (без int→float).
use criterion::Criterion;

use nvm_core::isa::{instruction::Instruction, opcode::OperationCode as Op};

use super::*;

/// Тело ячейки: `acc += A(i,j) * src[j]`. Регистры r0..r3 — рабочие;
/// `acc` в r3 накапливается снаружи.
fn emit_cell(asm: &mut Asm, i: u64, j: u64, src_base: u64) {
    let fi = i as f64;
    let fj = j as f64;
    asm.push(i3(Op::FADD, reg(0), fimm(fi), fimm(fj)));
    asm.push(i3(Op::FADD, reg(1), reg(0), fimm(1.0)));
    asm.push(i3(Op::FMUL, reg(1), reg(0), reg(1)));
    asm.push(i3(Op::FMUL, reg(1), reg(1), fimm(0.5)));
    asm.push(i3(Op::FADD, reg(1), reg(1), fimm(fi)));
    asm.push(i3(Op::FADD, reg(1), reg(1), fimm(1.0)));
    asm.push(i2(Op::MOVE, reg(2), fimm(1.0)));
    asm.push(i3(Op::FDIV, reg(1), reg(2), reg(1)));
    asm.push(i2(Op::LOAD64, reg(2), imm(src_base + j * 8)));
    asm.push(i3(Op::FMUL, reg(1), reg(1), reg(2)));
    asm.push(i3(Op::FADD, reg(3), reg(3), reg(1)));
}

fn emit_matvec(asm: &mut Asm, n: u64, src_base: u64, dst_base: u64) {
    for i in 0..n {
        asm.push(i2(Op::MOVE, reg(3), fimm(0.0)));
        for j in 0..n {
            emit_cell(asm, i, j, src_base);
        }
        asm.push(i2(Op::STORE64, imm(dst_base + i * 8), reg(3)));
    }
}

fn program(n: u64, iters: u64) -> Vec<Instruction> {
    let vb = 0;
    let wb = n * 8;
    let mut asm = Asm::new();
    for i in 0..n {
        asm.push(i2(Op::MOVE, reg(0), fimm(1.0)));
        asm.push(i2(Op::STORE64, imm(vb + i * 8), reg(0)));
    }
    for _ in 0..iters {
        emit_matvec(&mut asm, n, vb, wb);
        emit_matvec(&mut asm, n, wb, vb);
    }
    asm.push(i0(Op::EXIT));
    asm.finish()
}

pub fn spectral_norm(c: &mut Criterion) {
    let bytes = encode_to_nb(&program(80, 10));
    c.bench_function("nvm/spectral_norm", |b| {
        b.iter(|| load_and_run(&bytes, MEMORY))
    });
}
