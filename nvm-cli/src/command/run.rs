// nvm-cli/src/command/run.rs
//
//! Execution of the `run` command.
use std::{io::Read, path::Path, time::Instant};

use libnvm::{BytecodeSource, NVMAssembler, NVMError, NVMErrorKind, NVMl};

use crate::{ansi::ansi_supported, ansiprint};

pub struct RunArguments {
    pub file: String,
    pub time: bool,
    pub memory: Option<usize>,
}

pub fn run(args: RunArguments) -> i32 {
    let start = Instant::now();

    let source = if args.file == "-" {
        // Read the bytecode from stdin.
        let mut bytes = Vec::new();
        if let Err(e) = std::io::stdin().read_to_end(&mut bytes) {
            report_error(NVMError::new(
                NVMErrorKind::IoError(e),
                None,
                ansi_supported(),
            ));
            return 1;
        }
        BytecodeSource::Bytes(bytes)
    } else if is_assembly(&args.file) {
        // An NVM Assembly file: compile it into instructions and execute.
        let source = match std::fs::read_to_string(&args.file) {
            Ok(source) => source,
            Err(e) => {
                report_error(NVMError::new(
                    NVMErrorKind::IoError(e),
                    None,
                    ansi_supported(),
                ));
                return 1;
            }
        };

        let instructions = match NVMAssembler::assemble(&source) {
            Ok(instructions) => instructions,
            Err(e) => {
                e.report();
                return 1;
            }
        };

        BytecodeSource::Instructions(instructions)
    } else {
        BytecodeSource::File(args.file.into())
    };

    let nvm = if let Some(memory) = args.memory {
        NVMl::with_memory_size(memory)
    } else {
        NVMl::new()
    };

    if let Err(e) = nvm.run(source) {
        report_error(e);
        return 1;
    }

    if args.time {
        ansiprint!(
            "\x1b[1;36mFinished\x1b[0m in \x1b[1m{:?}\x1b[0m",
            start.elapsed()
        );
    }

    0
}

/// Whether the file is an NVM Assembly source (`.na`).
fn is_assembly(file: &str) -> bool {
    Path::new(file)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("na"))
}

/// Prints an execution error in the same style as the other CLI errors.
fn report_error(e: NVMError) {
    let e = NVMError::new(e.kind, e.instruction, ansi_supported());
    e.report();
}
