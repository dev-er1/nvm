// Вспомогательные функции для тестов.

use nvm_core::{
    isa::{
        instruction::Instruction,
        operand::{Operand, OperandKind},
        register::Register,
    },
    vm::NVM,
};

// Создание нового регистра.
pub fn reg(r: u8) -> Operand {
    Operand {
        kind: OperandKind::Register(Register(r)),
    }
}

// Создание immediate-значения.
pub fn imm(v: u64) -> Operand {
    Operand {
        kind: OperandKind::Immediate(v),
    }
}

// Запустить программу на новой ВМ и вернуть экземпляр ВМ.
pub fn run(program: Vec<Instruction>) -> NVM {
    let mut vm = NVM::new(0);
    vm.program = program;
    vm.run().expect("execution failed");
    vm
}

// Запустить программу на уже подготовленной ВМ.
pub fn run_on(mut vm: NVM, program: Vec<Instruction>) -> NVM {
    vm.program = program;
    vm.run().expect("execution failed");
    vm
}

// Запустить программу на новой ВМ и вернуть результат выполнения.
pub fn run_with_result(program: Vec<Instruction>) -> Result<NVM, nvm_core::vm::err::VMError> {
    let mut vm = NVM::new(0);
    vm.program = program;
    vm.run()?;
    Ok(vm)
}
