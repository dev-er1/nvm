// nvm-core/benches/executors/jumptable/mod.rs
//
// Бенчмарки jump table исполнителя ([`nvm_core::vm::jumptable`]).
//
// Каждый файл — отдельная программа бенчмарка. Зеркальные файлы
// находятся в `executors::match_vm`, что позволяет сравнивать
// поведение исполнителей на одном и том же коде.
pub mod dense_arithmetic_10k;
pub mod fib_loop_100k;
pub mod fib_loop_10k;

pub use dense_arithmetic_10k::dense_arithmetic_10k;
pub use fib_loop_10k::fib_loop_10k;
pub use fib_loop_100k::fib_loop_100k;

use nvm_core::{
    isa::instruction::Instruction,
    vm::{NVM, memory::NVMMemory},
};
use std::hint::black_box;

/// Выполняет программу на jump table исполнителе.
pub(crate) fn run(program: Vec<Instruction>, memory_size: usize) {
    let mut vm = NVM::from_program_and_memory(program, NVMMemory::new(memory_size));
    black_box(vm.jumptable_execute()).unwrap();
}
