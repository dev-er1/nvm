//! # nvm-core
//!
//! Этот крейт — **ядро *NVM*** (***N**ot **V**irtual **M**achine*).
//! Здесь находится:
//! - [`isa`] — NVM *Instruction Set Architecture (**ISA**)*.
//! - [`vm`] — сама виртуальная машина NVM.
//! - [`loader`] — загрузчик файлов в формате NVM Bytecode (см. `docs/File-Format/FILE-FORMAT.ru.md`).
pub mod error;
pub mod isa;
pub mod loader;
pub mod vm;

pub const NVM_VERSION: &str = env!("CARGO_PKG_VERSION");
