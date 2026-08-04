// nvm-core/benches/executors/match_vm/dense_arithmetic_10k.rs
//
// Программа: линейный код без переходов — 10 000 инструкций
// `IADD`. Зеркальный файл — `jumptable::dense_arithmetic_10k`.
use criterion::Criterion;

use super::run;
use crate::executors::common::dense_arithmetic_program;

pub fn dense_arithmetic_10k(c: &mut Criterion) {
    let program = dense_arithmetic_program(10_000);
    c.bench_function("match/dense_arithmetic_10k", |b| {
        b.iter(|| run(program.clone(), 1));
    });
}
