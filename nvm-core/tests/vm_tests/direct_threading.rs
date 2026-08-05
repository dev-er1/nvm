// Тесты на direct threading исполнитель.
use nvm_core::{
    isa::{
        instruction::Instruction,
        opcode::OperationCode,
        operand::{Operand, OperandKind},
        register::Register,
    },
    vm::{NVM, err::VMErrorKind},
};

use crate::vm_tests::helpers::*;

// Запустить программу через direct threading на новой ВМ и вернуть экземпляр ВМ.
fn run_dt(program: Vec<Instruction>) -> NVM {
    let mut vm = NVM::new(0);
    vm.program = program;
    vm.direct_threading_execute().expect("execution failed");
    vm
}

// Запустить программу через direct threading и вернуть результат выполнения.
fn run_dt_with_result(program: Vec<Instruction>) -> Result<NVM, nvm_core::vm::err::VMError> {
    let mut vm = NVM::new(0);
    vm.program = program;
    vm.direct_threading_execute()?;
    Ok(vm)
}

// ====== Базовое исполнение ======

#[test]
fn exit_stops_execution() {
    let vm = run_dt(vec![
        Instruction {
            opcode: OperationCode::EXIT,
            operand1: None,
            operand2: None,
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::MOVE,
            operand1: Some(reg(0)),
            operand2: Some(imm(42)),
            operand3: None,
        },
    ]);
    assert_eq!(vm.registers[Register(0)], 0);
}

#[test]
fn move_works() {
    let vm = run_dt(vec![Instruction {
        opcode: OperationCode::MOVE,
        operand1: Some(reg(0)),
        operand2: Some(imm(42)),
        operand3: None,
    }]);
    assert_eq!(vm.registers[Register(0)], 42);
}

#[test]
fn empty_program_is_ok() {
    run_dt(vec![]);
}

// ====== Арифметика ======

#[test]
fn integer_arithmetic_sequence() {
    let vm = run_dt(vec![
        Instruction {
            opcode: OperationCode::MOVE,
            operand1: Some(reg(0)),
            operand2: Some(imm(10)),
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::IADD,
            operand1: Some(reg(0)),
            operand2: Some(reg(0)),
            operand3: Some(imm(5)),
        },
        Instruction {
            opcode: OperationCode::IMUL,
            operand1: Some(reg(0)),
            operand2: Some(reg(0)),
            operand3: Some(imm(3)),
        },
    ]);
    assert_eq!(vm.registers[Register(0)], 45);
}

#[test]
fn float_arithmetic_works() {
    let vm = run_dt(vec![
        Instruction {
            opcode: OperationCode::MOVE,
            operand1: Some(reg(0)),
            operand2: Some(imm(1.5f64.to_bits())),
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::MOVE,
            operand1: Some(reg(1)),
            operand2: Some(imm(2.25f64.to_bits())),
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::FADD,
            operand1: Some(reg(2)),
            operand2: Some(reg(0)),
            operand3: Some(reg(1)),
        },
    ]);
    assert_eq!(f64::from_bits(vm.registers[Register(2)]), 3.75);
}

// ====== Память ======

#[test]
fn load_and_store_work() {
    let mut vm = NVM::new(16);
    vm.program = vec![
        Instruction {
            opcode: OperationCode::STORE64,
            operand1: Some(imm(0)),
            operand2: Some(imm(0xDEAD_BEEF)),
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::LOAD64,
            operand1: Some(reg(0)),
            operand2: Some(imm(0)),
            operand3: None,
        },
    ];
    vm.direct_threading_execute().expect("execution failed");
    assert_eq!(vm.registers[Register(0)], 0xDEAD_BEEF);
}

// ====== Переходы ======

#[test]
fn conditional_loop_works() {
    let mut vm = NVM::new(0);
    // r0 = 0
    // loop:
    //   r0 += 1
    //   r1 = (r0 < 10)   (ULT)
    //   if r1 != 0 goto loop
    vm.program = vec![
        Instruction {
            opcode: OperationCode::MOVE,
            operand1: Some(reg(0)),
            operand2: Some(imm(0)),
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::IADD,
            operand1: Some(reg(0)),
            operand2: Some(reg(0)),
            operand3: Some(imm(1)),
        },
        Instruction {
            opcode: OperationCode::ULT,
            operand1: Some(reg(1)),
            operand2: Some(reg(0)),
            operand3: Some(imm(10)),
        },
        Instruction {
            opcode: OperationCode::JNZ,
            operand1: Some(reg(1)),
            operand2: Some(imm(1)),
            operand3: None,
        },
    ];
    vm.direct_threading_execute().expect("execution failed");
    assert_eq!(vm.registers[Register(0)], 10);
}

#[test]
fn call_jumps_to_subroutine() {
    let vm = run_dt(vec![
        Instruction {
            opcode: OperationCode::CALL,
            operand1: Some(imm(2)),
            operand2: None,
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::MOVE,
            operand1: Some(reg(0)),
            operand2: Some(imm(10)),
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::MOVE,
            operand1: Some(reg(1)),
            operand2: Some(imm(42)),
            operand3: None,
        },
    ]);
    // Инструкции после CALL пропущены.
    assert_eq!(vm.registers[Register(0)], 0);
    assert_eq!(vm.registers[Register(1)], 42);
}

