// nvm-asm/src/error.rs
//
//! Pretty-print ошибок в стиле `rustc`/`clang`.
use std::fmt::{self, Display, Formatter, Write};

use crate::{
    codegen::err::CodegenError, lexer::err::LexerError, parser::err::ParserError,
    position::Position, src::SourceCode,
};

/// Виды ошибок компиляции NVM Assembly.
#[derive(Debug, Clone)]
pub enum NvmASMErrorKind {
    /// Ошибка лексического анализа.
    LexerError(LexerError),

    /// Ошибка синтаксического анализа.
    ParserError(ParserError),

    /// Ошибка кодогенератора.
    CodegenError(CodegenError),
}

impl Display for NvmASMErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::LexerError(err) => write!(f, "{err}"),
            Self::ParserError(err) => write!(f, "{err}"),
            Self::CodegenError(err) => write!(f, "{err}"),
        }
    }
}

/// Ошибка компиляции вместе с фрагментом исходного кода.
#[derive(Debug, Clone)]
pub struct NvmASMError {
    /// Позиция ошибки в исходном коде.
    position: Position,

    /// Вид ошибки.
    kind: NvmASMErrorKind,

    /// Поддерживает ли терминал ANSI-цвета.
    have_ansi: bool,

    /// Имя файла, в котором найдена ошибка.
    filename: Option<&'static str>,

    /// Исходный код, в котором найдена ошибка.
    src: SourceCode,
}

impl NvmASMError {
    /// Создаёт ошибку компиляции.
    pub fn error(
        position: Position,
        kind: NvmASMErrorKind,
        have_ansi: bool,
        filename: Option<&'static str>,
        src: SourceCode,
    ) -> Self {
        Self {
            position,
            kind,
            have_ansi,
            filename,
            src,
        }
    }

    /// Печатает ошибку в консоль в стиле `rustc`/`clang`.
    pub fn report(&self) {
        println!("{}", self.format());
    }

    /// Формирует текст ошибки без вывода в консоль.
    ///
    /// Полезно для сохранения ошибки в лог или для тестов.
    ///
    /// ```text
    /// Error: unexpected character: '!'.
    /// test.nasm -> 1:10..1:11
    ///   |
    /// 1 | MOVE R0, !
    ///   |          ^
    /// ```
    pub fn format(&self) -> String {
        let (start_line, start_col) = self.src.lookup_coordinates(self.position.start);
        let (end_line, end_col) = self.src.lookup_coordinates(self.position.end);

        let mut out = String::new();

        self.write_header(&mut out, start_line, start_col, end_line, end_col);
        self.write_source(&mut out, start_line, start_col, end_line, end_col);

        out
    }

    /// Заголовок ошибки: `Error: <сообщение>.` и `файл -> позиция`.
    fn write_header(
        &self,
        out: &mut String,
        start_line: u32,
        start_col: u32,
        end_line: u32,
        end_col: u32,
    ) {
        let error = self.styled("1;31", "Error");
        let message = self.styled("1", &self.kind.to_string());
        writeln!(out, "{error}: {message}.").unwrap();

        let position = format!("{start_line}:{start_col}..{end_line}:{end_col}");
        match self.filename {
            Some(file) => {
                let file = self.styled("36", file);
                let position = self.styled("1", &position);
                writeln!(out, "{file} -> {position}").unwrap();
            }
            None => {
                let position = self.styled("1", &position);
                writeln!(out, "-> {position}").unwrap();
            }
        }
    }

    /// Фрагмент исходного кода с подчёркиванием ошибки.
    fn write_source(
        &self,
        out: &mut String,
        start_line: u32,
        start_col: u32,
        end_line: u32,
        end_col: u32,
    ) {
        let width = end_line.to_string().len();

        self.write_gutter(out, width);

        if start_line == end_line {
            let line = self.line_text(start_line);
            self.write_numbered_line(out, start_line, width, &line);
            let carets = (end_col - start_col).max(1) as usize;
            self.write_carets(out, width, start_col, carets);
            return;
        }

        // Многострочный промежуток: первая строка, середина, последняя.
        let first = self.line_text(start_line);
        self.write_numbered_line(out, start_line, width, &first);
        let carets = first.len().saturating_sub((start_col - 1) as usize);
        self.write_carets(out, width, start_col, carets.max(1));

        for line_no in start_line + 1..end_line {
            let text = self.line_text(line_no);
            self.write_numbered_line(out, line_no, width, &text);
            self.write_carets(out, width, 1, text.len().max(1));
        }

        let last = self.line_text(end_line);
        self.write_numbered_line(out, end_line, width, &last);
        let carets = (end_col - 1).max(1) as usize;
        self.write_carets(out, width, 1, carets);
    }

    /// Строка-разделитель над фрагментом кода.
    fn write_gutter(&self, out: &mut String, width: usize) {
        writeln!(out, "{} |", " ".repeat(width)).unwrap();
    }

    /// Строка исходного кода с номером строки.
    fn write_numbered_line(&self, out: &mut String, line: u32, width: usize, text: &str) {
        let number = self.styled("1", &format!("{line:>width$}"));
        writeln!(out, "{number} | {text}").unwrap();
    }

    /// Строка подчёркивания ошибки (`^`).
    fn write_carets(&self, out: &mut String, width: usize, column: u32, count: usize) {
        let padding = " ".repeat((column - 1) as usize);
        let carets = self.styled("1;31", &"^".repeat(count));
        writeln!(out, "{} | {padding}{carets}", " ".repeat(width)).unwrap();
    }

    /// Текст строки с указанным номером без перевода строки.
    fn line_text(&self, line: u32) -> String {
        let start = self.src.line_starts[(line - 1) as usize] as usize;
        let end = self
            .src
            .line_starts
            .get(line as usize)
            .map_or(self.src.source.len(), |&offset| offset as usize);

        self.src.source[start..end]
            .trim_end_matches(['\r', '\n'])
            .to_string()
    }

    /// Оборачивает текст в ANSI-код, если терминал его поддерживает.
    fn styled(&self, code: &str, text: &str) -> String {
        if self.have_ansi {
            format!("\x1b[{code}m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }
}
