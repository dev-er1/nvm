// Тесты на успешный парсинг инструкций из `.nb`-файлов.
use nvm_core::isa::opcode::OperationCode;

use super::*;

#[test]
fn empty_instruction_stream_returns_empty_vec() {
    let instructions = run_loader(make_nb(&[])).expect("expected successful parse");
    assert!(instructions.is_empty());
}

#[test]
fn single_nop_instruction() {
    let instructions = run_loader(make_nb(&nop_bytes())).expect("expected successful parse");

    assert_eq!(instructions.len(), 1);
    assert!(matches!(instructions[0].opcode, OperationCode::NOP));
    assert_eq!(instructions[0].operand_count(), 0);
}

#[test]
fn single_exit_instruction() {
    let instructions = run_loader(make_nb(&exit_bytes())).expect("expected successful parse");

    assert_eq!(instructions.len(), 1);
    assert!(matches!(instructions[0].opcode, OperationCode::EXIT));
    assert_eq!(instructions[0].operand_count(), 0);
}

#[test]
fn move_with_two_registers() {
    let instructions = run_loader(make_nb(&move_reg_reg(0, 1))).expect("expected successful parse");

    assert_eq!(instructions.len(), 1);
    let instr = &instructions[0];
    assert!(matches!(instr.opcode, OperationCode::MOVE));

    let r0 = instr.operand1.unwrap().expect_register().unwrap();
    let r1 = instr.operand2.unwrap().expect_register().unwrap();
    assert_eq!(r0.0, 0);
    assert_eq!(r1.0, 1);
}

#[test]
fn move_with_register_and_immediate() {
    let instructions =
        run_loader(make_nb(&move_reg_imm(2, 42))).expect("expected successful parse");

    assert_eq!(instructions.len(), 1);
    let instr = &instructions[0];
    assert!(matches!(instr.opcode, OperationCode::MOVE));

    let dst = instr.operand1.unwrap().expect_register().unwrap();
    let val = instr.operand2.unwrap().expect_immediate().unwrap();
    assert_eq!(dst.0, 2);
    assert_eq!(val, 42);
}

#[test]
fn iadd_with_three_register_operands() {
    // IADD r0, r1, r2
    // opcode=11 (0x0B), operand_count=3
    // reg tag=0x00 + reg=0x00, reg tag=0x00 + reg=0x01, reg tag=0x00 + reg=0x02
    let bytes = vec![0x0B, 0x03, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02];

    let instructions = run_loader(make_nb(&bytes)).expect("expected successful parse");

    assert_eq!(instructions.len(), 1);
    let instr = &instructions[0];
    assert!(matches!(instr.opcode, OperationCode::IADD));
    assert_eq!(instr.operand_count(), 3);

    assert_eq!(instr.operand1.unwrap().expect_register().unwrap().0, 0);
    assert_eq!(instr.operand2.unwrap().expect_register().unwrap().0, 1);
    assert_eq!(instr.operand3.unwrap().expect_register().unwrap().0, 2);
}

#[test]
fn iadd_with_mixed_operands() {
    // IADD r0, r1, 100
    // opcode=11, operand_count=3
    // reg 0x00, reg 0x01, imm 100
    let mut bytes = vec![0x0B, 0x03, 0x00, 0x00, 0x00, 0x01, 0x01];
    bytes.extend_from_slice(&100u64.to_le_bytes());

    let instructions = run_loader(make_nb(&bytes)).expect("expected successful parse");

    assert_eq!(instructions.len(), 1);
    let instr = &instructions[0];
    assert!(matches!(instr.opcode, OperationCode::IADD));

    assert_eq!(instr.operand1.unwrap().expect_register().unwrap().0, 0);
    assert_eq!(instr.operand2.unwrap().expect_register().unwrap().0, 1);
    assert_eq!(instr.operand3.unwrap().expect_immediate().unwrap(), 100);
}

