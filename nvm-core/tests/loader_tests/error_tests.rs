// Tests on loader errors of `.nb`-files.
use nvm_core::loader::err::LoaderErrorKind;

use super::*;

#[test]
fn file_under_11_bytes_returns_format_error() {
    for size in 0..=10 {
        let data = vec![0u8; size];
        let err = match run_loader(data) {
            Err(e) => e,
            Ok(_) => panic!("expected loader error for size {size}"),
        };

        assert!(
            matches!(
                err.kind,
                LoaderErrorKind::FileIsNotInNVMBytecodeFormat { .. }
            ),
            "size {size}: expected FileIsNotInNVMBytecodeFormat, got {:?}",
            err.kind,
        );
    }
}

#[test]
fn unknown_opcode_returns_error() {
    // RET = 52 (0x34). Any opcode > 52 is unknown.
    let bytes = vec![53, 0x00]; // opcode 53 > RET

    let err = match run_loader(make_nb(&bytes)) {
        Err(e) => e,
        Ok(_) => panic!("expected loader error"),
    };

    assert!(matches!(
        err.kind,
        LoaderErrorKind::UnknownOpcode { byte: 53 }
    ));
}

#[test]
fn opcode_255_returns_unknown_opcode() {
    let bytes = vec![0xFF, 0x00];

    let err = match run_loader(make_nb(&bytes)) {
        Err(e) => e,
        Ok(_) => panic!("expected loader error"),
    };

    assert!(matches!(
        err.kind,
        LoaderErrorKind::UnknownOpcode { byte: 0xFF }
    ));
}

#[test]
fn unknown_operand_tag_returns_error() {
    // MOVE with operand tag 0xFF instead of 0x00 or 0x01.
    let bytes = vec![0x02, 0x01, 0xFF, 0x00];

    let err = match run_loader(make_nb(&bytes)) {
        Err(e) => e,
        Ok(_) => panic!("expected loader error"),
    };

    assert!(matches!(
        err.kind,
        LoaderErrorKind::UnknownOperandTag { byte: 0xFF }
    ));
}

#[test]
fn operand_tag_0x02_returns_error() {
    let bytes = vec![0x02, 0x01, 0x02, 0x00];

    let err = match run_loader(make_nb(&bytes)) {
        Err(e) => e,
        Ok(_) => panic!("expected loader error"),
    };

    assert!(matches!(
        err.kind,
        LoaderErrorKind::UnknownOperandTag { byte: 0x02 }
    ));
}

#[test]
fn truncated_instruction_header_returns_unexpected_eof() {
    // There is only an opcode, no operand_count.
    let bytes = vec![0x02];

    let err = match run_loader(make_nb(&bytes)) {
        Err(e) => e,
        Ok(_) => panic!("expected loader error"),
    };

    assert!(matches!(
        err.kind,
        LoaderErrorKind::UnexpectedEndOfFile { .. }
    ));
}

#[test]
fn truncated_register_operand_returns_unexpected_eof() {
    // MOVE r0, ? — there is a tag 0x00, but no register byte.
    let bytes = vec![0x02, 0x02, 0x00, 0x00, 0x00];

    let err = match run_loader(make_nb(&bytes)) {
        Err(e) => e,
        Ok(_) => panic!("expected loader error"),
    };

    assert!(matches!(
        err.kind,
        LoaderErrorKind::UnexpectedEndOfFile { .. }
    ));
}

#[test]
fn truncated_immediate_operand_returns_unexpected_eof() {
    // MOVE r0, imm — there is a tag 0x01, but only 3 bytes instead of 8.
    let bytes = vec![0x02, 0x02, 0x00, 0x00, 0x01, 0xAA, 0xBB, 0xCC];

    let err = match run_loader(make_nb(&bytes)) {
        Err(e) => e,
        Ok(_) => panic!("expected loader error"),
    };

    assert!(matches!(
        err.kind,
        LoaderErrorKind::UnexpectedEndOfFile { .. }
    ));
}

#[test]
fn truncated_after_opcode_with_operand_count_only() {
    // There is an opcode and operand_count = 2, but not a single operand.
    let bytes = vec![0x02, 0x02];

    let err = match run_loader(make_nb(&bytes)) {
        Err(e) => e,
        Ok(_) => panic!("expected loader error"),
    };

    assert!(matches!(
        err.kind,
        LoaderErrorKind::UnexpectedEndOfFile { .. }
    ));
}

#[test]
fn operand_count_4_returns_unknown_opcode() {
    // operand_count > 3 is treated as an unknown opcode.
    let bytes = vec![0x00, 0x04];

    let err = match run_loader(make_nb(&bytes)) {
        Err(e) => e,
        Ok(_) => panic!("expected loader error"),
    };

    assert!(matches!(err.kind, LoaderErrorKind::UnknownOpcode { .. }));
}

#[test]
fn operand_count_255_returns_unknown_opcode() {
    let bytes = vec![0x02, 0xFF];

    let err = match run_loader(make_nb(&bytes)) {
        Err(e) => e,
        Ok(_) => panic!("expected loader error"),
    };

    assert!(matches!(err.kind, LoaderErrorKind::UnknownOpcode { .. }));
}

#[test]
fn negative_error_reason_contains_description() {
    let err = match run_loader(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]) {
        Err(e) => e,
        Ok(_) => panic!("expected loader error"),
    };

    let msg = err.to_string();
    assert!(msg.contains("not in NVM Bytecode format"));
}

#[test]
fn truncated_file_with_only_magic() {
    // 5 magic + 4 bytes of version (9 in total).
    let mut data = b"NVMBC".to_vec();
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

    let err = match run_loader(data) {
        Err(e) => e,
        Ok(_) => panic!("expected loader error"),
    };

    assert!(matches!(
        err.kind,
        LoaderErrorKind::FileIsNotInNVMBytecodeFormat { .. }
    ));
}

#[test]
fn empty_instruction_with_operand_count_1_and_no_data() {
    // opcode=0x00 (NOP), operand_count=1, but not a single operand byte.
    let bytes = vec![0x00, 0x01];

    let err = match run_loader(make_nb(&bytes)) {
        Err(e) => e,
        Ok(_) => panic!("expected loader error"),
    };

    assert!(matches!(
        err.kind,
        LoaderErrorKind::UnexpectedEndOfFile { .. }
    ));
}
