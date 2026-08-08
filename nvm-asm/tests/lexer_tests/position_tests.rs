// nvm-asm/tests/lexer_tests/position_tests.rs
//
// Tests for token positions in the source code.
use nvm_asm::lexer::err::LexerErrorKind;
use nvm_asm::position::Position;

use super::*;

#[test]
fn positions_on_the_first_line() {
    let (tokens, _) = tokenize("MOVE R0, 42");

    assert_eq!(tokens[0].position, Position::new(0, 4)); // MOVE
    assert_eq!(tokens[1].position, Position::new(5, 7)); // R0
    assert_eq!(tokens[2].position, Position::new(7, 8)); // comma
    assert_eq!(tokens[3].position, Position::new(9, 11)); // 42
    assert_eq!(tokens[4].position, Position::new(11, 11)); // End
}

#[test]
fn positions_after_newline_are_byte_offsets() {
    let (tokens, _) = tokenize("MOVE R0, R1\nCALL foo");

    assert_eq!(tokens[4].position, Position::new(11, 12)); // \n
    assert_eq!(tokens[5].position, Position::new(12, 16)); // CALL
    assert_eq!(tokens[6].position, Position::new(17, 20)); // foo
}

#[test]
fn newline_token_has_single_byte_span() {
    let (tokens, _) = tokenize("MOVE R0, R1\nMOVE R0, R2");

    assert_eq!(tokens[4].position, Position::new(11, 12));
}

#[test]
fn crlf_is_a_single_newline_token() {
    let (tokens, errors) = tokenize("MOVE R0, R1\r\nMOVE R0, R2");

    assert!(errors.is_empty());

    // \r is discarded as whitespace, \n yields exactly one Newline.
    let newlines = tokens
        .iter()
        .filter(|t| matches!(t.kind, TokenKind::Newline))
        .count();
    assert_eq!(newlines, 1);
    assert!(matches!(tokens[5].kind, TokenKind::Mnemonic(_)));
}

#[test]
fn comment_does_not_affect_following_positions() {
    let (tokens, errors) = tokenize("MOVE R0, R1 ; note\nCALL foo");

    assert!(errors.is_empty());
    assert_eq!(tokens[4].position, Position::new(18, 19)); // \n after the comment
    assert_eq!(tokens[5].position, Position::new(19, 23)); // CALL
}

#[test]
fn end_token_is_at_source_end() {
    let (tokens, _) = tokenize("NOP\n");

    assert_eq!(tokens[2].position, Position::new(4, 4));
    assert!(matches!(tokens[2].kind, TokenKind::End));
}

#[test]
fn error_position_covers_whole_register_word() {
    // "R256" occupies bytes 0..4; the error covers the whole word.
    let (_, errors) = tokenize("R256");

    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].pos.start, 0);
    assert_eq!(errors[0].pos.end, 4);
}

#[test]
fn numbers_cover_their_full_span() {
    let (tokens, _) = tokenize("1234 1.5 -2");

    assert_eq!(tokens[0].position, Position::new(0, 4)); // 1234
    assert_eq!(tokens[1].position, Position::new(5, 8)); // 1.5
    assert_eq!(tokens[2].position, Position::new(9, 11)); // -2
}

#[test]
fn error_position_covers_single_character() {
    let (_, errors) = tokenize("!");

    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].pos, Position::new(0, 1));
}

#[test]
fn positions_after_an_error_continue() {
    let (tokens, errors) = tokenize("MOVE ! R0");

    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].pos, Position::new(5, 6)); // !
    // Parsing didn't stop: R0 and End point to their own bytes.
    assert_eq!(tokens[1].position, Position::new(7, 9)); // R0
    assert_eq!(tokens[2].position, Position::new(9, 9)); // End
}

#[test]
fn non_ascii_bytes_are_reported_per_byte() {
// The lexer works with ASCII only: each byte of a UTF-8
// sequence is a separate error on its own byte.
    let (tokens, errors) = tokenize("я");

    assert_eq!(errors.len(), 2);
    assert!(matches!(
        errors[0].kind,
        LexerErrorKind::UnexpectedCharacter(_)
    ));
    assert_eq!(errors[0].pos, Position::new(0, 1));
    assert_eq!(errors[1].pos, Position::new(1, 2));
    assert!(matches!(tokens[0].kind, TokenKind::End));
}

#[test]
fn end_token_after_last_token() {
    let (tokens, _) = tokenize("42");

    assert_eq!(tokens[0].position, Position::new(0, 2));
    assert_eq!(tokens[1].position, Position::new(2, 2)); // End
}

#[test]
fn positions_across_multiple_lines() {
    let (tokens, _) = tokenize("A\nB\nC");

    assert_eq!(tokens[0].position, Position::new(0, 1)); // A
    assert_eq!(tokens[1].position, Position::new(1, 2)); // \n
    assert_eq!(tokens[2].position, Position::new(2, 3)); // B
    assert_eq!(tokens[3].position, Position::new(3, 4)); // \n
    assert_eq!(tokens[4].position, Position::new(4, 5)); // C
    assert_eq!(tokens[5].position, Position::new(5, 5)); // End
}
