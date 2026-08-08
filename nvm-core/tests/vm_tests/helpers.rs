// Helper functions for tests.

use nvm_core::{
    isa::{
        instruction::Instruction,
        operand::{Operand, OperandKind},
        register::Register,
    },
    vm::NVM,
};

// Creating a new register.
pub fn reg(r: u8) -> Operand {
    Operand {
        kind: OperandKind::Register(Register(r)),
    }
}

// Creating an immediate value.
pub fn imm(v: u64) -> Operand {
    Operand {
        kind: OperandKind::Immediate(v),
    }
}

// Run the program on a new VM and return the VM instance.
pub fn run(program: Vec<Instruction>) -> NVM {
    let mut vm = NVM::new(0);
    vm.program = program;
    vm.run().expect("execution failed");
    vm
}

// Run the program on an already prepared VM.
pub fn run_on(mut vm: NVM, program: Vec<Instruction>) -> NVM {
    vm.program = program;
    vm.run().expect("execution failed");
    vm
}

// Run the program on a new VM and return the execution result.
pub fn run_with_result(program: Vec<Instruction>) -> Result<NVM, nvm_core::vm::err::VMError> {
    let mut vm = NVM::new(0);
    vm.program = program;
    vm.run()?;
    Ok(vm)
}
