// nvm-cli/src/cmd.rs
//
//! Definition of the commands.

pub enum Command {
    // `nvm help [--info <command>] [--dont-show-banner]`
    Help {
        /// If `true` — do not show the banner.
        dont_show_banner: bool,

        /// The command to show information about.
        cmd: Option<String>,
    },

    /// `nvm run <file> [--time] [--memory <bytes]`
    Run {
        /// Path to the file to execute.
        file: String,

        /// If `true` — print the execution time.
        time: bool,

        /// How much memory to allocate for program execution.
        memory: Option<usize>,
    },

    /// `nvm compile <file> [--output <path>] [--time]`
    Compile {
        /// Path to the NVM Assembly (.na) file.
        file: String,

        /// Where to write the resulting .nb file.
        ///
        /// If `None` — next to the source file.
        output: Option<String>,

        /// If `true` — print the compilation time.
        time: bool,
    },

    /// `nvm version`
    Version,
}

// `*Info` and `const COMMAND` are needed only to display information about
// any command.

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
