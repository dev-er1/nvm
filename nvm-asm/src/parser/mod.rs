//! # Parser
//!
//! A parser is a program that turns an array of tokens into
//! an ***abstract syntax tree*** (***AST***).
//!
//! ## Module contents
//! - [`ast`] — the AST;
//! - [`err`] — parser errors.
//!
//! ## Grammar
//!
//! A program consists of lines. Each line is a label, an instruction,
//! or a combination of the two:
//!
//! ```text
//! program     := statement*
//! statement   := label? instruction? eol
//!             |  instruction eol
//! label       := IDENT ':' [instruction]
//! instruction := MNEMONIC operand (',' operand)*
//! operand     := REGISTER | INTEGER | FLOAT | IDENT
//! ```
//!
//! A label and an instruction may be on the same line: `main: MOVE R0, 1`.
pub mod ast;
pub mod err;

use nvm_core::isa::opcode::OperationCode;

use crate::{
    lexer::token::{Token, TokenKind},
    position::Position,
};

use self::{
    ast::{AST, Instr, Operand, Statement},
    err::{ParserError, ParserErrorKind},
};

/// Parser for NVM Assembly.
///
/// Takes a token stream, builds an [`AST`] from it, and accumulates
/// the found errors in [`errors`](Self::errors).
pub struct Parser {
    /// The token stream.
    tokens: Vec<Token>,

    /// Index of the current token.
    index: usize,

    /// The built abstract syntax tree.
    ast: AST,

    /// Errors found during parsing.
    pub errors: Vec<ParserError>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            index: 0,
            ast: AST::new(),
            errors: Vec::new(),
        }
    }

    /// Parses the token stream.
    ///
    /// The result is collected into [`ast`](Self::ast), errors — into
    /// [`errors`](Self::errors). After an error, parsing continues from the
    /// next line.
    pub fn parse(&mut self) -> &AST {
        loop {
            match self.peek_kind() {
                Some(TokenKind::Newline) => {
                    self.bump();
                }
                Some(TokenKind::End) | None => break,
                Some(TokenKind::Ident(_)) => self.parse_label(),
                Some(TokenKind::Mnemonic(_)) => self.parse_instruction(),
                _ => {
                    self.push_error_at_current(ParserErrorKind::UnexpectedToken {
                        expected: "a label or an instruction",
                        got: self.current_kind_debug(),
                    });
                    self.skip_to_newline();
                }
            }
        }

        &self.ast
    }

    // ====== Parsing lines ======

    /// Parses a label: `name:` with an optional instruction on the same
    /// line.
    fn parse_label(&mut self) {
        let name = match self.bump_kind() {
            TokenKind::Ident(id) => id,
            _ => unreachable!("called only when the current token is an identifier"),
        };

        if !matches!(self.peek_kind(), Some(TokenKind::Colon)) {
            self.push_error_at_current(ParserErrorKind::ExpectedLabelColon);
            self.skip_to_newline();
            return;
        }

        let colon = self.bump();
        let position = self.tokens[self.index - 2].position.to(colon.position);
        self.ast.program.push(Statement::Label { name, position });

        if matches!(self.peek_kind(), Some(TokenKind::Mnemonic(_))) {
            self.parse_instruction();
        }
    }

    /// Parses an instruction: `MNEMONIC op1, op2, op3`.
    fn parse_instruction(&mut self) {
        let start_pos = self.current_token().position;
        let opcode = match self.bump_kind() {
            TokenKind::Mnemonic(opcode) => opcode,
            _ => unreachable!("called only when the current token is a mnemonic"),
        };

        let expected = operand_count(opcode);
        let needs_register_dst = requires_register_dst(opcode);

        let mut operands: [Option<Operand>; 3] = [None; 3];
        let mut count = 0usize;

        while count < expected as usize {
            if count > 0 {
                match self.peek_kind() {
                    Some(TokenKind::Comma) => {
                        self.bump();
                    }
                    Some(kind) if is_operand_start(kind) => {
                        // The comma is missing, but an operand begins — parsing
                        // continues so that the rest of the line's errors are found too.
                        self.push_error_at_current(ParserErrorKind::ExpectedComma);
                    }
                    _ => break,
                }
            }

            let Some(kind) = self.peek_kind() else {
                break;
            };

            if let Some(operand) = operand_from_kind(kind) {
                self.bump();
                operands[count] = Some(operand);
                count += 1;
            } else {
                break;
            }
        }

        // After a full set of operands, the line must end.
        if count == expected as usize
            && !matches!(
                self.peek_kind(),
                Some(TokenKind::Newline) | Some(TokenKind::End) | None
            )
        {
            // An extra operand is the same IncorrectNumberOfOperands,
            // so it is counted via the number of operands up to the end of the line.
            if self.peek_is_operand() {
                let extra = self.count_operands_until_newline();
                self.push_error(
                    start_pos,
                    ParserErrorKind::IncorrectNumberOfOperands {
                        expected: expected as u8,
                        got: (count + extra) as u8,
                    },
                );
            } else {
                self.push_error_at_current(ParserErrorKind::UnexpectedToken {
                    expected: "end of statement",
                    got: self.current_kind_debug(),
                });
            }
            self.skip_to_newline();
            return;
        }

        if count != expected as usize {
            self.push_error(
                start_pos,
                ParserErrorKind::IncorrectNumberOfOperands {
                    expected: expected as u8,
                    got: count as u8,
                },
            );
            self.skip_to_newline();
            return;
        }

        if needs_register_dst {
            let Some(dst) = operands[0] else {
                unreachable!("count == expected >= 1, so the destination is present");
            };

            if !matches!(dst, Operand::Register(_)) {
                self.push_error(
                    start_pos,
                    ParserErrorKind::ExpectedRegisterOperand {
                        got: format!("{dst:?}"),
                    },
                );
            }
        }

        self.ast.program.push(Statement::Instruction {
            position: start_pos,
            instruction: Instr {
                opcode,
                operand1: operands[0],
                operand2: operands[1],
                operand3: operands[2],
            },
        });
    }

    // ====== Helper functions ======

    /// Skips tokens up to the end of the line (including the line separator).
    fn skip_to_newline(&mut self) {
        loop {
            match self.peek_kind() {
                Some(TokenKind::Newline) => {
                    self.bump();
                    break;
                }
                Some(TokenKind::End) | None => break,
                _ => {
                    self.bump();
                }
            }
        }
    }

    /// Adds an error to [`errors`](Self::errors)
    /// with the position of the current token.
    fn push_error_at_current(&mut self, kind: ParserErrorKind) {
        let position = self.current_token().position;
        self.push_error(position, kind);
    }

    /// Adds an error to [`errors`](Self::errors).
    fn push_error(&mut self, position: Position, kind: ParserErrorKind) {
        self.errors.push(ParserError::new(kind, position));
    }

    /// Returns the debug representation of the current token.
    fn current_kind_debug(&self) -> String {
        match self.peek_kind() {
            Some(kind) => format!("{kind:?}"),
            None => String::from("end of input"),
        }
    }

    /// Whether the current token starts an operand.
    fn peek_is_operand(&self) -> bool {
        match self.peek_kind() {
            Some(kind) => is_operand_start(kind),
            None => false,
        }
    }

    /// Counts consecutive operands up to the end of the line (for extra operands).
    fn count_operands_until_newline(&self) -> usize {
        let mut extra = 0usize;
        let mut index = self.index;

        while let Some(token) = self.tokens.get(index) {
            match &token.kind {
                TokenKind::Newline | TokenKind::End => break,
                kind if is_operand_start(kind) => extra += 1,
                _ => break,
            }
            index += 1;
        }

        extra
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.index)
    }

    fn peek_kind(&self) -> Option<&TokenKind> {
        self.peek().map(|token| &token.kind)
    }

    fn current_token(&self) -> &Token {
        self.peek()
            .expect("the parser never reads beyond the final End token")
    }

    fn bump(&mut self) -> Token {
        let token = self.tokens[self.index].clone();
        self.index += 1;
        token
    }

    fn bump_kind(&mut self) -> TokenKind {
        self.bump().kind
    }
}

