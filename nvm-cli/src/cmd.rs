// nvm-cli/src/cmd.rs
//
//! Определение команд.

pub enum Command {
    // `nvm help [--info <command>] [--dont-show-banner]`
    Help {
        /// Если `true` — не показывать баннер.
        dont_show_banner: bool,

        /// Команда, о которой нужно вывести информацию.
        cmd: Option<String>,
    },

    /// `nvm run <file> [--time] [--memory <bytes]`
    Run {
        /// Путь к файлу, который нужно выполнить.
        file: String,

        /// Если `true` — выводить время выполнения.
        time: bool,

        /// Сколько выделить памяти на выполнение программы.
        memory: Option<usize>,
    },

    /// `nvm compile <file> [--output <path>] [--time]`
    Compile {
        /// Путь к файлу NVM Assembly (.na).
        file: String,

        /// Куда записать результирующий .nb файл.
        ///
        /// Если `None` — рядом с исходным файлом.
        output: Option<String>,

        /// Если `true` — выводить время компиляции.
        time: bool,
    },

    /// `nvm version`
    Version,
}

// `*Info` и `const COMMAND` нужны только для вывода информации об
// любой команде.

#[derive(Debug, Clone, Copy)]
pub struct FlagInfo {
    pub usage: &'static str,
    pub description: &'static str,
}

pub struct CommandInfo {
    pub name: &'static str,

    pub usage: &'static str,
    pub description: &'static str,
    pub flags: &'static [FlagInfo],
}

pub const COMMAND: &[CommandInfo] = &[
    CommandInfo {
        name: "help",
        usage: "help",
        description: "Display help.",
        flags: &[
            FlagInfo {
                usage: "--dont-show-banner",
                description: "Don't show NVM banner.",
            },
            FlagInfo {
                usage: "--info <command>",
                description: "Show information about <command>.",
            },
        ],
    },
    CommandInfo {
        name: "run",
        usage: "run <file>",
        description: "Execute NVM Bytecode.",
        flags: &[
            FlagInfo {
                usage: "--time",
                description: "Show execution time.",
            },
            FlagInfo {
                usage: "--memory <bytes>",
                description: "Allocate the specified amount of memory for program execution.",
            },
        ],
    },
    CommandInfo {
        name: "compile",
        usage: "compile <file>",
        description: "Compile NVM Assembly (.na) to NVM Bytecode (.nb).",
        flags: &[
            FlagInfo {
                usage: "--output <path>",
                description: "Write the output to <path> instead of the default .nb file.",
            },
            FlagInfo {
                usage: "--time",
                description: "Show compilation time.",
            },
        ],
    },
    CommandInfo {
        name: "version",
        usage: "version",
        description: "Print NVM version.",
        flags: &[],
    },
];