#[test]
fn ret_returns_to_saved_address() {
    let mut vm = NVM::new(0);
    vm.call_stack.push(1);

    vm.program = vec![
        Instruction {
            opcode: OperationCode::RET,
            operand1: None,
            operand2: None,
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::MOVE,
            operand1: Some(reg(0)),
            operand2: Some(imm(42)),
            operand3: None,
        },
    ];
    vm.direct_threading_execute().expect("execution failed");
    assert_eq!(vm.registers[Register(0)], 42);
}

#[test]
fn call_and_ret_round_trip() {
    let vm = run_dt(vec![
        Instruction {
            opcode: OperationCode::MOVE,
            operand1: Some(reg(0)),
            operand2: Some(imm(21)),
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::CALL,
            operand1: Some(imm(4)),
            operand2: None,
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::IADD,
            operand1: Some(reg(0)),
            operand2: Some(reg(0)),
            operand3: Some(reg(0)),
        },
        Instruction {
            opcode: OperationCode::EXIT,
            operand1: None,
            operand2: None,
            operand3: None,
        },
        // Подпрограмма: r0 += r0; RET.
        Instruction {
            opcode: OperationCode::IADD,
            operand1: Some(reg(0)),
            operand2: Some(reg(0)),
            operand3: Some(reg(0)),
        },
        Instruction {
            opcode: OperationCode::RET,
            operand1: None,
            operand2: None,
            operand3: None,
        },
    ]);
    // 21 -> CALL -> 42 -> RET -> 84 -> EXIT.
    assert_eq!(vm.registers[Register(0)], 84);
}

// ====== Ошибки ======

#[test]
fn division_by_zero() {
    let err = match run_dt_with_result(vec![Instruction {
        opcode: OperationCode::UDIV,
        operand1: Some(reg(0)),
        operand2: Some(imm(10)),
        operand3: Some(imm(0)),
    }]) {
        Ok(_) => panic!("expected division by zero error"),
        Err(e) => e,
    };
    assert!(matches!(err.kind, VMErrorKind::DivisionByZero));
}

#[test]
fn empty_call_stack() {
    let err = match run_dt_with_result(vec![Instruction {
        opcode: OperationCode::RET,
        operand1: None,
        operand2: None,
        operand3: None,
    }]) {
        Ok(_) => panic!("expected empty call stack error"),
        Err(e) => e,
    };
    assert!(matches!(err.kind, VMErrorKind::EmptyCallStack));
}

#[test]
fn invalid_address() {
    let err = match run_dt_with_result(vec![Instruction {
        opcode: OperationCode::LOAD64,
        operand1: Some(reg(0)),
        operand2: Some(imm(0)),
        operand3: None,
    }]) {
        Ok(_) => panic!("expected invalid address error"),
        Err(e) => e,
    };
    assert!(matches!(err.kind, VMErrorKind::InvalidAddress { .. }));
}

#[test]
fn incorrect_number_of_operands() {
    let err = match run_dt_with_result(vec![Instruction {
        opcode: OperationCode::MOVE,
        operand1: Some(reg(0)),
        operand2: Some(reg(1)),
        operand3: Some(reg(2)),
    }]) {
        Ok(_) => panic!("expected operand count error"),
        Err(e) => e,
    };
    assert!(matches!(
        err.kind,
        VMErrorKind::IncorrectNumberOfOperands {
            expected: 2,
            got: 3
        }
    ));
}

#[test]
fn incorrect_operand_type() {
    let err = match run_dt_with_result(vec![Instruction {
        opcode: OperationCode::MOVE,
        operand1: Some(Operand {
            kind: OperandKind::Immediate(1),
        }),
        operand2: Some(imm(2)),
        operand3: None,
    }]) {
        Ok(_) => panic!("expected operand type error"),
        Err(e) => e,
    };
    assert!(matches!(
        err.kind,
        VMErrorKind::IncorrectTypeOfOperand { .. }
    ));
}

// ====== Эквивалентность с match-исполнителем ======

#[test]
fn same_state_as_match_executor() {
    let program = vec![
        Instruction {
            opcode: OperationCode::MOVE,
            operand1: Some(reg(0)),
            operand2: Some(imm(1)),
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::STORE64,
            operand1: Some(imm(0)),
            operand2: Some(reg(0)),
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::LOAD64,
            operand1: Some(reg(1)),
            operand2: Some(imm(0)),
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::IMUL,
            operand1: Some(reg(1)),
            operand2: Some(reg(1)),
            operand3: Some(imm(2)),
        },
    ];

    let mut a = NVM::new(16);
    a.program = program.clone();
    a.match_execute().expect("execution failed");

    let mut b = NVM::new(16);
    b.program = program;
    b.direct_threading_execute().expect("execution failed");
    assert_same_state(&a, &b);
}
