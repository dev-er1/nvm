//! # Lexer (lexical analysis)
//!
//! A lexer is a program that turns code in any language
//! into a stream of tokens.
//!
//! ## Module contents
//! - [`token`] — the token enumeration and the token structure.
//! - [`err`] — the error enumeration and the structure of a single error.
pub mod err;
pub mod token;

use crate::{
    lexer::{
        err::{LexerError, LexerErrorKind},
        token::{Token, TokenKind},
    },
    position::Position,
    src::SourceCode,
    str_pool::StrPool,
};
use nvm_core::isa::{opcode::OperationCode, register::Register};

/// Lexer of the source code.
///
/// Parses the source code one character at a time, collecting the found
/// tokens into [`tokens`](Self::tokens) and errors into
/// [`errors`](Self::errors).
pub struct Lexer<'a> {
    /// The source code that the lexer parses.
    pub src: SourceCode,

    /// String pool into which identifiers (labels) are interned.
    pub str_pool: &'a mut StrPool,

    /// All tokens found.
    pub tokens: Vec<Token>,

    /// Errors found during tokenization.
    pub errors: Vec<LexerError>,

    /// Current byte position in the source code.
    index: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(src: SourceCode, str_pool: &'a mut StrPool) -> Self {
        Self {
            src,
            str_pool,
            tokens: Vec::new(),
            errors: Vec::new(),
            index: 0,
        }
    }

    /// Parses the source code into tokens.
    ///
    /// Tokens are collected into [`tokens`](Self::tokens) and errors
    /// into [`errors`](Self::errors). A [`TokenKind::End`] token,
    /// meaning the end of the source code, is always added at the end of the stream.
    pub fn tokenize(&mut self) -> &[Token] {
        loop {
            // Skip whitespace and comments.
            self.skip_trivia();

            if self.index >= self.src.source.len() {
                break;
            }

            self.next_token();
        }

        let pos = Position::new(self.index as u32, self.index as u32);
        self.tokens.push(Token::new(pos, TokenKind::End));

        &self.tokens
    }

    /// Parses a single token starting from the current position.
    ///
    /// If a character that is not part of any token is encountered,
    /// an error is added to [`errors`](Self::errors),
    /// and the character itself is skipped.
    fn next_token(&mut self) {
        let start = self.index;
        let first = self.bump().expect("called only if the symbol is present");

        let kind = match first {
            b'\n' => TokenKind::Newline,
            b',' => TokenKind::Comma,
            b':' => TokenKind::Colon,
            b'[' => TokenKind::OpeningSquareBracket,
            b']' => TokenKind::EndingSquareBracket,
            b'*' => TokenKind::Asterisk,

            // A dot before a digit starts a fractional number (.5).
            b'.' if self.is_digit_peek() => return self.lex_number(start, true),
            b'.' => TokenKind::Dot,

            // A sign before a digit starts a number with the given sign (-5).
            b'+' | b'-' if self.is_digit_peek() => return self.lex_number(start, false),
            b'+' => TokenKind::Plus,
            b'-' => TokenKind::Minus,

            b'0'..=b'9' => return self.lex_number(start, false),

            b'a'..=b'z' | b'A'..=b'Z' | b'_' => return self.lex_word(start),

            other => {
                self.push_error(start, LexerErrorKind::UnexpectedCharacter(other as char));
                return;
            }
        };

        let pos = Position::new(start as u32, self.index as u32);
        self.tokens.push(Token::new(pos, kind));
    }

    /// Skips whitespace (except line breaks) and comments.
    ///
    /// A comment starts with the `;` character and lasts until the end of the line.
    fn skip_trivia(&mut self) {
        loop {
            match self.peek() {
                Some(b' ' | b'\t' | b'\r') => {
                    self.bump();
                }
                Some(b';') => {
                    // Skip everything up to the line break; the line break itself
                    // is left alone: it will become a Newline token.
                    while let Some(byte) = self.peek() {
                        if byte == b'\n' {
                            break;
                        }
                        self.bump();
                    }
                }
                _ => break,
            }
        }
    }

    /// Parses a word: a register, a mnemonic, or an identifier.
    ///
    /// A word starts with a letter or an underscore. The first character
    /// has already been consumed as `start`.
    ///
    /// The word is classified in the following order:
    /// 1. `R0`..`R255` — a register;
    /// 2. a known mnemonic — [`TokenKind::Mnemonic`];
    /// 3. everything else — an identifier (for example, a label).
    fn lex_word(&mut self, start: usize) {
        while self
            .peek()
            .is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            self.bump();
        }

        let text = &self.src.source[start..self.index];

        let kind = if is_register_name(text) {
            let number = &text[1..];
            match number.parse::<u16>() {
                Ok(n) if n <= 255 => TokenKind::Register(Register(n as u8)),
                _ => {
                    self.push_error(start, LexerErrorKind::InvalidRegister(text.to_string()));
                    return;
                }
            }
        } else if let Ok(opcode) = text.parse::<OperationCode>() {
            TokenKind::Mnemonic(opcode)
        } else {
            TokenKind::Ident(self.str_pool.intern(text))
        };

        let pos = Position::new(start as u32, self.index as u32);
        self.tokens.push(Token::new(pos, kind));
    }

    /// Parses a number: an integer or a floating-point one.
    ///
    /// The first character (a digit, a sign, or a dot) has already been
    /// consumed into `start`. A fractional part is recognized only if a
    /// digit follows the dot; otherwise the dot becomes a separate token.
    fn lex_number(&mut self, start: usize, leading_dot: bool) {
        let mut is_float = leading_dot;

        // Integer part.
        while self.is_digit_peek() {
            self.bump();
        }

        // Fractional part.
        if self.peek() == Some(b'.') && self.peek_n(1).is_some_and(|b| b.is_ascii_digit()) {
            is_float = true;
            self.bump();
            while self.is_digit_peek() {
                self.bump();
            }
        }

        // Exponent: "e" or "E" with an optional sign.
        if matches!(self.peek(), Some(b'e') | Some(b'E')) {
            let mut offset = 1;
            if matches!(self.peek_n(offset), Some(b'+') | Some(b'-')) {
                offset += 1;
            }
            if self.peek_n(offset).is_some_and(|b| b.is_ascii_digit()) {
                is_float = true;
                self.bump();
                if matches!(self.peek(), Some(b'+') | Some(b'-')) {
                    self.bump();
                }
                while self.is_digit_peek() {
                    self.bump();
                }
            }
        }

        let text = &self.src.source[start..self.index];

        let kind = if is_float {
            match text.parse::<f64>() {
                Ok(value) if value.is_finite() => TokenKind::Float(value),
                _ => {
                    self.push_error(start, LexerErrorKind::InvalidFloat(text.to_string()));
                    return;
                }
            }
        } else {
            match text.parse::<i64>() {
                Ok(value) => TokenKind::Integer(value),
                Err(_) => {
                    self.push_error(start, LexerErrorKind::InvalidInteger(text.to_string()));
                    return;
                }
            }
        };

        let pos = Position::new(start as u32, self.index as u32);
        self.tokens.push(Token::new(pos, kind));
    }

    /// Adds an error to [`errors`](Self::errors).
    fn push_error(&mut self, start: usize, kind: LexerErrorKind) {
        let pos = Position::new(start as u32, self.index as u32);
        self.errors.push(LexerError::new(kind, pos));
    }

    /// Returns the byte at the current position without moving it.
    fn peek(&self) -> Option<u8> {
        self.src.source.as_bytes().get(self.index).copied()
    }

    /// Returns the byte `offset` bytes ahead of the current position.
    fn peek_n(&self, offset: usize) -> Option<u8> {
        self.src.source.as_bytes().get(self.index + offset).copied()
    }

    /// Returns the byte at the current position and moves the position forward.
    fn bump(&mut self) -> Option<u8> {
        let byte = self.peek()?;
        self.index += 1;
        Some(byte)
    }

    /// Whether the current byte is a digit.
    fn is_digit_peek(&self) -> bool {
        matches!(self.peek(), Some(b'0'..=b'9'))
    }
}

/// Whether the word is a register name: `R` followed by digits.
///
/// For example, `R0` — yes, `R256` — in form yes, but with an invalid
/// number, `r` and `r0x` — no.
fn is_register_name(text: &str) -> bool {
    text.len() > 1
        && matches!(text.as_bytes()[0], b'r' | b'R')
        && text[1..].bytes().all(|b| b.is_ascii_digit())
}
