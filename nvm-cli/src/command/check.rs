// nvm-cli/src/command/check.rs
//
//! Execution of the `check` command.
use std::{fs, time::Instant};

use libnvm::{NVMAssembler, NVMError, NVMErrorKind};

use crate::{ansi::ansi_supported, ansiprint};

pub struct CheckArguments {
    pub file: String,
    pub time: bool,
}

pub fn check(args: CheckArguments) -> i32 {
    let start = Instant::now();

    let source = match fs::read_to_string(&args.file) {
        Ok(source) => source,
        Err(e) => {
            report_io_error(&args.file, e);
            return 1;
        }
    };

    if let Err(e) = NVMAssembler::assemble(&source) {
        e.report();
        return 1;
    }

    ansiprint!("\x1b[1;32mChecked\x1b[0m \x1b[1m{}\x1b[0m", args.file);

    if args.time {
        ansiprint!(
            "\x1b[1;36mFinished\x1b[0m in \x1b[1m{:?}\x1b[0m",
            start.elapsed()
        );
    }

    0
}

/// Prints a file read error in the same style as the other CLI errors.
fn report_io_error(path: &str, e: std::io::Error) {
    // Adding the path to the message, since the io error itself doesn't know it.
    let e = std::io::Error::new(e.kind(), format!("{path}: {e}"));
    NVMError::new(NVMErrorKind::IoError(e), None, ansi_supported()).report();
}