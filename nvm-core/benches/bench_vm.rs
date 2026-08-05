// nvm-core/benches/bench_vm.rs
//
// Точка входа бенчмарков ВМ: каждый бенчмарк — отдельный файл в
// каталоге `benches/vm/`.
//
// Все бенчмарки замеряют полный цикл: байты `.nb`-формата формируются
// in-memory как `Vec<u8>`, после чего замеряется загрузка
// (транспиляция байтов в инструкции) + исполнение программы.
mod vm;

use criterion::{Criterion, criterion_group, criterion_main};

criterion_group! {
    name = vm_benches;
    config = Criterion::default();
    targets =
        vm::ackermann::ackermann,
        vm::binary_trees::binary_trees,
        vm::dense_arithmetic_10k::dense_arithmetic_10k,
        vm::fib_loop_10k::fib_loop_10k,
        vm::fib_loop_100k::fib_loop_100k,
        vm::fib_recursive::fib_recursive,
        vm::mandelbrot::mandelbrot,
        vm::nbody::nbody,
        vm::sieve::sieve,
        vm::spectral_norm::spectral_norm,
        vm::tak::tak,
}
criterion_main!(vm_benches);