#[test]
fn multiple_instructions_parsed_correctly() {
    // NOP, EXIT, MOVE r0, r1
    let mut bytes = vec![];
    bytes.extend_from_slice(&nop_bytes());
    bytes.extend_from_slice(&exit_bytes());
    bytes.extend_from_slice(&move_reg_reg(0, 1));

    let instructions = run_loader(make_nb(&bytes)).expect("expected successful parse");

    assert_eq!(instructions.len(), 3);

    assert!(matches!(instructions[0].opcode, OperationCode::NOP));
    assert!(matches!(instructions[1].opcode, OperationCode::EXIT));
    assert!(matches!(instructions[2].opcode, OperationCode::MOVE));
    assert_eq!(instructions[2].operand_count(), 2);
}

#[test]
fn ret_instruction() {
    // RET = opcode 52 (0x34), 0 операндов
    let bytes = vec![0x34, 0x00];

    let instructions = run_loader(make_nb(&bytes)).expect("expected successful parse");

    assert_eq!(instructions.len(), 1);
    assert!(matches!(instructions[0].opcode, OperationCode::RET));
}

#[test]
fn jmp_with_immediate_offset() {
    // JMP 3 — opcode=48 (0x30), operand_count=1, imm=3
    let mut bytes = vec![0x30, 0x01, 0x01];
    bytes.extend_from_slice(&3u64.to_le_bytes());

    let instructions = run_loader(make_nb(&bytes)).expect("expected successful parse");

    assert_eq!(instructions.len(), 1);
    assert!(matches!(instructions[0].opcode, OperationCode::JMP));
    assert_eq!(
        instructions[0]
            .operand1
            .unwrap()
            .expect_immediate()
            .unwrap(),
        3
    );
}

#[test]
fn jmp_with_register_offset() {
    // JMP r5 — opcode=48, operand_count=1, reg tag=0x00 + reg=5
    let bytes = vec![0x30, 0x01, 0x00, 0x05];

    let instructions = run_loader(make_nb(&bytes)).expect("expected successful parse");

    assert_eq!(instructions.len(), 1);
    assert!(matches!(instructions[0].opcode, OperationCode::JMP));
    assert_eq!(
        instructions[0]
            .operand1
            .unwrap()
            .expect_register()
            .unwrap()
            .0,
        5
    );
}

#[test]
fn instruction_index_is_preserved_in_order() {
    let mut bytes = vec![];
    bytes.extend_from_slice(&nop_bytes()); // #0
    bytes.extend_from_slice(&exit_bytes()); // #1
    bytes.extend_from_slice(&nop_bytes()); // #2
    bytes.extend_from_slice(&exit_bytes()); // #3
    bytes.extend_from_slice(&nop_bytes()); // #4

    let instructions = run_loader(make_nb(&bytes)).expect("expected successful parse");

    assert_eq!(instructions.len(), 5);
    assert!(matches!(instructions[2].opcode, OperationCode::NOP));
    assert!(matches!(instructions[3].opcode, OperationCode::EXIT));
}

#[test]
fn large_immediate_value() {
    // MOVE r0, 0xDEAD_BEEF_CAFE_BABE
    let bytes = move_reg_imm(0, 0xDEAD_BEEF_CAFE_BABE);

    let instructions = run_loader(make_nb(&bytes)).expect("expected successful parse");
    assert_eq!(instructions.len(), 1);

    let val = instructions[0]
        .operand2
        .unwrap()
        .expect_immediate()
        .unwrap();
    assert_eq!(val, 0xDEAD_BEEF_CAFE_BABE);
}

#[test]
fn instruction_stream_ignores_trailing_data_until_next_opcode() {
    // Инструкция за другой инструкцией без зазора.
    let mut bytes = vec![];
    bytes.extend_from_slice(&move_reg_imm(0, 7));
    bytes.extend_from_slice(&move_reg_imm(1, 8));

    let instructions = run_loader(make_nb(&bytes)).expect("expected successful parse");
    assert_eq!(instructions.len(), 2);

    assert_eq!(
        instructions[0]
            .operand1
            .unwrap()
            .expect_register()
            .unwrap()
            .0,
        0
    );
    assert_eq!(
        instructions[1]
            .operand1
            .unwrap()
            .expect_register()
            .unwrap()
            .0,
        1
    );
}
