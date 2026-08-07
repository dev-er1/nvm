// nvm-ffi/src/lib.rs
//
//! C ABI обёртка над `libnvm`.
//!
//! Даёт возможность использовать NVM из C, C++ и других языков
//! (Python через `ctypes`, Node через FFI, Go через `cgo` и т.д.).
//!
//! ## Конвенции
//!
//! - Все функции возвращают `0` (обёртка `NVM_FFI_OK`) при успехе и
//!   ненулевой код ошибки при неудаче (см. константы `NVM_FFI_ERR_*`).
//! - Текст последней ошибки потока получается через
//!   [`nvm_last_error`]. Ошибка живёт до следующего вызова FFI.
//! - Никаких указателей, выделенных Rust-ом, наружу не отдаём:
//!   результаты пишутся в буфер, предоставленный потребителем
//!   (паттерн "два вызова": сначала узнать размер, потом записать).
//! - Буферы передаются как `(указатель, ёмкость)`. При ошибке
//!   "буфер мал" в `written` всё равно записывается требуемый размер.
//! - Паника через FFI — UB, поэтому тела функций обёрнуты в
//!   [`catch_unwind`].
use std::{
    cell::RefCell,
    ffi::{CStr, CString, c_char},
    panic::{AssertUnwindSafe, catch_unwind},
    ptr,
    sync::LazyLock,
};

use libnvm::{BytecodeSource, DEFAULT_MEMORY_SIZE, NVM_VERSION, NVMAssembler, NVMError, NVMl};

/// Успех.
pub const NVM_FFI_OK: i32 = 0;

/// Ошибка компиляции NVM Assembly.
pub const NVM_FFI_ERR_COMPILE: i32 = 1;

/// Ошибка исполнения байт-кода.
pub const NVM_FFI_ERR_RUN: i32 = 2;

/// Нарушение контракта FFI (NULL-указатель, маленький буфер и т.д.).
pub const NVM_FFI_ERR_CONTRACT: i32 = 3;

/// Паника внутри FFI.
pub const NVM_FFI_ERR_PANIC: i32 = 4;

const PANIC_MESSAGE: &str = "panic in nvm-ffi";

thread_local! {
    /// Последний результат компиляции ([`nvm_compile`]).
    static LAST_COMPILED: RefCell<Option<Vec<u8>>> = const { RefCell::new(None) };

    /// Текст последней ошибки текущего потока.
    static LAST_ERROR: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// `NVM_VERSION` в виде C-строки (с завершающим NUL).
static VERSION_C: LazyLock<CString> =
    LazyLock::new(|| CString::new(NVM_VERSION).expect("NVM_VERSION contains nul"));

/// Выполняет FFI-функцию, превращая панику в код ошибки.
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

/// Читает NUL-терминированную C-строку как текст исходника.
fn read_source(source: *const c_char) -> Result<String, String> {
    if source.is_null() {
        return Err("source is null".to_string());
    }

    // NUL внутри исходника обрезает строку — наша семантика NUL-строк.
    Ok(String::from_utf8_lossy(unsafe { CStr::from_ptr(source) }.to_bytes()).into_owned())
}

/// Компилирует исходник и возвращает байты `.nb`.
fn compile_to_bytes(source: *const c_char) -> Result<Vec<u8>, String> {
    let source = read_source(source)?;
    NVMAssembler::assemble_to_bytecode(&source).map_err(|e| e.format())
}

/// Копирует бинарные данные в буфер потребителя.
///
/// В `written` записывается требуемый размер. Если буфер мал —
/// ошибка, но `written` всё равно сообщает требуемый размер.
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

/// Копирует строку (включая завершающий NUL) в буфер потребителя.
///
/// В `written` записывается требуемый размер (включая NUL).
fn copy_str_to_buffer(
    s: &str,
    buf: *mut u8,
    cap: usize,
    written: *mut usize,
) -> Result<(), String> {
    if written.is_null() {
        return Err("written is null".to_string());
    }

    // NUL-терминатор обязателен, поэтому на единицу больше длины строки.
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

/// Собирает текст ошибки исполнения.
fn format_nvm_error(e: &NVMError) -> String {
    match &e.instruction {
        Some(instruction) => format!("{}. --> {instruction}", e.kind),
        None => e.kind.to_string(),
    }
}

/// Исполняет байты `.nb` с заданным размером памяти.
fn run_bytecode(bytes: &[u8], memory_size: usize) -> Result<(), String> {
    NVMl::with_memory_size(memory_size)
        .run(BytecodeSource::Bytes(bytes.to_vec()))
        .map_err(|e| format_nvm_error(&e))
}

/// Версия NVM в виде C-строки (не требует освобождения).
///
/// ## Пример
/// ```c
/// printf("NVM %s\n", nvm_version());
/// ```
#[unsafe(no_mangle)]
pub extern "C" fn nvm_version() -> *const c_char {
    VERSION_C.as_ptr()
}

/// Компилирует исходник NVM Assembly и сохраняет байты `.nb`
/// в thread-local буфере (заменяя предыдущий результат).
///
/// Размер результата — [`nvm_compile_size`], чтение — [`nvm_compile_write`].
#[unsafe(no_mangle)]
pub extern "C" fn nvm_compile(source: *const c_char) -> i32 {
    guard(NVM_FFI_ERR_COMPILE, || {
        let bytes = compile_to_bytes(source)?;
        LAST_COMPILED.with(|m| *m.borrow_mut() = Some(bytes));
        Ok(())
    })
}

/// Записывает в `size` размер результата компиляции (без повторного
/// выделения). Сам результат не сохраняется.
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

/// Компилирует исходник и записывает байты `.nb` в `buf`.
///
/// В `written` — требуемый размер. Если буфер мал — `NVM_FFI_ERR_CONTRACT`.
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

/// Исполняет байты `.nb` с памятью по умолчанию.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn nvm_run_bytecode(bytes: *const u8, len: usize) -> i32 {
    guard(NVM_FFI_ERR_RUN, || {
        if len > 0 && bytes.is_null() {
            return Err("bytes is NULL".to_string());
        }
        // Само `run_bytecode` при len == 0 вернёт ошибку загрузчика.
        run_bytecode(
            unsafe { std::slice::from_raw_parts(bytes, len) },
            DEFAULT_MEMORY_SIZE,
        )
    })
}

/// Исполняет байты `.nb` с указанным размером памяти.
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

/// Компилирует исходник и сразу исполняет его.
///
/// Ошибки компиляции возвращают `NVM_FFI_ERR_COMPILE`,
/// ошибки исполнения — `NVM_FFI_ERR_RUN`.
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

/// Записывает текст последней ошибки текущего потока в `buf`.
///
/// В `written` — требуемый размер (включая NUL). Ошибка не очищается:
/// повторный вызов вернёт тот же текст, пока не произойдёт новая ошибка.
#[unsafe(no_mangle)]
pub extern "C" fn nvm_last_error(buf: *mut c_char, cap: usize, written: *mut usize) -> i32 {
    guard(NVM_FFI_ERR_CONTRACT, || {
        let error = LAST_ERROR.with(|e| e.borrow().clone()).unwrap_or_default();
        copy_str_to_buffer(&error, buf.cast(), cap, written)
    })
}
