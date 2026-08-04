// nvm-core/src/vm/memory.rs
//
//! # Память NVM
//!
//! В этом модуле определена память виртуальной машины NVM.
//!
//! Память NVM представляет собой непрерывную последовательность байт.
//! Каждая ячейка памяти имеет собственный адрес, начиная с `0`.
//!
//! Доступ к памяти осуществляется через инструкции семейства
//! `LOAD*` и `STORE*`.
//!
//! ## Представление памяти
//!
//! Внутри виртуальной машины память хранится в виде массива байт:
//!
//! ```text
//! Адрес:   0    1    2    3    ...
//!        +----+----+----+----+
//! Данные | 12 | FF | 00 | A5 |
//!        +----+----+----+----+
//! ```
//!
//! Такой подход позволяет хранить значения любого размера
//! (`u8`, `u16`, `u32`, `u64` и т.д.).
use std::ops::Index;

use crate::isa::register::Register;

/// Память виртуальной машины.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NVMMemory {
    /// Последовательность байт памяти.
    data: Vec<u8>,
}

impl NVMMemory {
    /// Создаёт память указанного размера.
    ///
    /// Все байты инициализируются нулём.
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0; size],
        }
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    // ================== load_* ==================

    /// Загружает 8-битное беззнаковое значение из памяти.
    pub fn load_u8(&self, address: usize) -> Option<u8> {
        self.data.get(address).copied()
    }

    pub fn load_u16(&self, address: usize) -> Option<u16> {
        Some(u16::from_le_bytes(
            self.data.get(address..address + 2)?.try_into().unwrap(),
        ))
    }

    pub fn load_u32(&self, address: usize) -> Option<u32> {
        Some(u32::from_le_bytes(
            self.data.get(address..address + 4)?.try_into().unwrap(),
        ))
    }

    pub fn load_u64(&self, address: usize) -> Option<u64> {
        Some(u64::from_le_bytes(
            self.data.get(address..address + 8)?.try_into().unwrap(),
        ))
    }

    /// Загружает 8-битное знаковое значение из памяти.
    pub fn load_i8(&self, address: usize) -> Option<i8> {
        Some(*self.data.get(address)? as i8)
    }

    pub fn load_i16(&self, address: usize) -> Option<i16> {
        Some(i16::from_le_bytes(
            self.data.get(address..address + 2)?.try_into().unwrap(),
        ))
    }

    pub fn load_i32(&self, address: usize) -> Option<i32> {
        Some(i32::from_le_bytes(
            self.data.get(address..address + 4)?.try_into().unwrap(),
        ))
    }

    pub fn load_i64(&self, address: usize) -> Option<i64> {
        Some(i64::from_le_bytes(
            self.data.get(address..address + 8)?.try_into().unwrap(),
        ))
    }

    /// Загружает 32-битное дробное значение из памяти.
    pub fn load_f32(&self, address: usize) -> Option<f32> {
        Some(f32::from_le_bytes(
            self.data.get(address..address + 4)?.try_into().unwrap(),
        ))
    }

    pub fn load_f64(&self, address: usize) -> Option<f64> {
        Some(f64::from_le_bytes(
            self.data.get(address..address + 8)?.try_into().unwrap(),
        ))
    }

    // ================== store_* ==================

    /// Записывает 8-битное беззнаковое значение в память.
    pub fn store_u8(&mut self, address: usize, value: u8) -> Option<()> {
        *self.data.get_mut(address)? = value;
        Some(())
    }

    pub fn store_u16(&mut self, address: usize, value: u16) -> Option<()> {
        self.data
            .get_mut(address..address + 2)?
            .copy_from_slice(&value.to_le_bytes());

        Some(())
    }

    pub fn store_u32(&mut self, address: usize, value: u32) -> Option<()> {
        self.data
            .get_mut(address..address + 4)?
            .copy_from_slice(&value.to_le_bytes());

        Some(())
    }

    pub fn store_u64(&mut self, address: usize, value: u64) -> Option<()> {
        self.data
            .get_mut(address..address + 8)?
            .copy_from_slice(&value.to_le_bytes());

        Some(())
    }

    /// Записывает 8-битное знаковое значение в память.
    pub fn store_i8(&mut self, address: usize, value: i8) -> Option<()> {
        self.store_u8(address, value as u8)
    }

    pub fn store_i16(&mut self, address: usize, value: i16) -> Option<()> {
        self.data
            .get_mut(address..address + 2)?
            .copy_from_slice(&value.to_le_bytes());

        Some(())
    }

    pub fn store_i32(&mut self, address: usize, value: i32) -> Option<()> {
        self.data
            .get_mut(address..address + 4)?
            .copy_from_slice(&value.to_le_bytes());

        Some(())
    }

    pub fn store_i64(&mut self, address: usize, value: i64) -> Option<()> {
        self.data
            .get_mut(address..address + 8)?
            .copy_from_slice(&value.to_le_bytes());

        Some(())
    }

    /// Записывает 32-битное число с плавающей точкой в память.
    pub fn store_f32(&mut self, address: usize, value: f32) -> Option<()> {
        self.data
            .get_mut(address..address + 4)?
            .copy_from_slice(&value.to_le_bytes());

        Some(())
    }

    /// Записывает 64-битное число с плавающей точкой в память.
    pub fn store_f64(&mut self, address: usize, value: f64) -> Option<()> {
        self.data
            .get_mut(address..address + 8)?
            .copy_from_slice(&value.to_le_bytes());

        Some(())
    }
}

// Реализация Index позволяет обращаться к регистрам
// через оператор индексации:
//
// ```
// let value = memory[Register(0)];
// ```
impl Index<Register> for NVMMemory {
    type Output = u8;

    fn index(&self, index: Register) -> &Self::Output {
        &self.data[index.0 as usize]
    }
}
