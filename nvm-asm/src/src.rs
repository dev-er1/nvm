// nvm-core/src/src.rs
//
//! Хранение исходного кода и работа с координатами текста.
use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct SourceCode {
    /// Полный текст исходного кода в формате UTF-8
    pub source: String,

    /// Байтовые смещения, на которых начинается каждая строка.
    /// Индекс в этом векторе + 1 = номер строки.
    pub line_starts: Vec<u32>,
}

impl SourceCode {
    pub fn new(source: String) -> Self {
        let mut line_starts = vec![0]; // Первая строка всегда начинается с 0-го байта

        // Быстро сканируем байты в поисках переноса строки '\n'
        for (pos, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push((pos + 1) as u32);
            }
        }

        Self {
            source,
            line_starts,
        }
    }

    pub fn lookup_coordinates(&self, byte_index: u32) -> (u32, u32) {
        let line_idx = match self.line_starts.binary_search(&byte_index) {
            Ok(idx) => idx,
            Err(idx) => idx - 1,
        };

        let line_start_byte = self.line_starts[line_idx];
        let line = line_idx + 1;
        let column = byte_index - line_start_byte + 1;

        (line as u32, column)
    }
}

impl Display for SourceCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.source)
    }
}
