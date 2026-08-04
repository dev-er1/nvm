// nvm-core/benches/executors/match_vm/fib_loop_10k.rs
//
// Программа: цикл на 10 000 итераций (арифметика + память +
// обратный переход). Зеркальный файл — `jumptable::fib_loop_10k`.
use criterion::Criterion;

use super::run;
use crate::executors::common::fib_loop_program;

pub fn fib_loop_10k(c: &mut Criterion) {
    let program = fib_loop_program(10_000);
    c.bench_function("match/fib_loop_10k", |b| {
        b.iter(|| run(program.clone(), 10_000));
    });
}
