//! # Лексер (лексический анализ)
//!
//! Лексер — это программа, которая превращает код на любом языке
//! в поток токенов.
//!
//! ## Содержимое модуля
//! - [`token`] — перечисление токенов и структура токена.
//! - [`err`] — перечисление ошибок и структура одной ошибки.
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

/// Лексер исходного кода.
///
/// Разбирает исходный код по одному символу, собирая найденные
/// токены в [`tokens`](Self::tokens), а ошибки — в
/// [`errors`](Self::errors).
pub struct Lexer<'a> {
    /// Исходный код, который разбирает лексер.
    pub src: SourceCode,

    /// Пул строк, в который интернируются идентификаторы (метки).
    pub str_pool: &'a mut StrPool,

    /// Все найденные токены.
    pub tokens: Vec<Token>,

    /// Ошибки, обнаруженные при разборе.
    pub errors: Vec<LexerError>,

    /// Текущая позиция байта в исходном коде.
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

    /// Разбирание исходного кода на токены.
    ///
    /// Токены складываются в [`tokens`](Self::tokens), а ошибки —
    /// в [`errors`](Self::errors). В конце потока всегда добавляется
    /// токен [`TokenKind::End`], означающий конец исходного кода.
    pub fn tokenize(&mut self) -> &[Token] {
        loop {
            // Пропускаем пробелы и комментарии, чтобы не плодить
            // лишние токены.
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

    /// Разбирание одного токена, начиная с текущей позиции.
    ///
    /// Если встречен символ, который не является частью ни одного
    /// токена, в [`errors`](Self::errors) добавляется ошибка,
    /// а сам символ пропускается.
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

            // Точка перед цифрой начинает дробное число (.5).
            b'.' if self.is_digit_peek() => return self.lex_number(start, true),
            b'.' => TokenKind::Dot,

            // Знак перед цифрой начинает число с указанным знаком (-5).
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

    /// Пропускает пробелы (кроме переносов строк) и комментарии.
    ///
    /// Комментарий начинается с символа `;` и длится до конца строки.
    fn skip_trivia(&mut self) {
        loop {
            match self.peek() {
                Some(b' ' | b'\t' | b'\r') => {
                    self.bump();
                }
                Some(b';') => {
                    // Пропускаем всё до переноса строки, сам перенос
                    // не трогаем: он станет токеном Newline.
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

    /// Разбирает слово: регистр, мнемонику или идентификатор.
    ///
    /// Слово начинается с буквы или подчёркивания. Первым символом
    /// уже считан сам `start`.
    ///
    /// Классификация слова происходит в следующем порядке:
    /// 1. `R0`..`R255` — регистр;
    /// 2. известная мнемоника — [`TokenKind::Mnemonic`];
    /// 3. всё остальное — идентификатор (например, метка).
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

    /// Разбирает число: целое или с плавающей точкой.
    ///
    /// Первым символом (цифра, знак или точка) уже считан в
    /// `start`. Дробная часть распознаётся только если за точкой
    /// следует цифра, иначе точка становится отдельным токеном.
    fn lex_number(&mut self, start: usize, leading_dot: bool) {
        let mut is_float = leading_dot;

        // Целая часть.
        while self.is_digit_peek() {
            self.bump();
        }

        // Дробная часть.
        if self.peek() == Some(b'.') && self.peek_n(1).is_some_and(|b| b.is_ascii_digit()) {
            is_float = true;
            self.bump();
            while self.is_digit_peek() {
                self.bump();
            }
        }

        // Экспонента: "e" или "E" с необязательным знаком.
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

    /// Добавляет ошибку лексического анализа в [`errors`](Self::errors).
    fn push_error(&mut self, start: usize, kind: LexerErrorKind) {
        let pos = Position::new(start as u32, self.index as u32);
        self.errors.push(LexerError::new(kind, pos));
    }

    /// Возвращает байт на текущей позиции, не сдвигая её.
    fn peek(&self) -> Option<u8> {
        self.src.source.as_bytes().get(self.index).copied()
    }

    /// Возвращает байт на `offset` байтов впереди текущей позиции.
    fn peek_n(&self, offset: usize) -> Option<u8> {
        self.src.source.as_bytes().get(self.index + offset).copied()
    }

    /// Возвращает байт на текущей позиции и сдвигает позицию вперёд.
    fn bump(&mut self) -> Option<u8> {
        let byte = self.peek()?;
        self.index += 1;
        Some(byte)
    }

    /// Является ли текущий байт цифрой.
    fn is_digit_peek(&self) -> bool {
        matches!(self.peek(), Some(b'0'..=b'9'))
    }
}

/// Является ли слово именем регистра: `R`, за которым идут цифры.
///
/// Например, `R0` — да, `R256` — по форме да, но с недопустимым
/// номером, `r` и `r0x` — нет.
fn is_register_name(text: &str) -> bool {
    text.len() > 1
        && matches!(text.as_bytes()[0], b'r' | b'R')
        && text[1..].bytes().all(|b| b.is_ascii_digit())
}
