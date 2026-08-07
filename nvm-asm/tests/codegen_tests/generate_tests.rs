// nvm-asm/tests/codegen_tests/generate_tests.rs
//
// Тесты на генерацию инструкций из AST.
use nvm_core::isa::opcode::OperationCode;

use super::*;

#[test]
fn program_without_labels_generates_as_is() {
    let program = codegen("MOVE R0, 42\nIADD R0, R1, 1").expect("valid program");

    assert_eq!(program.len(), 2);
    assert!(matches!(program[0].opcode, OperationCode::MOVE));
    assert_operand_eq(program[0].operand1, reg(0));
    assert_operand_eq(program[0].operand2, imm(42));
    assert!(matches!(program[1].opcode, OperationCode::IADD));
    assert_operand_eq(program[1].operand1, reg(0));
    assert_operand_eq(program[1].operand2, reg(1));
    assert_operand_eq(program[1].operand3, imm(1));
}

#[test]
fn label_points_to_next_instruction() {
    let program =
        codegen("begin:\nMOVE R0, 1\nJMP begin\nJMP end\nend:\nRET").expect("valid program");

    assert_eq!(program.len(), 4);
    // "begin" стоит перед MOVE (индекс 0), "end" — перед RET (индекс 3).
    assert!(matches!(program[1].opcode, OperationCode::JMP));
    assert_operand_eq(program[1].operand1, imm(0));
    assert!(matches!(program[2].opcode, OperationCode::JMP));
    assert_operand_eq(program[2].operand1, imm(3));
}

#[test]
fn forward_jump_is_resolved() {
    let program = codegen("JMP loop\nNOP\nloop:\nNOP").expect("valid program");

    assert_eq!(program.len(), 3);
    assert!(matches!(program[0].opcode, OperationCode::JMP));
    assert_operand_eq(program[0].operand1, imm(2));
}

#[test]
fn backward_jump_is_resolved() {
    let program = codegen("loop:\nNOP\nJMP loop").expect("valid program");

    assert_eq!(program.len(), 2);
    assert!(matches!(program[1].opcode, OperationCode::JMP));
    assert_operand_eq(program[1].operand1, imm(0));
}

#[test]
fn call_resolves_to_instruction_index() {
    let program = codegen("CALL sub\nNOP\nsub:\nRET").expect("valid program");

    assert_eq!(program.len(), 3);
    assert!(matches!(program[0].opcode, OperationCode::CALL));
    assert_operand_eq(program[0].operand1, imm(2));
}

#[test]
fn jz_resolves_target_operand() {
    let program = codegen("JZ R0, skip\nNOP\nskip:\nEXIT").expect("valid program");

    assert_eq!(program.len(), 3);
    assert!(matches!(program[0].opcode, OperationCode::JZ));
    assert_operand_eq(program[0].operand1, reg(0));
    assert_operand_eq(program[0].operand2, imm(2));
}

#[test]
fn jnz_resolves_target_operand() {
    let program = codegen("JNZ R0, skip\nNOP\nskip:\nEXIT").expect("valid program");

    assert_eq!(program.len(), 3);
    assert!(matches!(program[0].opcode, OperationCode::JNZ));
    assert_operand_eq(program[0].operand1, reg(0));
    assert_operand_eq(program[0].operand2, imm(2));
}

#[test]
fn label_as_move_source() {
    let program = codegen("MOVE R0, here\nhere:\nNOP").expect("valid program");

    assert_eq!(program.len(), 2);
    assert!(matches!(program[0].opcode, OperationCode::MOVE));
    assert_operand_eq(program[0].operand1, reg(0));
    assert_operand_eq(program[0].operand2, imm(1));
}

#[test]
fn label_as_load_address() {
    let program = codegen("LOAD8 R0, data\ndata:\nNOP").expect("valid program");

    assert_eq!(program.len(), 2);
    assert!(matches!(program[0].opcode, OperationCode::LOAD8));
    assert_operand_eq(program[0].operand1, reg(0));
    assert_operand_eq(program[0].operand2, imm(1));
}

#[test]
fn several_labels_in_a_row() {
    let program = codegen("a:\nb:\nMOVE R0, 1").expect("valid program");

    assert_eq!(program.len(), 1);
    assert!(matches!(program[0].opcode, OperationCode::MOVE));
    assert_operand_eq(program[0].operand1, reg(0));
    assert_operand_eq(program[0].operand2, imm(1));
}

#[test]
fn label_after_last_instruction() {
    let program = codegen("JMP end\nend:").expect("valid program");

    assert_eq!(program.len(), 1);
    assert!(matches!(program[0].opcode, OperationCode::JMP));
    // Метка стоит после фина — переход на индекс, равный длине программы
    // (такой jump завершает выполнение).
    assert_operand_eq(program[0].operand1, imm(1));
}

#[test]
fn zero_operand_instructions_keep_operands_empty() {
    let program = codegen("NOP\nEXIT\nRET").expect("valid program");

    assert_eq!(program.len(), 3);
    assert_operand_none(program[0].operand1);
    assert_operand_none(program[1].operand2);
    assert_operand_none(program[2].operand3);
}

