// nvm-cli/src/command/compile.rs
//
//! Execution of the `compile` command.
use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use libnvm::{NVMAssembler, NVMError, NVMErrorKind};

use crate::{ansi::ansi_supported, ansiprint};

pub struct CompileArguments {
    pub file: String,
    pub output: Option<String>,
    pub time: bool,
}

pub fn compile(args: CompileArguments) -> i32 {
    let start = Instant::now();

    let source = match fs::read_to_string(&args.file) {
        Ok(source) => source,
        Err(e) => {
            report_io_error(&args.file, e);
            return 1;
        }
    };

    let bytecode = match NVMAssembler::assemble_to_bytecode(&source) {
        Ok(bytecode) => bytecode,
        Err(e) => {
            e.report();
            return 1;
        }
    };

    let output = args
        .output
        .map(PathBuf::from)
        .unwrap_or_else(|| default_output(&args.file));

    if let Err(e) = fs::write(&output, bytecode) {
        report_io_error(output.to_string_lossy().as_ref(), e);
        return 1;
    }

    ansiprint!(
        "\x1b[1;32mCompiled\x1b[0m \x1b[1m{}\x1b[0m \x1b[2m->\x1b[0m \x1b[1m{}\x1b[0m",
        args.file,
        output.display()
    );

    if args.time {
        ansiprint!(
            "\x1b[1;36mFinished\x1b[0m in \x1b[1m{:?}\x1b[0m",
            start.elapsed()
        );
    }

    0
}

/// Where to write the result by default: the same path as the source,
/// but with the `.nb` extension.
fn default_output(input: &str) -> PathBuf {
    Path::new(input).with_extension("nb")
}

/// Prints a file read/write error in the same style as the other CLI errors.
fn report_io_error(path: &str, e: std::io::Error) {
    // Adding the path to the message, since the io error itself doesn't know it.
    let e = std::io::Error::new(e.kind(), format!("{path}: {e}"));
    NVMError::new(NVMErrorKind::IoError(e), None, ansi_supported()).report();
}
