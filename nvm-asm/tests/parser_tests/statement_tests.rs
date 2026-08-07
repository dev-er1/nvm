// nvm-asm/tests/parser_tests/statement_tests.rs
//
// Тесты на разбор меток и инструкций.
use nvm_asm::parser::ast::{Operand, Statement};
use nvm_core::isa::opcode::OperationCode;

use super::*;

#[test]
fn empty_program_is_an_empty_ast() {
    let (ast, errors) = parse("");

    assert!(errors.is_empty());
    assert!(ast.program.is_empty());
}

#[test]
fn blank_lines_and_comments_are_skipped() {
    let (ast, errors) = parse("\n\n; только комментарий\n");

    assert!(errors.is_empty());
    assert!(ast.program.is_empty());
}

#[test]
fn label_only_statement() {
    let (ast, errors) = parse("main:");

    assert!(errors.is_empty());
    assert_eq!(ast.program.len(), 1);
    assert!(matches!(
        ast.program[0],
        Statement::Label {
            position,
            ..
        } if position.start == 0 && position.end == 5
    ));
}

#[test]
fn label_position_covers_name_and_colon() {
    let (ast, errors) = parse(" main:");

    assert!(errors.is_empty());
    match &ast.program[0] {
        Statement::Label { position, .. } => {
            assert_eq!(position.start, 1);
            assert_eq!(position.end, 6);
        }
        _ => panic!("expected a label"),
    }
}

#[test]
fn label_and_instruction_on_one_line() {
    let (ast, errors) = parse("main: MOVE R0, 42");

    assert!(errors.is_empty());
    assert_eq!(ast.program.len(), 2);
    assert!(matches!(ast.program[0], Statement::Label { .. }));
    match &ast.program[1] {
        Statement::Instruction { instruction, .. } => {
            assert!(matches!(instruction.opcode, OperationCode::MOVE));
            assert!(matches!(
                instruction.operand1,
                Some(Operand::Register(r)) if r.0 == 0
            ));
            assert!(matches!(instruction.operand2, Some(Operand::Immediate(42))));
        }
        _ => panic!("expected an instruction"),
    }
}

#[test]
fn two_labels_and_instruction_on_one_line() {
    let (ast, errors) = parse("a: b: NOP");

    assert!(errors.is_empty());
    assert_eq!(ast.program.len(), 3);
    assert!(matches!(ast.program[0], Statement::Label { .. }));
    assert!(matches!(ast.program[1], Statement::Label { .. }));
    assert!(matches!(ast.program[2], Statement::Instruction { .. }));
}

#[test]
fn register_operands() {
    let (ast, errors) = parse("MOVE R0, R1");

    assert!(errors.is_empty());
    let instruction = first_instr(&ast);
    assert!(matches!(instruction.operand1, Some(Operand::Register(r)) if r.0 == 0));
    assert!(matches!(instruction.operand2, Some(Operand::Register(r)) if r.0 == 1));
    assert!(instruction.operand3.is_none());
}

#[test]
fn integer_operands() {
    let (ast, errors) = parse("MOVE R0, 42");

    assert!(errors.is_empty());
    let instruction = first_instr(&ast);
    assert!(matches!(instruction.operand1, Some(Operand::Register(r)) if r.0 == 0));
    assert!(matches!(instruction.operand2, Some(Operand::Immediate(42))));
}

#[test]
fn negative_integer_wraps_to_u64() {
    let (ast, errors) = parse("MOVE R0, -1");

    assert!(errors.is_empty());
    let instruction = first_instr(&ast);
    assert!(matches!(
        instruction.operand2,
        Some(Operand::Immediate(v)) if v == u64::MAX
    ));
}

#[test]
fn float_becomes_bit_pattern() {
    let (ast, errors) = parse("FADD R0, 1.0, 2.0");

    assert!(errors.is_empty());
    let instruction = first_instr(&ast);
    assert!(matches!(
        instruction.operand2,
        Some(Operand::Immediate(v)) if v == 1.0f64.to_bits()
    ));
    assert!(matches!(
        instruction.operand3,
        Some(Operand::Immediate(v)) if v == 2.0f64.to_bits()
    ));
}

#[test]
fn label_reference_is_kept_in_ast() {
    let (ast, errors) = parse("JMP loop\nloop: NOP");

    assert!(errors.is_empty());
    assert_eq!(ast.program.len(), 3);

    match (&ast.program[0], &ast.program[1]) {
        (Statement::Instruction { instruction, .. }, Statement::Label { name, .. }) => {
            // Ссылка на метку и её объявление указывают на одно имя.
            assert!(matches!(instruction.operand1, Some(Operand::Label(id)) if id == *name));
        }
        _ => panic!("expected an instruction and a label"),
    }
}

#[test]
fn call_with_label_and_no_operand_instructions() {
    let (ast, errors) = parse("CALL foo\nNOP\nEXIT\nRET");

    assert!(errors.is_empty());

    let call = match &ast.program[0] {
        Statement::Instruction { instruction, .. } => instruction,
        _ => panic!("expected an instruction"),
    };
    assert!(matches!(call.operand1, Some(Operand::Label(_))));

    for statement in &ast.program[1..] {
        match statement {
            Statement::Instruction { instruction, .. } => {
                assert!(instruction.operand1.is_none());
            }
            _ => panic!("expected an instruction"),
        }
    }
}

#[test]
fn three_operand_instruction() {
    let (ast, errors) = parse("IADD R0, R1, 5");

    assert!(errors.is_empty());
    let instruction = first_instr(&ast);
    assert!(matches!(instruction.operand1, Some(Operand::Register(r)) if r.0 == 0));
    assert!(matches!(instruction.operand2, Some(Operand::Register(r)) if r.0 == 1));
    assert!(matches!(instruction.operand3, Some(Operand::Immediate(5))));
}

#[test]
fn label_before_instruction_across_lines() {
    let (ast, errors) = parse("main:\nMOVE R0, 1\nCALL foo\nRET");

    assert!(errors.is_empty());
    assert_eq!(ast.program.len(), 4);
    assert!(matches!(ast.program[0], Statement::Label { .. }));
    assert!(matches!(ast.program[1], Statement::Instruction { .. }));
    assert!(matches!(ast.program[2], Statement::Instruction { .. }));
    assert!(matches!(ast.program[3], Statement::Instruction { .. }));
}
