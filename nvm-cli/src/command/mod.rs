//! В этом модуле находятся исполнители [`Command`].
pub mod compile;
pub mod help;
pub mod run;

use crate::{
    ansiprint,
    cmd::Command,
    command::{compile::CompileArguments, help::HelpArguments, run::RunArguments},
};

use libnvm::NVM_VERSION;

pub fn route(cmd: Command) -> i32 {
    match cmd {
        Command::Help {
            dont_show_banner,
            cmd,
        } => help::help(HelpArguments {
            dont_show_banner,
            cmd,
        }),
        Command::Run { file, time, memory } => run::run(RunArguments { file, time, memory }),
        Command::Compile {
            file,
            output,
            time,
        } => compile::compile(CompileArguments {
            file,
            output,
            time,
        }),
        Command::Version => {
            ansiprint!("\x1b[1mv{NVM_VERSION}\x1b[0m");
            0
        }
    }
}
