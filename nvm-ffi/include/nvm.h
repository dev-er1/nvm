/**
  * @file nvm.h
  * @brief C ABI для NVM. Работает и в C, и в C++ (extern "C").
  *
  * Конвенции:
  *   - 0 (NVM_FFI_OK) — успех, любое другое значение — код ошибки;
  *   - текст последней ошибки потока — через nvm_last_error();
  *   - результаты пишутся в буфер потребителя (паттерн "два вызова":
  *     сначала узнать размер через *_size/written, потом записать);
  *   - при ошибке "буфер мал" в written всё равно записывается
  *     требуемый размер.
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

/* Версия NVM в виде C-строки (не требует освобождения). */
const char* nvm_version(void);

/* Компилирует NVM Assembly и сохраняет .nb-байты в thread-local буфере. */
int nvm_compile(const char* source);

/* Записывает в size размер результата компиляции. */
int nvm_compile_size(const char* source, size_t* size);

/* Компилирует и записывает .nb-байты в buf (ёмкость cap). */
int nvm_compile_write(const char* source, uint8_t* buf, size_t cap, size_t* written);

/* Исполняет .nb-байты с памятью по умолчанию. */
int nvm_run_bytecode(const uint8_t* bytes, size_t len);

/* Исполняет .nb-байты с указанным размером памяти. */
int nvm_run_bytecode_mem(const uint8_t* bytes, size_t len, size_t memory_size);

/* Компилирует исходник и сразу исполняет его. */
int nvm_run_source(const char* source);

/* Текст последней ошибки текущего потока (включая NUL). */
int nvm_last_error(char* buf, size_t cap, size_t* written);

#ifdef __cplusplus
}
#endif

#endif /* NVM_NVM_H */
