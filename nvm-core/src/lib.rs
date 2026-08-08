//! # `nvm-core`
//!
//! This crate is the **core of *NVM*** (***N**ot **V**irtual **M**achine*).
//! It contains:
//! - [`isa`] — the NVM *Instruction Set Architecture (**ISA**)*.
//! - [`vm`] — the NVM virtual machine itself.
//! - [`loader`] — the loader of files in the NVM Bytecode format (see `docs/File-Format/FILE-FORMAT.ru.md`).
pub mod error;
pub mod isa;
pub mod loader;
pub mod vm;

pub const NVM_VERSION: &str = env!("CARGO_PKG_VERSION");
