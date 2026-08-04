// Тесты на jump table исполнитель.
use nvm_core::{
    isa::{instruction::Instruction, opcode::OperationCode, register::Register},
    vm::{NVM, err::VMErrorKind},
};

use crate::vm_tests::helpers::*;

// ====== Базовое исполнение ======

#[test]
fn exit_stops_execution() {
    let vm = run_jt(vec![
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
    let vm = run_jt(vec![Instruction {
        opcode: OperationCode::MOVE,
        operand1: Some(reg(0)),
        operand2: Some(imm(42)),
        operand3: None,
    }]);
    assert_eq!(vm.registers[Register(0)], 42);
}

#[test]
fn empty_program_is_ok() {
    run_jt(vec![]);
}

// ====== Арифметика ======

#[test]
fn integer_arithmetic_sequence() {
    let vm = run_jt(vec![
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
            operand1: Some(reg(1)),
            operand2: Some(reg(0)),
            operand3: Some(imm(2)),
        },
        Instruction {
            opcode: OperationCode::ISUB,
            operand1: Some(reg(2)),
            operand2: Some(imm(100)),
            operand3: Some(reg(1)),
        },
    ]);
    assert_eq!(vm.registers[Register(0)], 15);
    assert_eq!(vm.registers[Register(1)], 30);
    assert_eq!(vm.registers[Register(2)], 70);
}

#[test]
fn division_by_zero_error() {
    let err = match run_jt_with_result(vec![Instruction {
        opcode: OperationCode::SDIV,
        operand1: Some(reg(0)),
        operand2: Some(imm(10)),
        operand3: Some(imm(0)),
    }]) {
        Err(err) => err,
        Ok(_) => panic!("expected DivisionByZero error"),
    };
    assert!(matches!(err.kind, VMErrorKind::DivisionByZero));
}

#[test]
fn float_arithmetic_works() {
    let vm = run_jt(vec![
        Instruction {
            opcode: OperationCode::MOVE,
            operand1: Some(reg(0)),
            operand2: Some(imm(1.5f64.to_bits())),
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::MOVE,
            operand1: Some(reg(1)),
            operand2: Some(imm(2.5f64.to_bits())),
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::FADD,
            operand1: Some(reg(2)),
            operand2: Some(reg(0)),
            operand3: Some(reg(1)),
        },
    ]);
    assert_eq!(
        f64::from_bits(vm.registers[Register(2)]),
        1.5 + 2.5,
        "FADD result"
    );
}

// ====== Память ======

#[test]
fn load_store_roundtrip() {
    let vm = NVM::new(16);
    let program = vec![
        Instruction {
            opcode: OperationCode::STORE64,
            operand1: Some(imm(0)),
            operand2: Some(imm(u64::MAX)),
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::LOAD64,
            operand1: Some(reg(0)),
            operand2: Some(imm(0)),
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::LOAD8,
            operand1: Some(reg(1)),
            operand2: Some(imm(7)),
            operand3: None,
        },
    ];

    let vm = run_jt_on(vm, program);

    assert_eq!(vm.registers[Register(0)], u64::MAX);
    assert_eq!(vm.registers[Register(1)], 0xFF);
}

#[test]
fn invalid_address_error() {
    let mut vm = NVM::new(16);
    vm.program = vec![Instruction {
        opcode: OperationCode::LOAD8,
        operand1: Some(reg(0)),
        operand2: Some(imm(16)),
        operand3: None,
    }];

    let err = match vm.jumptable_execute() {
        Err(err) => err,
        Ok(_) => panic!("expected InvalidAddress error"),
    };
    assert!(matches!(err.kind, VMErrorKind::InvalidAddress { .. }));
}

// ====== Переходы ======

#[test]
fn jmp_works() {
    let vm = run_jt(vec![
        Instruction {
            opcode: OperationCode::MOVE,
            operand1: Some(reg(0)),
            operand2: Some(imm(1)),
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::JMP,
            operand1: Some(imm(3)),
            operand2: None,
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::MOVE,
            operand1: Some(reg(0)),
            operand2: Some(imm(99)),
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::MOVE,
            operand1: Some(reg(1)),
            operand2: Some(imm(7)),
            operand3: None,
        },
    ]);
    assert_eq!(vm.registers[Register(0)], 1);
    assert_eq!(vm.registers[Register(1)], 7);
}

#[test]
fn jz_taken_when_zero() {
    let vm = run_jt(vec![
        Instruction {
            opcode: OperationCode::JZ,
            operand1: Some(reg(0)),
            operand2: Some(imm(2)),
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::MOVE,
            operand1: Some(reg(1)),
            operand2: Some(imm(1)),
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::MOVE,
            operand1: Some(reg(2)),
            operand2: Some(imm(2)),
            operand3: None,
        },
    ]);
    assert_eq!(vm.registers[Register(1)], 0);
    assert_eq!(vm.registers[Register(2)], 2);
}

#[test]
fn jnz_taken_when_nonzero() {
    let mut vm = NVM::new(0);
    vm.registers[Register(0)] = 5;

    let vm = run_jt_on(
        vm,
        vec![
            Instruction {
                opcode: OperationCode::JNZ,
                operand1: Some(reg(0)),
                operand2: Some(imm(2)),
                operand3: None,
            },
            Instruction {
                opcode: OperationCode::MOVE,
                operand1: Some(reg(1)),
                operand2: Some(imm(1)),
                operand3: None,
            },
            Instruction {
                opcode: OperationCode::MOVE,
                operand1: Some(reg(2)),
                operand2: Some(imm(2)),
                operand3: None,
            },
        ],
    );
    assert_eq!(vm.registers[Register(1)], 0);
    assert_eq!(vm.registers[Register(2)], 2);
}

#[test]
fn call_jumps_to_subroutine() {
    let vm = run_jt(vec![
        Instruction {
            opcode: OperationCode::CALL,
            operand1: Some(imm(2)),
            operand2: None,
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::MOVE,
            operand1: Some(reg(0)),
            operand2: Some(imm(1)),
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::MOVE,
            operand1: Some(reg(1)),
            operand2: Some(imm(42)),
            operand3: None,
        },
    ]);
    assert_eq!(vm.registers[Register(0)], 0);
    assert_eq!(vm.registers[Register(1)], 42);
}

#[test]
fn ret_returns_to_saved_address() {
    let mut vm = NVM::new(0);
    vm.call_stack.push(1);

    let vm = run_jt_on(
        vm,
        vec![
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
        ],
    );
    assert_eq!(vm.registers[Register(0)], 42);
}

#[test]
fn ret_empty_call_stack_error() {
    let err = match run_jt_with_result(vec![Instruction {
        opcode: OperationCode::RET,
        operand1: None,
        operand2: None,
        operand3: None,
    }]) {
        Err(err) => err,
        Ok(_) => panic!("expected EmptyCallStack error"),
    };
    assert!(matches!(err.kind, VMErrorKind::EmptyCallStack));
}

// ====== Проверки на этапе кодирования ======

#[test]
fn incorrect_operand_count_error() {
    let err = match run_jt_with_result(vec![Instruction {
        opcode: OperationCode::MOVE,
        operand1: Some(reg(0)),
        operand2: None,
        operand3: None,
    }]) {
        Err(err) => err,
        Ok(_) => panic!("expected IncorrectNumberOfOperands error"),
    };
    assert!(matches!(
        err.kind,
        VMErrorKind::IncorrectNumberOfOperands {
            expected: 2,
            got: 1
        }
    ));
}

#[test]
fn incorrect_operand_type_error() {
    let err = match run_jt_with_result(vec![Instruction {
        opcode: OperationCode::MOVE,
        operand1: Some(imm(0)),
        operand2: Some(imm(42)),
        operand3: None,
    }]) {
        Err(err) => err,
        Ok(_) => panic!("expected IncorrectTypeOfOperand error"),
    };
    assert!(matches!(
        err.kind,
        VMErrorKind::IncorrectTypeOfOperand { .. }
    ));
}

// ====== Эквивалентность со стандартным исполнителем ======

#[test]
fn same_result_as_match_executor() {
    let program = vec![
        Instruction {
            opcode: OperationCode::MOVE,
            operand1: Some(reg(0)),
            operand2: Some(imm(3)),
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::IADD,
            operand1: Some(reg(1)),
            operand2: Some(reg(0)),
            operand3: Some(imm(4)),
        },
        Instruction {
            opcode: OperationCode::STORE8,
            operand1: Some(imm(8)),
            operand2: Some(reg(1)),
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::MOVE,
            operand1: Some(reg(2)),
            operand2: Some(imm(0)),
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::JNZ,
            operand1: Some(reg(2)),
            operand2: Some(imm(2)),
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::MOVE,
            operand1: Some(reg(3)),
            operand2: Some(imm(1)),
            operand3: None,
        },
    ];

    let a = run_on(NVM::new(16), program.clone());
    let b = run_jt_on(NVM::new(16), program);

    assert_same_state(&a, &b);
}

#[test]
fn mixed_operand_kinds_match_default() {
    let mut vm = NVM::new(16);
    vm.registers[Register(0)] = 4;
    vm.registers[Register(1)] = 100;

    let program = vec![
        Instruction {
            opcode: OperationCode::ISUB,
            operand1: Some(reg(2)),
            operand2: Some(imm(100)),
            operand3: Some(reg(1)),
        },
        Instruction {
            opcode: OperationCode::IADD,
            operand1: Some(reg(2)),
            operand2: Some(reg(0)),
            operand3: Some(imm(1)),
        },
        Instruction {
            opcode: OperationCode::STORE8,
            operand1: Some(reg(0)),
            operand2: Some(reg(2)),
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::LOAD8,
            operand1: Some(reg(3)),
            operand2: Some(reg(0)),
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::JZ,
            operand1: Some(reg(3)),
            operand2: Some(reg(2)),
            operand3: None,
        },
        Instruction {
            opcode: OperationCode::MOVE,
            operand1: Some(reg(5)),
            operand2: Some(imm(1)),
            operand3: None,
        },
    ];

    let a = run_on(NVM::new(16), program.clone());
    let b = run_jt_on(NVM::new(16), program);

    assert_same_state(&a, &b);
}
