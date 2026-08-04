//! В этом модуле находятся исполнители [`Command`].
pub mod help;
pub mod run;

use crate::{
    cmd::Command,
    command::{help::HelpArguments, run::RunArguments},
};

pub fn route(cmd: Command) -> i32 {
    match cmd {
        Command::Help {
            dont_show_banner,
            cmd,
        } => help::help(HelpArguments {
            dont_show_banner,
            cmd,
        }),
        Command::Run { file, time } => run::run(RunArguments { file, time }),
    }
}
