// nvm-cli/src/command/run.rs
//
//! Исполнение команды `run`.
use std::{io::Read, time::Instant};

use libnvm::{BytecodeSource, NVMError, NVMErrorKind, NVMl};

use crate::ansi::ansi_supported;

pub struct RunArguments {
    pub file: String,
    pub time: bool,
}

pub fn run(args: RunArguments) -> i32 {
    let start = Instant::now();

    let source = if args.file == "-" {
        // Читаем байт-код из stdin.
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
    } else {
        BytecodeSource::File(args.file.into())
    };

    let nvm = NVMl::new(None);

    if let Err(e) = nvm.run(source) {
        report_error(e);
        return 1;
    }

    if args.time {
        println!("Execution time: {:?}.", start.elapsed());
    }

    0
}

/// Выводит ошибку исполнения в том же стиле, что и остальные ошибки CLI.
fn report_error(e: NVMError) {
    let e = NVMError::new(e.kind, e.instruction, ansi_supported());
    e.report();
}
