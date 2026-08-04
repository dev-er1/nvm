// nvm-core/benches/executors/match_vm/fib_loop_100k.rs
//
// Программа: цикл на 100 000 итераций (арифметика + память +
// обратный переход). Зеркальный файл — `jumptable::fib_loop_100k`.
use criterion::Criterion;

use super::run;
use crate::executors::common::fib_loop_program;

pub fn fib_loop_100k(c: &mut Criterion) {
    let program = fib_loop_program(100_000);
    c.bench_function("match/fib_loop_100k", |b| {
        b.iter(|| run(program.clone(), 100_000));
    });
}