#[test]
fn label_in_third_operand_slot_is_resolved() {
    let program = codegen("IADD R0, R1, here\nhere:\nNOP").expect("valid program");

    assert_eq!(program.len(), 2);
    assert!(matches!(program[0].opcode, OperationCode::IADD));
    assert_operand_eq(program[0].operand1, reg(0));
    assert_operand_eq(program[0].operand2, reg(1));
    assert_operand_eq(program[0].operand3, imm(1));
}

#[test]
fn multiple_references_to_the_same_label_all_resolve() {
    let program = codegen("JMP loop\nJMP loop\nloop:\nNOP").expect("valid program");

    assert_eq!(program.len(), 3);
    assert!(matches!(program[0].opcode, OperationCode::JMP));
    assert_operand_eq(program[0].operand1, imm(2));
    assert!(matches!(program[1].opcode, OperationCode::JMP));
    assert_operand_eq(program[1].operand1, imm(2));
}

#[test]
fn unused_labels_do_not_fail() {
    let program = codegen("unused:\nMOVE R0, 1").expect("valid program");

    assert_eq!(program.len(), 1);
    assert!(matches!(program[0].opcode, OperationCode::MOVE));
    assert_operand_eq(program[0].operand1, reg(0));
    assert_operand_eq(program[0].operand2, imm(1));
}

#[test]
fn label_only_program_generates_empty_program() {
    let program = codegen("start:").expect("valid program");

    assert_eq!(program.len(), 0);
    assert!(program.is_empty());
}

#[test]
fn empty_program_generates_empty_program() {
    let program = codegen("").expect("valid program");

    assert_eq!(program.len(), 0);
    assert!(program.is_empty());
}

#[test]
fn chained_jumps_resolve_in_order() {
    let program = codegen("a:\nJMP b\nb:\nJMP c\nc:\nRET").expect("valid program");

    assert_eq!(program.len(), 3);
    // a -> 0 (первая инструкция), b -> 1, c -> 2 (последняя).
    assert!(matches!(program[0].opcode, OperationCode::JMP));
    assert_operand_eq(program[0].operand1, imm(1));
    assert!(matches!(program[1].opcode, OperationCode::JMP));
    assert_operand_eq(program[1].operand1, imm(2));
}

#[test]
fn label_and_instruction_on_one_line_resolves() {
    let program = codegen("here: NOP\nJMP here").expect("valid program");

    assert_eq!(program.len(), 2);
    // Метка на той же строке перед NOP — точка перехода на индекс 0.
    assert!(matches!(program[1].opcode, OperationCode::JMP));
    assert_operand_eq(program[1].operand1, imm(0));
}

#[test]
fn extreme_registers_are_preserved() {
    let program = codegen("MOVE R255, 42\nIADD R0, R255, R200\nNOT R1, R2").expect("valid program");

    assert_eq!(program.len(), 3);
    assert_operand_eq(program[0].operand1, reg(255));
    assert_operand_eq(program[0].operand2, imm(42));
    assert_operand_eq(program[1].operand1, reg(0));
    assert_operand_eq(program[1].operand2, reg(255));
    assert_operand_eq(program[1].operand3, reg(200));
    assert_operand_eq(program[2].operand1, reg(1));
    assert_operand_eq(program[2].operand2, reg(2));
}

#[test]
fn zero_immediate_is_kept_as_immediate() {
    let program = codegen("MOVE R0, 0").expect("valid program");

    assert_eq!(program.len(), 1);
    assert_operand_eq(program[0].operand1, reg(0));
    assert_operand_eq(program[0].operand2, imm(0));
}

#[test]
fn jump_to_label_at_last_instruction() {
    let program = codegen("JMP last\nNOP\nlast:\nEXIT").expect("valid program");

    assert_eq!(program.len(), 3);
    assert!(matches!(program[0].opcode, OperationCode::JMP));
    assert_operand_eq(program[0].operand1, imm(2));
}

#[test]
fn labels_differ_by_case() {
    let program = codegen("A:\nMOVE R0, 1\na:\nJMP A").expect("valid program");

    assert_eq!(program.len(), 2);
    // "A" и "a" — разные метки: переход по "A" ведёт на индекс 0.
    assert!(matches!(program[1].opcode, OperationCode::JMP));
    assert_operand_eq(program[1].operand1, imm(0));
}

#[test]
fn long_program_resolves_distant_labels() {
    let mut src = String::from("start:\nJMP mid\n");
    for _ in 0..10 {
        src.push_str("NOP\n");
    }
    src.push_str("mid:\nJMP start\n");
    for _ in 0..10 {
        src.push_str("NOP\n");
    }

    let program = codegen(&src).expect("valid program");

    assert_eq!(program.len(), 22);
    // "start" -> 0 (первая инструкция), "mid" -> 11 (перед "JMP start").
    assert!(matches!(program[0].opcode, OperationCode::JMP));
    assert_operand_eq(program[0].operand1, imm(11));
    assert!(matches!(program[11].opcode, OperationCode::JMP));
    assert_operand_eq(program[11].operand1, imm(0));
}