/// Converts a token kind into an AST operand if the token can be an
/// operand. Integer literals are "wrapped" into `u64` per two's complement
/// rules, floating-point literals — into their bit
/// representation (`f64::to_bits`).
fn operand_from_kind(kind: &TokenKind) -> Option<Operand> {
    match kind {
        TokenKind::Register(register) => Some(Operand::Register(*register)),
        TokenKind::Integer(value) => Some(Operand::Immediate(*value as u64)),
        TokenKind::Float(value) => Some(Operand::Immediate((*value).to_bits())),
        TokenKind::Ident(id) => Some(Operand::Label(*id)),
        _ => None,
    }
}

/// Whether the token starts an operand.
fn is_operand_start(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Register(_) | TokenKind::Integer(_) | TokenKind::Float(_) | TokenKind::Ident(_)
    )
}

/// The expected number of operands for an opcode.
fn operand_count(opcode: OperationCode) -> u8 {
    use OperationCode::*;

    match opcode {
        NOP | EXIT | RET => 0,
        JMP | CALL => 1,
        JZ | JNZ | MOVE | LOAD8 | LOAD16 | LOAD32 | LOAD64 | STORE8 | STORE16 | STORE32
        | STORE64 | INEG | FNEG | NOT => 2,
        IADD | ISUB | IMUL | SDIV | UDIV | SREM | UREM | FADD | FSUB | FMUL | FDIV | FREM | AND
        | OR | XOR | SHL | SHR | SAR | IEQ | INE | SLT | SLE | SGT | SGE | ULT | ULE | UGT
        | UGE | FEQ | FNE | FLT | FLE | FGT | FGE => 3,
    }
}

/// Whether the destination (the first operand) must be a register.
///
/// Matches the [iserial](crate::lexer) signatures and the executor's
/// dispatch table: the destinations of `MOVE`, `LOAD*`, unary and binary
/// operations — registers only.
fn requires_register_dst(opcode: OperationCode) -> bool {
    use OperationCode::*;

    matches!(
        opcode,
        MOVE | LOAD8
            | LOAD16
            | LOAD32
            | LOAD64
            | INEG
            | FNEG
            | NOT
            | IADD
            | ISUB
            | IMUL
            | SDIV
            | UDIV
            | SREM
            | UREM
            | FADD
            | FSUB
            | FMUL
            | FDIV
            | FREM
            | AND
            | OR
            | XOR
            | SHL
            | SHR
            | SAR
            | IEQ
            | INE
            | SLT
            | SLE
            | SGT
            | SGE
            | ULT
            | ULE
            | UGT
            | UGE
            | FEQ
            | FNE
            | FLT
            | FLE
            | FGT
            | FGE
    )
}
