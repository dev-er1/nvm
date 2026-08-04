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
    vm.match_execute().expect("execution failed");
    vm
}

// Запустить программу на уже подготовленной ВМ.
pub fn run_on(mut vm: NVM, program: Vec<Instruction>) -> NVM {
    vm.program = program;
    vm.match_execute().expect("execution failed");
    vm
}

// Запустить программу на новой ВМ и вернуть результат выполнения.
pub fn run_with_result(program: Vec<Instruction>) -> Result<NVM, nvm_core::vm::err::VMError> {
    let mut vm = NVM::new(0);
    vm.program = program;
    vm.match_execute()?;
    Ok(vm)
}

// ====== Jump table ======

// Запустить программу через jump table на новой ВМ и вернуть экземпляр ВМ.
pub fn run_jt(program: Vec<Instruction>) -> NVM {
    let mut vm = NVM::new(0);
    vm.program = program;
    vm.jumptable_execute().expect("execution failed");
    vm
}

// Запустить программу через jump table на уже подготовленной ВМ.
pub fn run_jt_on(mut vm: NVM, program: Vec<Instruction>) -> NVM {
    vm.program = program;
    vm.jumptable_execute().expect("execution failed");
    vm
}

// Запустить программу через jump table и вернуть результат выполнения.
pub fn run_jt_with_result(program: Vec<Instruction>) -> Result<NVM, nvm_core::vm::err::VMError> {
    let mut vm = NVM::new(0);
    vm.program = program;
    vm.jumptable_execute()?;
    Ok(vm)
}

// Сравнить состояния двух ВМ (регистры, стек вызовов, память).
pub fn assert_same_state(a: &NVM, b: &NVM) {
    for i in 0..255 {
        assert_eq!(
            a.registers[nvm_core::isa::register::Register(i as u8)],
            b.registers[nvm_core::isa::register::Register(i as u8)],
            "register {i} differs"
        );
    }
    assert_eq!(a.call_stack, b.call_stack, "call stacks differ");
    assert_eq!(a.memory, b.memory, "memories differ");
}
