/**
  * @file nvm.h
  * @brief C ABI for NVM. Works in both C and C++ (extern "C").
  *
  * Conventions:
  *   - 0 (NVM_FFI_OK) — success, any other value — an error code;
  *   - the text of the last thread error is read via nvm_last_error();
  *   - results are written into a consumer-provided buffer (the "two calls"
  *     pattern: first get the size via *_size/written, then write);
  *   - on the "buffer too small" error, the required size is still
  *     written into written.
 */
#ifndef NVM_NVM_H
#define NVM_NVM_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define NVM_FFI_OK 0
#define NVM_FFI_ERR_COMPILE 1
#define NVM_FFI_ERR_RUN 2
#define NVM_FFI_ERR_CONTRACT 3
#define NVM_FFI_ERR_PANIC 4

/// @brief The NVM version as a C string (does not need to be freed).
const char* nvm_version(void);

/// @brief Compiles NVM Assembly and saves the .nb bytes in a thread-local buffer.
int nvm_compile(const char* source);

/// @brief Writes the size of the compilation result into size.
int nvm_compile_size(const char* source, size_t* size);

/// @brief Compiles and writes the `.nb` bytes into buf (capacity cap).
int nvm_compile_write(const char* source, uint8_t* buf, size_t cap, size_t* written);

/// @brief Executes the .nb bytes with the default memory.
int nvm_run_bytecode(const uint8_t* bytes, size_t len);

/// @brief Executes the .nb bytes with the specified memory size.
int nvm_run_bytecode_mem(const uint8_t* bytes, size_t len, size_t memory_size);

/// @brief Compiles the source and immediately executes it.
int nvm_run_source(const char* source);

/// @brief The text of the last error of the current thread (including the NUL).
int nvm_last_error(char* buf, size_t cap, size_t* written);

#ifdef __cplusplus
}
#endif

#endif /* NVM_NVM_H */
