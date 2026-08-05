// nvm-cli/src/error.rs
//
//! Ошибки CLI.
use std::{
    fmt::{self, Display, Formatter},
    iter::repeat_n,
};

use crate::ansiprint;

pub enum CLIErrorKind {
    /// Неизвестный флаг.
    UnknownFlag(String),

    /// Неожиданное значение для флага.
    ///
    /// Ошибка происходит, когда даётся значение флагу,
    /// который не требует значения.
    ///
    /// ## Пример
    /// ```text
    /// nvm help --show-banner 67
    ///                        ^^
    /// ```
    UnexpectedValue(
        // Флаг.
        String,
        // Значение.
        String,
    ),

    /// Неизвестная команда.
    UnknownCommand(String),

    /// Нету значения для флага, который требует значение.
    MissingValueForFlag(
        // Флаг.
        String,
    ),

    /// Нету значения для команды, которая требует значение.
    MissingValueForCommand(
        // Команда.
        String,
    ),

    /// Неожиданный аргумент.
    ///
    /// Ошибка происходит, когда команде передано больше аргументов,
    /// чем она ожидает.
    ///
    /// ## Пример
    /// ```text
    /// nvm run prog.nb extra
    ///              ^^^^^^^^
    /// ```
    UnexpectedArgument(
        // Аргумент.
        String,
    ),

    /// Неправильное значение.
    /// 
    /// Например, флаг ожидает [`u64`], а получает отрицательное число.
    InvalidValue(
        /// Имя флага.
        String,
    )
}

impl Display for CLIErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownFlag(flag) => write!(f, "unknown flag: '{flag}'"),
            Self::UnexpectedValue(flag, value) => {
                write!(f, "unexpected value in flag '{flag}': '{value}'")
            }
            Self::UnknownCommand(cmd) => write!(f, "unknown command: '{cmd}'"),
            Self::MissingValueForFlag(flag) => write!(f, "missing value for flag '{flag}'"),
            Self::MissingValueForCommand(cmd) => write!(f, "missing value for command '{cmd}'"),
            Self::UnexpectedArgument(arg) => write!(f, "unexpected argument: '{arg}'"),
            Self::InvalidValue(flag) => write!(f, "invalid value for flag '{flag}'"),
        }
    }
}

pub struct CLIError {
    pub kind: CLIErrorKind,

    /// Аргументы CLI для Pretty-Print ошибки.
    pub raw_args: Option<Vec<String>>,

    /// Какие аргументы в `raw_args` неправильные.
    /// Нужно для вывода ошибки.
    pub which_args: Option<Vec<usize>>,
}

impl CLIError {
    pub fn new(
        kind: CLIErrorKind,
        raw_args: Option<Vec<String>>,
        which_args: Option<Vec<usize>>,
    ) -> Self {
        Self {
            kind,
            raw_args,
            which_args,
        }
    }

    pub fn report(&self) {
        ansiprint!("\x1b[1;31mError\x1b[0m: \x1b[1m{}\x1b[0m.", self.kind);
        println!();

        if let Some(raw_args) = &self.raw_args
            && let Some(which_args) = &self.which_args
        {
            // Исходная командная строка.
            let args = raw_args.join(" ");

            ansiprint!("\x1b[36m-->\x1b[0m  nvm {args}");

            // Указатели на неправильные аргументы.
            let mut pointers = String::new();

            for (i, arg) in raw_args.iter().enumerate() {
                if which_args.contains(&(i)) {
                    pointers.extend(repeat_n('^', arg.len()));
                } else {
                    pointers.extend(repeat_n(' ', arg.len()));
                }

                // Пробел между аргументами.
                if i + 1 < raw_args.len() {
                    pointers.push(' ');
                }
            }

            ansiprint!("         \x1b[1;31m{pointers}\x1b[0m");
            println!();
        }
    }
}
