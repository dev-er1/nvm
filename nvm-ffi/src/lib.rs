// nvm-ffi/src/lib.rs
//
//! C ABI wrapper over `libnvm`.
//!
//! Enables using NVM from C, C++ and other languages
//! (Python via `ctypes`, Node via FFI, Go via `cgo`, etc.).
//!
//! ## Conventions
//!
//! - All functions return `0` (the wrapper `NVM_FFI_OK`) on success and
//!   a non-zero error code on failure (see the `NVM_FFI_ERR_*` constants).
//! - The text of the last thread error is obtained via
//!   [`nvm_last_error`]. The error lives until the next FFI call.
//! - No Rust-allocated pointers are passed out:
//!   results are written into a consumer-provided buffer
//!   (the "two calls" pattern: first find out the size, then write).
//! - Buffers are passed as `(pointer, capacity)`. On the
//!   "buffer too small" error, the required size is still written into `written`.
//! - A panic across FFI is UB, so the function bodies are wrapped in
//!   [`catch_unwind`].
use std::{
    cell::RefCell,
    ffi::{CStr, CString, c_char},
    panic::{AssertUnwindSafe, catch_unwind},
    ptr,
    sync::LazyLock,
};

use libnvm::{BytecodeSource, DEFAULT_MEMORY_SIZE, NVM_VERSION, NVMAssembler, NVMError, NVMl};

/// Success.
pub const NVM_FFI_OK: i32 = 0;

/// An NVM Assembly compilation error.
pub const NVM_FFI_ERR_COMPILE: i32 = 1;

/// A bytecode execution error.
pub const NVM_FFI_ERR_RUN: i32 = 2;

/// An FFI contract violation (NULL pointer, small buffer, etc.).
pub const NVM_FFI_ERR_CONTRACT: i32 = 3;

/// A panic inside FFI.
pub const NVM_FFI_ERR_PANIC: i32 = 4;

const PANIC_MESSAGE: &str = "panic in nvm-ffi";

thread_local! {
    /// The last compilation result ([`nvm_compile`]).
    static LAST_COMPILED: RefCell<Option<Vec<u8>>> = const { RefCell::new(None) };

    /// The text of the last error of the current thread.
    static LAST_ERROR: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// `NVM_VERSION` as a C string (with a terminating NUL).
static VERSION_C: LazyLock<CString> =
    LazyLock::new(|| CString::new(NVM_VERSION).expect("NVM_VERSION contains nul"));

/// Runs an FFI function, converting a panic into an error code.
fn guard(err_code: i32, f: impl FnOnce() -> Result<(), String>) -> i32 {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(())) => NVM_FFI_OK,
        Ok(Err(message)) => {
            set_last_error(message);
            err_code
        }
        Err(_) => {
            set_last_error(PANIC_MESSAGE.to_string());
            NVM_FFI_ERR_PANIC
        }
    }
}

fn set_last_error(message: String) {
    LAST_ERROR.with(|e| *e.borrow_mut() = Some(message));
}

/// Reads a NUL-terminated C string as the source text.
fn read_source(source: *const c_char) -> Result<String, String> {
    if source.is_null() {
        return Err("source is null".to_string());
    }

    // A NUL inside the source truncates the string — our NUL-string semantics.
    Ok(String::from_utf8_lossy(unsafe { CStr::from_ptr(source) }.to_bytes()).into_owned())
}

/// Compiles the source and returns the `.nb` bytes.
fn compile_to_bytes(source: *const c_char) -> Result<Vec<u8>, String> {
    let source = read_source(source)?;
    NVMAssembler::assemble_to_bytecode(&source).map_err(|e| e.format())
}

/// Copies binary data into the consumer's buffer.
///
/// The required size is written into `written`. If the buffer is too small —
/// an error, but `written` still reports the required size.
fn copy_bytes_to_buffer(
    bytes: &[u8],
    buf: *mut u8,
    cap: usize,
    written: *mut usize,
) -> Result<(), String> {
    if written.is_null() {
        return Err("written is null".to_string());
    }
    unsafe { *written = bytes.len() };

    if bytes.len() > cap {
        return Err(format!(
            "buffer too small: need {} bytes, got {cap}",
            bytes.len()
        ));
    }
    if bytes.is_empty() {
        return Ok(());
    }
    if buf.is_null() {
        return Err("buf is null".to_string());
    }

    unsafe { ptr::copy_nonoverlapping(bytes.as_ptr(), buf, bytes.len()) };
    Ok(())
}

/// Copies a string (including the terminating NUL) into the consumer's buffer.
///
/// The required size (including the NUL) is written into `written`.
fn copy_str_to_buffer(
    s: &str,
    buf: *mut u8,
    cap: usize,
    written: *mut usize,
) -> Result<(), String> {
    if written.is_null() {
        return Err("written is null".to_string());
    }

    // The NUL terminator is mandatory, so one more than the string length.
    let needed = s.len() + 1;
    unsafe { *written = needed };

    if needed > cap {
        return Err(format!("buffer too small: need {needed} bytes, got {cap}"));
    }
    if buf.is_null() {
        return Err("buf is null".to_string());
    }

    unsafe {
        ptr::copy_nonoverlapping(s.as_ptr(), buf, s.len());
        *buf.add(s.len()) = 0;
    }
    Ok(())
}

