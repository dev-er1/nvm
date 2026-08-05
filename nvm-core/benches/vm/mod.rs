// nvm-core/benches/vm/mod.rs
//
// Общие компоненты бенчмарков ВМ: построение программ, кодирование
// в формат NVM Bytecode (`Vec<u8>`) и запуск (загрузка + исполнение).
//
// Каждый бенчмарк — отдельный файл в этом каталоге. Бенчмарк строится
// как программа из `Instruction`, кодируется в байты `.nb`-формата
// (без файлов — просто вектор), после чего замеряется полный цикл:
// загрузка (транспиляция байтов в инструкции) + исполнение.
use std::{collections::HashMap, hint::black_box};

use nvm_core::{
    NVM_VERSION,
    isa::{
        instruction::Instruction,
        opcode::OperationCode,
        operand::{Operand, OperandKind},
        register::Register,
    },
    loader::NVMLoader,
    vm::{NVM, memory::NVMMemory},
};

// ====== Постоянные для всех бенчмарков ======

/// Номер регистра-указателя стека значений (используется рекурсивными бенчмарками).
pub const SP: u8 = 14;

/// Размер памяти ВМ для бенчмарков (4 МиБ — хватает ситу sieve 1 000 000).
pub const MEMORY: usize = 1 << 22;

// ====== Бенчмарки (отдельный файл на каждый) ======

pub mod ackermann;
pub mod binary_trees;
pub mod dense_arithmetic_10k;
pub mod fib_loop_100k;
pub mod fib_loop_10k;
pub mod fib_recursive;
pub mod mandelbrot;
pub mod nbody;
pub mod sieve;
pub mod spectral_norm;
pub mod tak;

// ====== Операнды и инструкции ======

/// Операнд-регистр.
pub fn reg(n: u8) -> Operand {
    Operand {
        kind: OperandKind::Register(Register(n)),
    }
}

/// Операнд-immediate.
pub fn imm(v: u64) -> Operand {
    Operand {
        kind: OperandKind::Immediate(v),
    }
}

/// Immediate из битов `f64` (константы с плавающей точкой).
pub fn fimm(v: f64) -> Operand {
    imm(v.to_bits())
}

pub fn i0(op: OperationCode) -> Instruction {
    Instruction {
        opcode: op,
        operand1: None,
        operand2: None,
        operand3: None,
    }
}

pub fn i1(op: OperationCode, o1: Operand) -> Instruction {
    Instruction {
        opcode: op,
        operand1: Some(o1),
        operand2: None,
        operand3: None,
    }
}

pub fn i2(op: OperationCode, o1: Operand, o2: Operand) -> Instruction {
    Instruction {
        opcode: op,
        operand1: Some(o1),
        operand2: Some(o2),
        operand3: None,
    }
}

pub fn i3(op: OperationCode, o1: Operand, o2: Operand, o3: Operand) -> Instruction {
    Instruction {
        opcode: op,
        operand1: Some(o1),
        operand2: Some(o2),
        operand3: Some(o3),
    }
}

// ====== Сборка программ с метками ======

/// Сборщик программы с именованными метками для переходов.
///
/// `jump`/`call`/`jz`/`jnz` принимают имя метки; финальные адреса
/// переходов вычисляются в [`Asm::finish`].
pub struct Asm {
    code: Vec<Instruction>,
    labels: HashMap<String, usize>,
    fixups: Vec<(usize, String, u8)>,
}

impl Asm {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            labels: HashMap::new(),
            fixups: Vec::new(),
        }
    }

    /// Помечает текущую позицию именем.
    pub fn label(&mut self, name: impl Into<String>) {
        self.labels.insert(name.into(), self.code.len());
    }

    /// Добавляет инструкцию и возвращает её индекс.
    pub fn push(&mut self, instr: Instruction) -> usize {
        self.code.push(instr);
        self.code.len() - 1
    }

    /// `JMP <label>`.
    pub fn jump(&mut self, label: &str) {
        let idx = self.push(i1(OperationCode::JMP, imm(0)));
        self.fixups.push((idx, label.to_string(), 1));
    }

    /// `CALL <label>`.
    pub fn call(&mut self, label: &str) {
        let idx = self.push(i1(OperationCode::CALL, imm(0)));
        self.fixups.push((idx, label.to_string(), 1));
    }

    /// `JZ <cond>, <label>`.
    pub fn jz(&mut self, cond: Operand, label: &str) {
        let idx = self.push(i2(OperationCode::JZ, cond, imm(0)));
        self.fixups.push((idx, label.to_string(), 2));
    }

    /// `JNZ <cond>, <label>`.
    pub fn jnz(&mut self, cond: Operand, label: &str) {
        let idx = self.push(i2(OperationCode::JNZ, cond, imm(0)));
        self.fixups.push((idx, label.to_string(), 2));
    }

    /// Разрешает переходы и возвращает программу.
    pub fn finish(mut self) -> Vec<Instruction> {
        for (idx, label, slot) in self.fixups {
            let target = *self
                .labels
                .get(&label)
                .unwrap_or_else(|| panic!("unknown label `{label}`"))
                as u64;
            match slot {
                1 => self.code[idx].operand1 = Some(imm(target)),
                2 => self.code[idx].operand2 = Some(imm(target)),
                _ => unreachable!(),
            }
        }
        self.code
    }
}

// ====== Кодирование и запуск ======

/// Кодирует программу в байты формата NVM Bytecode (`Vec<u8>`).
pub fn encode_to_nb(program: &[Instruction]) -> Vec<u8> {
    let mut bytes = Vec::new();

    // Magic + минимальная версия NVM.
    bytes.extend_from_slice(b"NVMBC");
    for part in NVM_VERSION.split('.').map(|p| p.parse::<u16>().unwrap()) {
        bytes.extend_from_slice(&part.to_le_bytes());
    }

    for instr in program {
        bytes.push(instr.opcode as u8);
        bytes.push(instr.operand_count() as u8);
        for operand in [instr.operand1, instr.operand2, instr.operand3]
            .into_iter()
            .flatten()
        {
            match operand.kind {
                OperandKind::Register(Register(r)) => {
                    bytes.push(0x00);
                    bytes.push(r);
                }
                OperandKind::Immediate(v) => {
                    bytes.push(0x01);
                    bytes.extend_from_slice(&v.to_le_bytes());
                }
            }
        }
    }

    bytes
}

/// Загружает байты в формате `.nb` (транспиляция в инструкции) и исполняет программу.
/// Возвращает ВМ с результатом.
pub fn load_and_run_vm(bytes: &[u8], memory_size: usize) -> NVM {
    let instructions = NVMLoader::new(bytes.to_vec())
        .transpile()
        .expect("failed to load benchmark bytecode");
    let mut vm = NVM::from_program_and_memory(instructions, NVMMemory::new(memory_size));
    vm.run().expect("benchmark program failed");
    vm
}

/// Загружает байты в формате`.nb` и исполняет программу (замеряемая часть).
pub fn load_and_run(bytes: &[u8], memory_size: usize) {
    let vm = load_and_run_vm(bytes, memory_size);
    black_box(vm);
}
