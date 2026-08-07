// nvm-asm/tests/codegen_tests/error_tests.rs
//
// Тесты на ошибки кодогенерации.
use nvm_asm::codegen::err::CodegenErrorKind;
use nvm_asm::position::Position;

use super::*;

#[test]
fn duplicate_label_is_reported() {
    let err = codegen("a:\nNOP\na:").expect_err("duplicate label must fail");

    assert!(matches!(
        err.kind,
        CodegenErrorKind::DuplicateLabel { ref name } if name == "a"
    ));
    // Ошибка указывает на второе объявление метки.
    assert_eq!(err.position, Position::new(7, 9));
}

#[test]
fn undefined_label_is_reported() {
    let err = codegen("JMP nowhere").expect_err("undefined label must fail");

    assert!(matches!(
        err.kind,
        CodegenErrorKind::UndefinedLabel { ref name } if name == "nowhere"
    ));
    // Ошибка указывает на инструкцию со ссылкой.
    assert_eq!(err.position, Position::new(0, 3));
}

#[test]
fn first_undefined_label_wins() {
    let err = codegen("JMP a\nJMP b").expect_err("undefined labels must fail");

    assert!(matches!(
        err.kind,
        CodegenErrorKind::UndefinedLabel { ref name } if name == "a"
    ));
}

#[test]
fn error_message_contains_label_name() {
    let err = codegen("JMP nowhere").expect_err("undefined label must fail");

    assert!(err.to_string().contains("nowhere"));
    assert!(err.to_string().contains("undefined label"));
}

#[test]
fn duplicate_label_message_contains_label_name() {
    let err = codegen("x:\nx:").expect_err("duplicate label must fail");

    assert!(err.to_string().contains("x"));
    assert!(err.to_string().contains("duplicate label"));
}

#[test]
fn duplicate_label_is_checked_before_undefined_reference() {
    // Дубликат обнаруживается в первом проходе и перекрывает
    // неопределённую ссылку во втором.
    let err = codegen("a:\nJMP b\na:").expect_err("duplicate label must fail");

    assert!(matches!(err.kind, CodegenErrorKind::DuplicateLabel { .. }));
}

#[test]
fn labels_are_case_sensitive() {
    // "Loop" и "loop" — разные идентификаторы.
    let err = codegen("JMP Loop\nloop:\nNOP").expect_err("case mismatch must fail");

    assert!(matches!(
        err.kind,
        CodegenErrorKind::UndefinedLabel { ref name } if name == "Loop"
    ));
}

#[test]
fn undefined_label_in_second_operand() {
    let err = codegen("JZ R0, nowhere").expect_err("undefined label must fail");

    assert!(matches!(
        err.kind,
        CodegenErrorKind::UndefinedLabel { ref name } if name == "nowhere"
    ));
    // Ошибка указывает на инструкцию, а не на операнд.
    assert_eq!(err.position, Position::new(0, 2));
}

#[test]
fn undefined_label_in_third_operand() {
    let err = codegen("IADD R0, R1, nowhere").expect_err("undefined label must fail");

    assert!(matches!(
        err.kind,
        CodegenErrorKind::UndefinedLabel { ref name } if name == "nowhere"
    ));
    assert_eq!(err.position, Position::new(0, 4));
}

#[test]
fn undefined_reference_when_program_has_no_instructions() {
    let err = codegen("JMP nowhere\nend:").expect_err("undefined label must fail");

    assert!(matches!(
        err.kind,
        CodegenErrorKind::UndefinedLabel { ref name } if name == "nowhere"
    ));
}

#[test]
fn duplicate_label_on_adjacent_lines_is_reported() {
    let err = codegen("x:\nx:\nMOVE R0, 1").expect_err("duplicate label must fail");

    assert!(matches!(
        err.kind,
        CodegenErrorKind::DuplicateLabel { ref name } if name == "x"
    ));
    // Позиция — второе объявление (после первого и перевода строки).
    assert_eq!(err.position, Position::new(3, 5));
}

#[test]
fn error_on_duplicate_label_then_reference_to_undefined_keeps_first_position() {
    // Ошибка на дубликат метки в первом проходе имеет позицию
    // второго объявления, даже если далее есть неопределённая ссылка.
    let err = codegen("a:\nJMP b\na:\nJMP c").expect_err("duplicate label must fail");

    assert!(matches!(err.kind, CodegenErrorKind::DuplicateLabel { .. }));
    assert_eq!(err.position, Position::new(9, 11));
}