/// Assembles the text of an execution error.
fn format_nvm_error(e: &NVMError) -> String {
    match &e.instruction {
        Some(instruction) => format!("{}. --> {instruction}", e.kind),
        None => e.kind.to_string(),
    }
}

/// Executes the `.nb` bytes with the given memory size.
fn run_bytecode(bytes: &[u8], memory_size: usize) -> Result<(), String> {
    NVMl::with_memory_size(memory_size)
        .run(BytecodeSource::Bytes(bytes.to_vec()))
        .map_err(|e| format_nvm_error(&e))
}

/// The NVM version as a C string (does not need to be freed).
///
/// ## Example
/// ```c
/// printf("NVM %s\n", nvm_version());
/// ```
#[unsafe(no_mangle)]
pub extern "C" fn nvm_version() -> *const c_char {
    VERSION_C.as_ptr()
}

/// Compiles an NVM Assembly source and saves the `.nb` bytes
/// in a thread-local buffer (replacing the previous result).
///
/// The result size is in [`nvm_compile_size`], reading in [`nvm_compile_write`].
#[unsafe(no_mangle)]
pub extern "C" fn nvm_compile(source: *const c_char) -> i32 {
    guard(NVM_FFI_ERR_COMPILE, || {
        let bytes = compile_to_bytes(source)?;
        LAST_COMPILED.with(|m| *m.borrow_mut() = Some(bytes));
        Ok(())
    })
}

/// Writes the size of the compilation result into `size` (without
/// re-allocating). The result itself is not stored.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn nvm_compile_size(source: *const c_char, size: *mut usize) -> i32 {
    guard(NVM_FFI_ERR_COMPILE, || {
        if size.is_null() {
            return Err("size is null".to_string());
        }

        let bytes = compile_to_bytes(source)?;
        unsafe { *size = bytes.len() };
        Ok(())
    })
}

/// Compiles the source and writes the `.nb` bytes into `buf`.
///
/// `written` receives the required size. If the buffer is too small — `NVM_FFI_ERR_CONTRACT`.
#[unsafe(no_mangle)]
pub extern "C" fn nvm_compile_write(
    source: *const c_char,
    buf: *mut u8,
    cap: usize,
    written: *mut usize,
) -> i32 {
    guard(NVM_FFI_ERR_COMPILE, || {
        let bytes = compile_to_bytes(source)?;
        copy_bytes_to_buffer(&bytes, buf, cap, written)
    })
}

/// Executes the `.nb` bytes with the default memory.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn nvm_run_bytecode(bytes: *const u8, len: usize) -> i32 {
    guard(NVM_FFI_ERR_RUN, || {
        if len > 0 && bytes.is_null() {
            return Err("bytes is NULL".to_string());
        }
        // `run_bytecode` itself will return a loader error when len == 0.
        run_bytecode(
            unsafe { std::slice::from_raw_parts(bytes, len) },
            DEFAULT_MEMORY_SIZE,
        )
    })
}

/// Executes the `.nb` bytes with the specified memory size.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn nvm_run_bytecode_mem(bytes: *const u8, len: usize, memory_size: usize) -> i32 {
    guard(NVM_FFI_ERR_RUN, || {
        if len > 0 && bytes.is_null() {
            return Err("bytes is NULL".to_string());
        }
        run_bytecode(
            unsafe { std::slice::from_raw_parts(bytes, len) },
            memory_size,
        )
    })
}

/// Compiles the source and immediately executes it.
///
/// Compilation errors return `NVM_FFI_ERR_COMPILE`,
/// execution errors — `NVM_FFI_ERR_RUN`.
#[unsafe(no_mangle)]
pub extern "C" fn nvm_run_source(source: *const c_char) -> i32 {
    let compiled = catch_unwind(AssertUnwindSafe(|| {
        read_source(source).and_then(|src| NVMAssembler::assemble(&src).map_err(|e| e.format()))
    }));

    let instructions = match compiled {
        Ok(Ok(instructions)) => instructions,
        Ok(Err(message)) => {
            set_last_error(message);
            return NVM_FFI_ERR_COMPILE;
        }
        Err(_) => {
            set_last_error(PANIC_MESSAGE.to_string());
            return NVM_FFI_ERR_PANIC;
        }
    };

    guard(NVM_FFI_ERR_RUN, || {
        NVMl::with_memory_size(DEFAULT_MEMORY_SIZE)
            .run(BytecodeSource::Instructions(instructions))
            .map_err(|e| format_nvm_error(&e))
    })
}

/// Writes the text of the last error of the current thread into `buf`.
///
/// The required size (including the NUL) is written into `written`. The error is not cleared:
/// a repeated call returns the same text until a new error occurs.
#[unsafe(no_mangle)]
pub extern "C" fn nvm_last_error(buf: *mut c_char, cap: usize, written: *mut usize) -> i32 {
    guard(NVM_FFI_ERR_CONTRACT, || {
        let error = LAST_ERROR.with(|e| e.borrow().clone()).unwrap_or_default();
        copy_str_to_buffer(&error, buf.cast(), cap, written)
    })
}
