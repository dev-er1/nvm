// nvm-core/benches/bench_executors.rs
//
// Бенчмарки исполнителей NVM.
//
// Оба исполнителя выполняют одни и те же программы, поэтому результаты
// можно сравнивать напрямую. См. подмодули `executors::match_vm`
// и `executors::jumptable`.
mod executors;

use criterion::{criterion_group, criterion_main};

use executors::{jumptable, match_vm};

criterion_group!(
    match_vm_benches,
    match_vm::fib_loop_10k,
    match_vm::fib_loop_100k,
    match_vm::dense_arithmetic_10k
);

criterion_group!(
    jumptable_benches,
    jumptable::fib_loop_10k,
    jumptable::fib_loop_100k,
    jumptable::dense_arithmetic_10k
);

criterion_main!(match_vm_benches, jumptable_benches);
