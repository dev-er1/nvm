// nvm-core/benches/vm/mandelbrot.rs
//
// Множество Мандельброта 200×200, до 64 итераций на точку:
// вещественный цикл с условным выходом (FGT/JNZ) и запись байтами.
use criterion::Criterion;

use nvm_core::isa::{instruction::Instruction, opcode::OperationCode as Op};

use super::*;

fn program(width: u64, height: u64, iters: u64) -> Vec<Instruction> {
    let mut asm = Asm::new();
    let mut n = 0u64;
    for yy in 0..height {
        for xx in 0..width {
            let cr = -2.0 + xx as f64 * 3.0 / width as f64;
            let ci = -1.0 + yy as f64 * 2.0 / height as f64;
            asm.push(i2(Op::MOVE, reg(0), fimm(0.0)));
            asm.push(i2(Op::MOVE, reg(1), fimm(0.0))); 
            asm.push(i2(Op::MOVE, reg(2), fimm(cr)));
            asm.push(i2(Op::MOVE, reg(3), fimm(ci)));
            asm.push(i2(Op::MOVE, reg(4), fimm(4.0)));
            asm.push(i2(Op::MOVE, reg(5), imm(0)));
            let loop_lbl = n;
            let done_lbl = n + 1;
            n += 2;
            asm.label(format!("mb{loop_lbl}_loop"));
            asm.push(i3(Op::FMUL, reg(6), reg(0), reg(0)));
            asm.push(i3(Op::FMUL, reg(7), reg(1), reg(1)));
            asm.push(i3(Op::FSUB, reg(6), reg(6), reg(7)));
            asm.push(i3(Op::FADD, reg(6), reg(6), reg(2)));
            asm.push(i3(Op::FMUL, reg(7), reg(0), reg(1)));
            asm.push(i3(Op::FMUL, reg(7), reg(7), fimm(2.0)));
            asm.push(i3(Op::FADD, reg(7), reg(7), reg(3)));
            asm.push(i2(Op::MOVE, reg(0), reg(6)));
            asm.push(i2(Op::MOVE, reg(1), reg(7)));
            asm.push(i3(Op::FMUL, reg(6), reg(0), reg(0)));
            asm.push(i3(Op::FMUL, reg(7), reg(1), reg(1)));
            asm.push(i3(Op::FADD, reg(6), reg(6), reg(7)));
            asm.push(i3(Op::FGT, reg(6), reg(6), reg(4)));
            asm.jnz(reg(6), &format!("mb{done_lbl}_done"));
            asm.push(i3(Op::IADD, reg(5), reg(5), imm(1)));
            asm.push(i3(Op::ULT, reg(6), reg(5), imm(iters)));
            asm.jnz(reg(6), &format!("mb{loop_lbl}_loop"));
            asm.label(format!("mb{done_lbl}_done"));
            asm.push(i2(Op::STORE8, imm(yy * width + xx), reg(5)));
        }
    }
    asm.push(i0(Op::EXIT));
    asm.finish()
}

pub fn mandelbrot(c: &mut Criterion) {
    let bytes = encode_to_nb(&program(200, 200, 64));
    c.bench_function("nvm/mandelbrot", |b| {
        b.iter(|| load_and_run(&bytes, MEMORY))
    });
}
