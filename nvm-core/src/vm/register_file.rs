// nvm-core/src/vm/register_file.rs
//
//! # Хранилище регистров
//!
//! В этом модуле определено хранилище регистров виртуальной машины NVM.
//!
//! Каждый регистр хранит одно значение типа [`u64`].
use std::{
    array,
    ops::{Index, IndexMut},
};

use crate::isa::register::Register;

pub struct RegisterFile {
    registers: [u64; 255],
}

impl RegisterFile {
    pub fn new() -> Self {
        Self {
            registers: array::from_fn(|_| u64::default()),
        }
    }

    /// Создаёт хранилище регистров, используя указанный массив значений.
    pub fn from_registers(registers: [u64; 255]) -> Self {
        Self { registers }
    }
}

// Реализация трейтов для `RegisterFile`

impl Default for RegisterFile {
    fn default() -> Self {
        Self::new()
    }
}

// Реализация Index позволяет обращаться к регистрам
// через оператор индексации:
//
// ```
// let value = registers[Register(0)];
// ```
impl Index<Register> for RegisterFile {
    type Output = u64;

    fn index(&self, index: Register) -> &Self::Output {
        &self.registers[index.0 as usize]
    }
}

// Реализация IndexMut позволяет изменять значения регистров
// через оператор индексации:
//
// ```
// registers[Register(0)] = Value::default();
// ```
impl IndexMut<Register> for RegisterFile {
    fn index_mut(&mut self, index: Register) -> &mut Self::Output {
        &mut self.registers[index.0 as usize]
    }
}
