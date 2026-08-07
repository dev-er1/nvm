// nvm-asm/tests/parser_tests/error_tests.rs
//
// Тесты на ошибки парсера и восстановление после них.
use nvm_asm::parser::ast::Statement;
use nvm_asm::parser::err::ParserErrorKind;

use super::*;

#[test]
fn label_without_colon_is_an_error() {
    let (ast, errors) = parse("foo\nMOVE R0, 1");

    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind,
        ParserErrorKind::ExpectedLabelColon
    ));
    // Строка пропущена, но разбор продолжается: инструкция ниже разобрана.
    assert_eq!(ast.program.len(), 1);
}

#[test]
fn missing_comma_between_operands() {
    let (ast, errors) = parse("MOVE R0 R1");

    assert_eq!(errors.len(), 1);
    assert!(matches!(errors[0].kind, ParserErrorKind::ExpectedComma));

    // Операнды разобраны несмотря на ошибку.
    assert_eq!(ast.program.len(), 1);
    assert!(matches!(ast.program[0], Statement::Instruction { .. }));
}

#[test]
fn too_few_operands() {
    let (ast, errors) = parse("MOVE R0");

    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind,
        ParserErrorKind::IncorrectNumberOfOperands {
            expected: 2,
            got: 1,
        }
    ));
    assert!(ast.program.is_empty());
}

#[test]
fn too_many_operands() {
    let (ast, errors) = parse("NOP R0");

    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind,
        ParserErrorKind::IncorrectNumberOfOperands {
            expected: 0,
            got: 1,
        }
    ));
    assert!(ast.program.is_empty());
}

#[test]
fn destination_must_be_a_register() {
    let (ast, errors) = parse("MOVE 42, R1");

    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind,
        ParserErrorKind::ExpectedRegisterOperand { .. }
    ));
    // Инструкция всё равно попадает в AST, ошибка — только диагностика.
    assert_eq!(ast.program.len(), 1);
}

#[test]
fn destination_label_is_rejected() {
    let (ast, errors) = parse("MOVE loop, R1\nloop:");

    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind,
        ParserErrorKind::ExpectedRegisterOperand { .. }
    ));
    assert_eq!(ast.program.len(), 2);
}

#[test]
fn trailing_tokens_after_instruction() {
    let (ast, errors) = parse("MOVE R0, R1 extra");

    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind,
        ParserErrorKind::IncorrectNumberOfOperands {
            expected: 2,
            got: 3,
        }
    ));
    assert!(ast.program.is_empty());
}

#[test]
fn unexpected_token_at_statement_start() {
    let (ast, errors) = parse(": MOVE R0, R1");

    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind,
        ParserErrorKind::UnexpectedToken {
            expected: "a label or an instruction",
            ..
        }
    ));
    assert!(ast.program.is_empty());
}

#[test]
fn error_positions_point_to_the_culprit() {
    let (_, errors) = parse("MOVE R0\n");

    assert_eq!(errors.len(), 1);
    // Ошибка о недостающих операндах указывает на начало инструкции.
    assert_eq!(errors[0].position.start, 0);
    assert_eq!(errors[0].position.end, 4);
}

#[test]
fn recovery_continues_after_an_error() {
    let (ast, errors) = parse("MOVE R0\nNOP\nRET");

    assert_eq!(errors.len(), 1);
    // Строки после ошибочной разобраны.
    assert_eq!(ast.program.len(), 2);
    assert!(matches!(ast.program[0], Statement::Instruction { .. }));
    assert!(matches!(ast.program[1], Statement::Instruction { .. }));
}

#[test]
fn several_errors_are_collected() {
    let (ast, errors) = parse("MOVE R0\nMOVE\nMOVE R1, 2");

    assert_eq!(errors.len(), 2);
    assert_eq!(ast.program.len(), 1);
}
