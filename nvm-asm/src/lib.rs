//! # `nvm-asm`
//!
//! `nvm-asm` — крейт в проекте NVM, для компиляции текстового
//! формата NVM Bytecode (назовём его NVM Assembly) в NVM Bytecode.
//!
//! ## Содержимое
//! - [`src`] — хранение исходного кода.
//! - [`str_pool`] — пул строк.
//! - [`position`] — структура позиции токена в исходном коде.
//! - [`lexer`] — лексер (лексический анализ).
pub mod error;
pub mod lexer;
pub mod position;
pub mod src;
pub mod str_pool;
