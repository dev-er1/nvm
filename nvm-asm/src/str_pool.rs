// nvm-asm/src/str_pool.rs
//
//! Пул строк.
use std::collections::HashMap;

use crate::src::SourceCode;

/// Идентификатор строки
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
pub struct StrId(pub u32);

#[derive(Debug, Clone, Default)]
pub struct StrPool {
    /// Хранилище строк. Индекс в векторе — это и есть наш StrId.
    storage: Vec<Box<str>>,

    /// Карта для быстрого поиска.
    lookup: HashMap<String, StrId>,
}

impl StrPool {
    pub fn from_source(src: &SourceCode) -> Self {
        let code_len = src.source.len();

        let estimated_lines = src.line_starts.len();
        let estimated_words = (code_len / 6).max(estimated_lines);

        let capacity = estimated_words.max(32);

        Self {
            storage: Vec::with_capacity(capacity),
            lookup: HashMap::with_capacity(capacity),
        }
    }

    pub fn with_capacity(items: usize) -> Self {
        Self {
            storage: Vec::with_capacity(items),
            lookup: HashMap::with_capacity(items),
        }
    }

    pub fn intern(&mut self, s: &str) -> StrId {
        // Если строка уже есть, просто возвращаем её ID
        if let Some(&id) = self.lookup.get(s) {
            return id;
        }

        let id = StrId(self.storage.len() as u32);

        let boxed_str = s.to_string().into_boxed_str();
        self.storage.push(boxed_str);

        self.lookup.insert(s.to_string(), id);

        id
    }

    /// Дступ к строке по ID за O(1) без хэширования
    #[inline]
    pub fn get(&self, id: StrId) -> &str {
        &self.storage[id.0 as usize]
    }
}
