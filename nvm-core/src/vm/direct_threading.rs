// nvm-core/src/vm/direct_threading.rs
//
//! # Direct Threading исполнитель
//!
//! В этом модуле реализован исполнитель инструкций на основе
//! *direct threading* (прямой сквозной диспетчеризации).
//!
//! ## Идея
//!
//! Вместо того, чтобы разбирать инструкцию прямо в горячем цикле
//! (`Option`-ы операндов, их виды, количество), программа **один раз**
//! кодируется в плоский массив [`u64`]:
//!
//! 4 слота по 8 байт на каждую инструкцию.
//!
//! Слот `0` — заголовок: **адрес хендлера** этой инструкции, выбранного
//! по опкоду и видам операндов. "Виды" — по одному биту на операнд
//! ("операнд — регистр"). Хендлеры специализированы по сигнатуре:
//! по одному на каждую валидную комбинацию видов операндов
//! (например, `IADD` — четыре: `(imm, imm)`, `(reg, imm)`,
//! `(imm, reg)`, `(reg, reg)`). В обработчике не остаётся веток
//! по виду операнда — она устранена уже при кодировании.
//!
//! В отличие от jump table исполнителя, таблица переходов используется
//! **только на этапе кодирования**: в поток программы сразу кладутся
//! адреса хендлеров, поэтому в горячем цикле нет ни индексации таблицы,
//! ни вычисления индекса диспетчеризации, ни проверки границ таблицы.
//!
//! Каждый операнд "разворачивается" в число:
//! - регистр — в номер регистра (бит "регистр" в заголовке);
//! - immediate — как есть (бит сброшен);
//! - отсутствующий операнд — в `0` (бит сброшен).
//!
//! Количество и обязательные типы операндов (приёмники обязаны быть
//! регистрами) проверяются **один раз** при кодировании.
//!
//! ## Горячий цикл
//!
//! В цикле на инструкцию приходится: чтение адреса хендлера из
//! заголовка без проверки границ (инвариант `ip < len`), косвенный
//! вызов обработчика и одна проверка следующего `ip` на выход за
//! конец программы. Операнды читаются обработчиком по сырому указателю
//! без копий.
//!
//! ## Отличия от [`crate::vm::default`]
//!
//! Поведение совпадает со стандартным исполнителем, за одним исключением:
//! проверки количества и типов операндов выполняются при **кодировании**
//! программы (до начала исполнения), а не во время исполнения.
use crate::{
    isa::{
        instruction::Instruction,
        opcode::OperationCode,
        operand::{Operand, OperandKind},
        register::Register,
    },
    vm::{
        NVM,
        err::{VMError, VMErrorKind},
    },
};

/// Сколько слотов занимает одна инструкция в закодированной программе.
const SLOTS: usize = 4;

/// Количество опкодов NVM.
const OPCODE_COUNT: usize = OperationCode::RET as usize + 1;

/// Количество записей в таблице переходов: по 8 сигнатур на опкод.
const TABLE_LEN: usize = OPCODE_COUNT * 8;

/// Значение `EXIT` для следующего `ip`: завершает исполнение, так как
/// `>= instruction_count`.
const EXIT_MARKER: usize = usize::MAX;

/// Результат обработчика: индекс следующей инструкции.
type HandlerResult = Result<usize, VMError>;

/// Функция-обработчик одной инструкции.
///
/// Получает ВМ, указатель на слоты операндов текущей инструкции
/// (слот `0` — операнд 1, слот `1` — операнд 2, слот `2` — операнд 3)
/// и индекс текущей инструкции. Возвращает индекс следующей инструкции.
///
/// # Safety
///
/// Указатель указывает на операнды инструкции с индексом `ip`
/// в закодированной программе (гарантируется инвариантом
/// `0 <= ip < instruction_count` в [`NVM::direct_threading_execute`]);
/// обработчик читает только слоты `0..SLOTS - 1` относительно него,
/// т.е. строго в пределах инструкции.
type Handler = unsafe fn(&mut NVM, *const u64, usize) -> HandlerResult;

/// Читает слот операнда по указателю.
#[inline(always)]
fn slot(p: *const u64, n: usize) -> u64 {
    // SAFETY: см. `Handler` — слоты находятся в пределах инструкции.
    unsafe { *p.add(n) }
}

/// Читает значение регистрового операнда (в слоте — номер регистра).
#[inline(always)]
fn read_reg(vm: &NVM, p: *const u64, n: usize) -> u64 {
    vm.registers[Register(slot(p, n) as u8)]
}

/// Читает значение immediate-операнда (в слоте — значение).
#[inline(always)]
fn read_imm(p: *const u64, n: usize) -> u64 {
    slot(p, n)
}

/// Читает значение операнда:
/// - `reg` — содержимое регистра;
/// - `imm` — immediate из слота.
macro_rules! read_operand {
    ($vm:expr, $p:expr, $n:expr, reg) => {
        read_reg($vm, $p, $n)
    };
    ($vm:expr, $p:expr, $n:expr, imm) => {
        read_imm($p, $n)
    };
}

// ====== Обработчики ======
//
// Обработчики сгруппированы по "формам" инструкций. Для каждой формы
// генерируются специализированные варианты по сигнатуре операндов:
// `r` — регистр, `i` — immediate (порядок букв — порядок операндов).

fn nop(_vm: &mut NVM, _p: *const u64, ip: usize) -> HandlerResult {
    Ok(ip + 1)
}

fn exit(_vm: &mut NVM, _p: *const u64, _ip: usize) -> HandlerResult {
    Ok(EXIT_MARKER)
}

fn ret(vm: &mut NVM, _p: *const u64, _ip: usize) -> HandlerResult {
    let ip = vm
        .call_stack
        .pop()
        .ok_or_else(|| VMError::new(VMErrorKind::EmptyCallStack))?;
    Ok(ip)
}

/// Заглушка для невалидных сигнатур (никогда не вызывается:
/// кодирование не позволяет построить такую сигнатуру).
fn invalid_signature(_vm: &mut NVM, _p: *const u64, _ip: usize) -> HandlerResult {
    unreachable!("jump table: invalid operand signature reached")
}

/// Генератор варианта `MOVE` (2 операнда: dst — регистр, src — любой).
macro_rules! move_variant {
    ($name:ident, $k:tt) => {
        fn $name(vm: &mut NVM, p: *const u64, ip: usize) -> HandlerResult {
            vm.registers[Register(slot(p, 0) as u8)] = read_operand!(vm, p, 1, $k);
            Ok(ip + 1)
        }
    };
}

move_variant!(move_ri, imm);
move_variant!(move_rr, reg);

/// Генератор варианта `LOAD*` (2 операнда: dst — регистр, адрес — любой).
macro_rules! load_variant {
    ($name:ident, $method:ident, $k:tt) => {
        fn $name(vm: &mut NVM, p: *const u64, ip: usize) -> HandlerResult {
            let address = read_operand!(vm, p, 1, $k) as usize;
            let value = vm.memory.$method(address).ok_or_else(|| {
                VMError::new(VMErrorKind::InvalidAddress {
                    got: address,
                    memory_length: vm.memory.len(),
                })
            })?;
            vm.registers[Register(slot(p, 0) as u8)] = u64::from(value);
            Ok(ip + 1)
        }
    };
}

load_variant!(load8_ri, load_u8, imm);
load_variant!(load8_rr, load_u8, reg);
load_variant!(load16_ri, load_u16, imm);
load_variant!(load16_rr, load_u16, reg);
load_variant!(load32_ri, load_u32, imm);
load_variant!(load32_rr, load_u32, reg);
load_variant!(load64_ri, load_u64, imm);
load_variant!(load64_rr, load_u64, reg);

/// Генератор варианта `STORE*` (2 операнда: адрес и значение — любые).
macro_rules! store_variant {
    ($name:ident, $method:ident, $cast:ty, $ka:tt, $kv:tt) => {
        fn $name(vm: &mut NVM, p: *const u64, ip: usize) -> HandlerResult {
            let address = read_operand!(vm, p, 0, $ka) as usize;
            let value = read_operand!(vm, p, 1, $kv) as $cast;
            vm.memory.$method(address, value).ok_or_else(|| {
                VMError::new(VMErrorKind::InvalidAddress {
                    got: address,
                    memory_length: vm.memory.len(),
                })
            })?;
            Ok(ip + 1)
        }
    };
}

store_variant!(store8_ii, store_u8, u8, imm, imm);
store_variant!(store8_ri, store_u8, u8, reg, imm);
store_variant!(store8_ir, store_u8, u8, imm, reg);
store_variant!(store8_rr, store_u8, u8, reg, reg);
store_variant!(store16_ii, store_u16, u16, imm, imm);
store_variant!(store16_ri, store_u16, u16, reg, imm);
store_variant!(store16_ir, store_u16, u16, imm, reg);
store_variant!(store16_rr, store_u16, u16, reg, reg);
store_variant!(store32_ii, store_u32, u32, imm, imm);
store_variant!(store32_ri, store_u32, u32, reg, imm);
store_variant!(store32_ir, store_u32, u32, imm, reg);
store_variant!(store32_rr, store_u32, u32, reg, reg);
store_variant!(store64_ii, store_u64, u64, imm, imm);
store_variant!(store64_ri, store_u64, u64, reg, imm);
store_variant!(store64_ir, store_u64, u64, imm, reg);
store_variant!(store64_rr, store_u64, u64, reg, reg);

/// Генератор варианта бинарной операции (3 операнда: dst — регистр,
/// src1 и src2 — любые). `$op` — замыкание `|lhs, rhs| ...` над `u64`
/// (преобразование битов в `f64` и обратно — внутри замыкания).
macro_rules! binary_variant {
    ($name:ident, $k1:tt, $k2:tt, $op:expr) => {
        fn $name(vm: &mut NVM, p: *const u64, ip: usize) -> HandlerResult {
            let lhs = read_operand!(vm, p, 1, $k1);
            let rhs = read_operand!(vm, p, 2, $k2);
            vm.registers[Register(slot(p, 0) as u8)] = $op(lhs, rhs);
            Ok(ip + 1)
        }
    };
}

macro_rules! binops {
    ($op:expr, $($name:ident: $k1:tt $k2:tt),+ $(,)?) => {
        $(binary_variant!($name, $k1, $k2, $op);)+
    };
}

binops!(
    |l: u64, r: u64| l.wrapping_add(r),
    iadd_ii: imm imm,
    iadd_ri: reg imm,
    iadd_ir: imm reg,
    iadd_rr: reg reg,
);

binops!(
    |l: u64, r: u64| l.wrapping_sub(r),
    isub_ii: imm imm,
    isub_ri: reg imm,
    isub_ir: imm reg,
    isub_rr: reg reg,
);

binops!(
    |l: u64, r: u64| l.wrapping_mul(r),
    imul_ii: imm imm,
    imul_ri: reg imm,
    imul_ir: imm reg,
    imul_rr: reg reg,
);

/// Генератор варианта деления/остатка — как `binary_variant`, но
/// с проверкой делителя на ноль.
macro_rules! division_variant {
    ($name:ident, $k1:tt, $k2:tt, $op:expr) => {
        fn $name(vm: &mut NVM, p: *const u64, ip: usize) -> HandlerResult {
            let rhs = read_operand!(vm, p, 2, $k2);
            ensure_nonzero_divisor(rhs)?;
            let lhs = read_operand!(vm, p, 1, $k1);
            vm.registers[Register(slot(p, 0) as u8)] = $op(lhs, rhs);
            Ok(ip + 1)
        }
    };
}

macro_rules! divisions {
    ($op:expr, $($name:ident: $k1:tt $k2:tt),+ $(,)?) => {
        $(division_variant!($name, $k1, $k2, $op);)+
    };
}

divisions!(
    |l: u64, r: u64| ((l as i64).wrapping_div(r as i64)) as u64,
    sdiv_ii: imm imm,
    sdiv_ri: reg imm,
    sdiv_ir: imm reg,
    sdiv_rr: reg reg,
);

divisions!(
    |l: u64, r: u64| l / r,
    udiv_ii: imm imm,
    udiv_ri: reg imm,
    udiv_ir: imm reg,
    udiv_rr: reg reg,
);

divisions!(
    |l: u64, r: u64| ((l as i64).wrapping_rem(r as i64)) as u64,
    srem_ii: imm imm,
    srem_ri: reg imm,
    srem_ir: imm reg,
    srem_rr: reg reg,
);

divisions!(
    |l: u64, r: u64| l % r,
    urem_ii: imm imm,
    urem_ri: reg imm,
    urem_ir: imm reg,
    urem_rr: reg reg,
);

/// Генератор варианта унарной операции (2 операнда: dst — регистр,
/// src — любой).
macro_rules! unary_variant {
    ($name:ident, $k:tt, $op:expr) => {
        fn $name(vm: &mut NVM, p: *const u64, ip: usize) -> HandlerResult {
            let value = read_operand!(vm, p, 1, $k);
            vm.registers[Register(slot(p, 0) as u8)] = $op(value);
            Ok(ip + 1)
        }
    };
}

macro_rules! unaries {
    ($op:expr, $($name:ident: $k:tt),+ $(,)?) => {
        $(unary_variant!($name, $k, $op);)+
    };
}

unaries!(
    |v: u64| (v as i64).wrapping_neg() as u64,
    ineg_ri: imm,
    ineg_rr: reg,
);

binops!(
    |l: u64, r: u64| (f64::from_bits(l) + f64::from_bits(r)).to_bits(),
    fadd_ii: imm imm,
    fadd_ri: reg imm,
    fadd_ir: imm reg,
    fadd_rr: reg reg,
);

binops!(
    |l: u64, r: u64| (f64::from_bits(l) - f64::from_bits(r)).to_bits(),
    fsub_ii: imm imm,
    fsub_ri: reg imm,
    fsub_ir: imm reg,
    fsub_rr: reg reg,
);

binops!(
    |l: u64, r: u64| (f64::from_bits(l) * f64::from_bits(r)).to_bits(),
    fmul_ii: imm imm,
    fmul_ri: reg imm,
    fmul_ir: imm reg,
    fmul_rr: reg reg,
);

binops!(
    |l: u64, r: u64| (f64::from_bits(l) / f64::from_bits(r)).to_bits(),
    fdiv_ii: imm imm,
    fdiv_ri: reg imm,
    fdiv_ir: imm reg,
    fdiv_rr: reg reg,
);

binops!(
    |l: u64, r: u64| (f64::from_bits(l) % f64::from_bits(r)).to_bits(),
    frem_ii: imm imm,
    frem_ri: reg imm,
    frem_ir: imm reg,
    frem_rr: reg reg,
);

unaries!(
    |v: u64| (-f64::from_bits(v)).to_bits(),
    fneg_ri: imm,
    fneg_rr: reg,
);

binops!(
    |l: u64, r: u64| l & r,
    and_ii: imm imm,
    and_ri: reg imm,
    and_ir: imm reg,
    and_rr: reg reg,
);

binops!(
    |l: u64, r: u64| l | r,
    or_ii: imm imm,
    or_ri: reg imm,
    or_ir: imm reg,
    or_rr: reg reg,
);

binops!(
    |l: u64, r: u64| l ^ r,
    xor_ii: imm imm,
    xor_ri: reg imm,
    xor_ir: imm reg,
    xor_rr: reg reg,
);

binops!(
    |l: u64, r: u64| l.wrapping_shl(r as u32),
    shl_ii: imm imm,
    shl_ri: reg imm,
    shl_ir: imm reg,
    shl_rr: reg reg,
);

binops!(
    |l: u64, r: u64| l.wrapping_shr(r as u32),
    shr_ii: imm imm,
    shr_ri: reg imm,
    shr_ir: imm reg,
    shr_rr: reg reg,
);

binops!(
    |l: u64, r: u64| ((l as i64).wrapping_shr(r as u32)) as u64,
    sar_ii: imm imm,
    sar_ri: reg imm,
    sar_ir: imm reg,
    sar_rr: reg reg,
);

unaries!(
    |v: u64| !v,
    not_ri: imm,
    not_rr: reg,
);

binops!(
    |l: u64, r: u64| (l == r) as u64,
    ieq_ii: imm imm,
    ieq_ri: reg imm,
    ieq_ir: imm reg,
    ieq_rr: reg reg,
);

binops!(
    |l: u64, r: u64| (l != r) as u64,
    ine_ii: imm imm,
    ine_ri: reg imm,
    ine_ir: imm reg,
    ine_rr: reg reg,
);

binops!(
    |l: u64, r: u64| ((l as i64) < (r as i64)) as u64,
    slt_ii: imm imm,
    slt_ri: reg imm,
    slt_ir: imm reg,
    slt_rr: reg reg,
);

binops!(
    |l: u64, r: u64| ((l as i64) <= (r as i64)) as u64,
    sle_ii: imm imm,
    sle_ri: reg imm,
    sle_ir: imm reg,
    sle_rr: reg reg,
);

binops!(
    |l: u64, r: u64| ((l as i64) > (r as i64)) as u64,
    sgt_ii: imm imm,
    sgt_ri: reg imm,
    sgt_ir: imm reg,
    sgt_rr: reg reg,
);

binops!(
    |l: u64, r: u64| ((l as i64) >= (r as i64)) as u64,
    sge_ii: imm imm,
    sge_ri: reg imm,
    sge_ir: imm reg,
    sge_rr: reg reg,
);

binops!(
    |l: u64, r: u64| (l < r) as u64,
    ult_ii: imm imm,
    ult_ri: reg imm,
    ult_ir: imm reg,
    ult_rr: reg reg,
);

binops!(
    |l: u64, r: u64| (l <= r) as u64,
    ule_ii: imm imm,
    ule_ri: reg imm,
    ule_ir: imm reg,
    ule_rr: reg reg,
);

binops!(
    |l: u64, r: u64| (l > r) as u64,
    ugt_ii: imm imm,
    ugt_ri: reg imm,
    ugt_ir: imm reg,
    ugt_rr: reg reg,
);

binops!(
    |l: u64, r: u64| (l >= r) as u64,
    uge_ii: imm imm,
    uge_ri: reg imm,
    uge_ir: imm reg,
    uge_rr: reg reg,
);

binops!(
    |l: u64, r: u64| (f64::from_bits(l) == f64::from_bits(r)) as u64,
    feq_ii: imm imm,
    feq_ri: reg imm,
    feq_ir: imm reg,
    feq_rr: reg reg,
);

binops!(
    |l: u64, r: u64| (f64::from_bits(l) != f64::from_bits(r)) as u64,
    fne_ii: imm imm,
    fne_ri: reg imm,
    fne_ir: imm reg,
    fne_rr: reg reg,
);

binops!(
    |l: u64, r: u64| (f64::from_bits(l) < f64::from_bits(r)) as u64,
    flt_ii: imm imm,
    flt_ri: reg imm,
    flt_ir: imm reg,
    flt_rr: reg reg,
);

binops!(
    |l: u64, r: u64| (f64::from_bits(l) <= f64::from_bits(r)) as u64,
    fle_ii: imm imm,
    fle_ri: reg imm,
    fle_ir: imm reg,
    fle_rr: reg reg,
);

binops!(
    |l: u64, r: u64| (f64::from_bits(l) > f64::from_bits(r)) as u64,
    fgt_ii: imm imm,
    fgt_ri: reg imm,
    fgt_ir: imm reg,
    fgt_rr: reg reg,
);

binops!(
    |l: u64, r: u64| (f64::from_bits(l) >= f64::from_bits(r)) as u64,
    fge_ii: imm imm,
    fge_ri: reg imm,
    fge_ir: imm reg,
    fge_rr: reg reg,
);

/// Генератор варианта безусловного перехода (1 операнд — цель).
macro_rules! jmp_variant {
    ($name:ident, $k:tt) => {
        fn $name(_vm: &mut NVM, p: *const u64, _ip: usize) -> HandlerResult {
            let target = read_operand!(_vm, p, 0, $k) as usize;
            Ok(target)
        }
    };
}

jmp_variant!(jmp_i, imm);
jmp_variant!(jmp_r, reg);

/// Генератор варианта `CALL` (1 операнд — цель).
macro_rules! call_variant {
    ($name:ident, $k:tt) => {
        fn $name(vm: &mut NVM, p: *const u64, ip: usize) -> HandlerResult {
            let target = read_operand!(vm, p, 0, $k) as usize;
            // Адрес возврата — инструкция, следующая за `CALL`.
            vm.call_stack.push(ip + 1);
            Ok(target)
        }
    };
}

call_variant!(call_i, imm);
call_variant!(call_r, reg);

/// Генератор варианта условного перехода (2 операнда:
/// условие и цель — любые). `$taken` — замыкание `|cond| bool`.
macro_rules! cond_variant {
    ($name:ident, $k1:tt, $k2:tt, $taken:expr) => {
        fn $name(_vm: &mut NVM, p: *const u64, ip: usize) -> HandlerResult {
            let cond = read_operand!(_vm, p, 0, $k1);
            let target = read_operand!(_vm, p, 1, $k2) as usize;
            if $taken(cond) { Ok(target) } else { Ok(ip + 1) }
        }
    };
}

cond_variant!(jz_ii, imm, imm, |cond: u64| cond == 0);
cond_variant!(jz_ri, reg, imm, |cond: u64| cond == 0);
cond_variant!(jz_ir, imm, reg, |cond: u64| cond == 0);
cond_variant!(jz_rr, reg, reg, |cond: u64| cond == 0);
cond_variant!(jnz_ii, imm, imm, |cond: u64| cond != 0);
cond_variant!(jnz_ri, reg, imm, |cond: u64| cond != 0);
cond_variant!(jnz_ir, imm, reg, |cond: u64| cond != 0);
cond_variant!(jnz_rr, reg, reg, |cond: u64| cond != 0);

// ====== Таблица переходов ======

macro_rules! register {
    ($table:ident, $opcode:expr, $kinds:expr, $handler:ident) => {
        $table[$opcode as usize * 8 + $kinds] = $handler;
    };
}

macro_rules! register_binary {
    ($table:ident, $opcode:expr, $ii:ident, $ri:ident, $ir:ident, $rr:ident) => {
        register!($table, $opcode, 1, $ii);
        register!($table, $opcode, 3, $ri);
        register!($table, $opcode, 5, $ir);
        register!($table, $opcode, 7, $rr);
    };
}

macro_rules! register_store {
    ($table:ident, $opcode:expr, $ii:ident, $ri:ident, $ir:ident, $rr:ident) => {
        register!($table, $opcode, 0, $ii);
        register!($table, $opcode, 1, $ri);
        register!($table, $opcode, 2, $ir);
        register!($table, $opcode, 3, $rr);
    };
}

macro_rules! register_rr {
    ($table:ident, $opcode:expr, $ri:ident, $rr:ident) => {
        register!($table, $opcode, 1, $ri);
        register!($table, $opcode, 3, $rr);
    };
}

macro_rules! register_cond {
    ($table:ident, $opcode:expr, $ii:ident, $ri:ident, $ir:ident, $rr:ident) => {
        register!($table, $opcode, 0, $ii);
        register!($table, $opcode, 1, $ri);
        register!($table, $opcode, 2, $ir);
        register!($table, $opcode, 3, $rr);
    };
}

/// Строит таблицу переходов: `индекс = опкод * 8 + виды операндов`.
///
/// Виды операндов — по одному биту на операнд (`1` — регистр):
/// бит `0` — операнд 1, бит `1` — операнд 2, бит `2` — операнд 3.
///
/// Для каждого опкода регистрируются только валидные сигнатуры
/// (приёмники всегда регистры), остальные слоты — заглушка
/// [`invalid_signature`].
const fn build_jump_table() -> [Handler; TABLE_LEN] {
    let mut table = [invalid_signature as Handler; TABLE_LEN];

    register!(table, OperationCode::NOP, 0, nop);
    register!(table, OperationCode::EXIT, 0, exit);

    register_rr!(table, OperationCode::MOVE, move_ri, move_rr);

    register_rr!(table, OperationCode::LOAD8, load8_ri, load8_rr);
    register_rr!(table, OperationCode::LOAD16, load16_ri, load16_rr);
    register_rr!(table, OperationCode::LOAD32, load32_ri, load32_rr);
    register_rr!(table, OperationCode::LOAD64, load64_ri, load64_rr);

    register_store!(
        table,
        OperationCode::STORE8,
        store8_ii,
        store8_ri,
        store8_ir,
        store8_rr
    );
    register_store!(
        table,
        OperationCode::STORE16,
        store16_ii,
        store16_ri,
        store16_ir,
        store16_rr
    );
    register_store!(
        table,
        OperationCode::STORE32,
        store32_ii,
        store32_ri,
        store32_ir,
        store32_rr
    );
    register_store!(
        table,
        OperationCode::STORE64,
        store64_ii,
        store64_ri,
        store64_ir,
        store64_rr
    );

    register_binary!(
        table,
        OperationCode::IADD,
        iadd_ii,
        iadd_ri,
        iadd_ir,
        iadd_rr
    );
    register_binary!(
        table,
        OperationCode::ISUB,
        isub_ii,
        isub_ri,
        isub_ir,
        isub_rr
    );
    register_binary!(
        table,
        OperationCode::IMUL,
        imul_ii,
        imul_ri,
        imul_ir,
        imul_rr
    );

    register_binary!(
        table,
        OperationCode::SDIV,
        sdiv_ii,
        sdiv_ri,
        sdiv_ir,
        sdiv_rr
    );
    register_binary!(
        table,
        OperationCode::UDIV,
        udiv_ii,
        udiv_ri,
        udiv_ir,
        udiv_rr
    );
    register_binary!(
        table,
        OperationCode::SREM,
        srem_ii,
        srem_ri,
        srem_ir,
        srem_rr
    );
    register_binary!(
        table,
        OperationCode::UREM,
        urem_ii,
        urem_ri,
        urem_ir,
        urem_rr
    );

    register_rr!(table, OperationCode::INEG, ineg_ri, ineg_rr);

    register_binary!(
        table,
        OperationCode::FADD,
        fadd_ii,
        fadd_ri,
        fadd_ir,
        fadd_rr
    );
    register_binary!(
        table,
        OperationCode::FSUB,
        fsub_ii,
        fsub_ri,
        fsub_ir,
        fsub_rr
    );
    register_binary!(
        table,
        OperationCode::FMUL,
        fmul_ii,
        fmul_ri,
        fmul_ir,
        fmul_rr
    );
    register_binary!(
        table,
        OperationCode::FDIV,
        fdiv_ii,
        fdiv_ri,
        fdiv_ir,
        fdiv_rr
    );
    register_binary!(
        table,
        OperationCode::FREM,
        frem_ii,
        frem_ri,
        frem_ir,
        frem_rr
    );

    register_rr!(table, OperationCode::FNEG, fneg_ri, fneg_rr);

    register_binary!(table, OperationCode::AND, and_ii, and_ri, and_ir, and_rr);
    register_binary!(table, OperationCode::OR, or_ii, or_ri, or_ir, or_rr);
    register_binary!(table, OperationCode::XOR, xor_ii, xor_ri, xor_ir, xor_rr);

    register_rr!(table, OperationCode::NOT, not_ri, not_rr);

    register_binary!(table, OperationCode::SHL, shl_ii, shl_ri, shl_ir, shl_rr);
    register_binary!(table, OperationCode::SHR, shr_ii, shr_ri, shr_ir, shr_rr);
    register_binary!(table, OperationCode::SAR, sar_ii, sar_ri, sar_ir, sar_rr);

    register_binary!(table, OperationCode::IEQ, ieq_ii, ieq_ri, ieq_ir, ieq_rr);
    register_binary!(table, OperationCode::INE, ine_ii, ine_ri, ine_ir, ine_rr);
    register_binary!(table, OperationCode::SLT, slt_ii, slt_ri, slt_ir, slt_rr);
    register_binary!(table, OperationCode::SLE, sle_ii, sle_ri, sle_ir, sle_rr);
    register_binary!(table, OperationCode::SGT, sgt_ii, sgt_ri, sgt_ir, sgt_rr);
    register_binary!(table, OperationCode::SGE, sge_ii, sge_ri, sge_ir, sge_rr);
    register_binary!(table, OperationCode::ULT, ult_ii, ult_ri, ult_ir, ult_rr);
    register_binary!(table, OperationCode::ULE, ule_ii, ule_ri, ule_ir, ule_rr);
    register_binary!(table, OperationCode::UGT, ugt_ii, ugt_ri, ugt_ir, ugt_rr);
    register_binary!(table, OperationCode::UGE, uge_ii, uge_ri, uge_ir, uge_rr);

    register_binary!(table, OperationCode::FEQ, feq_ii, feq_ri, feq_ir, feq_rr);
    register_binary!(table, OperationCode::FNE, fne_ii, fne_ri, fne_ir, fne_rr);
    register_binary!(table, OperationCode::FLT, flt_ii, flt_ri, flt_ir, flt_rr);
    register_binary!(table, OperationCode::FLE, fle_ii, fle_ri, fle_ir, fle_rr);
    register_binary!(table, OperationCode::FGT, fgt_ii, fgt_ri, fgt_ir, fgt_rr);
    register_binary!(table, OperationCode::FGE, fge_ii, fge_ri, fge_ir, fge_rr);

    register!(table, OperationCode::JMP, 0, jmp_i);
    register!(table, OperationCode::JMP, 1, jmp_r);

    register_cond!(table, OperationCode::JZ, jz_ii, jz_ri, jz_ir, jz_rr);
    register_cond!(table, OperationCode::JNZ, jnz_ii, jnz_ri, jnz_ir, jnz_rr);

    register!(table, OperationCode::CALL, 0, call_i);
    register!(table, OperationCode::CALL, 1, call_r);

    register!(table, OperationCode::RET, 0, ret);

    table
}

/// Таблица переходов: `индекс = опкод * 8 + виды операндов`.
///
/// Используется **только на этапе кодирования** ([`encode`]), чтобы
/// положить в поток программы адрес нужного хендлера. В горячем цикле
/// обращения к таблице нет.
static JUMP_TABLE: [Handler; TABLE_LEN] = build_jump_table();

// ====== Исполнитель ======

impl NVM {
    /// Выполняет программу на основе direct threading.
    pub fn direct_threading_execute(&mut self) -> Result<(), VMError> {
        // Кодируем программу один раз.
        let code = encode(&self.program)?;
        let instruction_count = self.program.len();

        if instruction_count == 0 {
            return Ok(());
        }

        let mut ip = 0usize;

        // Инвариант цикла: `0 <= ip < instruction_count`, поэтому чтение
        // заголовка инструкции и слотов операндов (через обработчик)
        // не выходит за границы `code`.
        loop {
            // SAFETY: `ip < instruction_count`, см. инвариант выше.
            let base = unsafe { code.as_ptr().add(ip * SLOTS) };
            // SAFETY: заголовок — первый слот инструкции (в границах);
            // хранит адрес хендлера, см. [`encode`].
            let handler: Handler = unsafe { std::mem::transmute(*base) };
            // SAFETY: обработчик читает только слоты операндов
            // текущей инструкции, см. [`Handler`].
            let next = unsafe { handler(self, base.add(1), ip) }?;

            // `EXIT` возвращает [`EXIT_MARKER`]; переход за конец программы
            // тоже завершает исполнение (как в `default`-исполнителе).
            if next >= instruction_count {
                return Ok(());
            }
            ip = next;
        }
    }
}

// ====== Кодирование программы ======

/// Ожидаемое количество операндов и слоты, которые обязаны быть регистрами.
///
/// Слоты нумеруются с `1` (слот `0` — заголовок). Слоты, не входящие
/// в `register_slots`, могут быть как регистром, так и immediate
/// (как в [`crate::vm::default`]).
fn operand_pattern(opcode: OperationCode) -> (u8, &'static [usize]) {
    use OperationCode::*;

    match opcode {
        NOP | EXIT | RET => (0, &[]),
        JMP | CALL => (1, &[]),
        JZ | JNZ => (2, &[]),
        MOVE | LOAD8 | LOAD16 | LOAD32 | LOAD64 => (2, &[1]),
        STORE8 | STORE16 | STORE32 | STORE64 => (2, &[]),
        INEG | FNEG | NOT => (2, &[1]),
        IADD | ISUB | IMUL | SDIV | UDIV | SREM | UREM => (3, &[1]),
        FADD | FSUB | FMUL | FDIV | FREM => (3, &[1]),
        AND | OR | XOR | SHL | SHR | SAR => (3, &[1]),
        IEQ | INE | SLT | SLE | SGT | SGE | ULT | ULE | UGT | UGE => (3, &[1]),
        FEQ | FNE | FLT | FLE | FGT | FGE => (3, &[1]),
    }
}

/// Является ли операнд регистром.
fn is_register(operand: &Option<Operand>) -> bool {
    matches!(
        operand,
        Some(Operand {
            kind: OperandKind::Register(_),
            ..
        })
    )
}

/// Кодирует программу в плоский массив `u64`.
///
/// Каждая инструкция занимает [`SLOTS`] слотов:
/// `[заголовок, операнд1, операнд2, операнд3]`. В заголовке — адрес
/// хендлера этой инструкции (выбирается по опкоду и видам операндов,
/// см. [`build_jump_table`]).
///
/// При кодировании проверяются количество и обязательные типы операндов
/// (см. [`operand_pattern`]).
fn encode(program: &[Instruction]) -> Result<Vec<u64>, VMError> {
    let mut code = Vec::with_capacity(program.len() * SLOTS);

    for instr in program {
        let count = instr.operand_count();
        let (expected, register_slots) = operand_pattern(instr.opcode);

        if count != expected as usize {
            return Err(VMError::new(VMErrorKind::IncorrectNumberOfOperands {
                expected,
                got: count as u8,
            }));
        }

        for &slot in register_slots {
            let operand = [&instr.operand1, &instr.operand2, &instr.operand3][slot - 1];
            if !is_register(operand) {
                return Err(VMError::new(VMErrorKind::IncorrectTypeOfOperand {
                    expected: OperandKind::Register(Register(0)),
                    got: operand.map(|o| o.kind).unwrap_or(OperandKind::Immediate(0)),
                }));
            }
        }

        let kinds = (is_register(&instr.operand1) as u64)
            | (is_register(&instr.operand2) as u64) << 1
            | (is_register(&instr.operand3) as u64) << 2;

        // Диспетчеризация: в заголовок кладём сам адрес хендлера
        // сигнатуры. Таблица переходов участвует только на этапе
        // кодирования — в горячем цикле её нет.
        let handler = JUMP_TABLE[instr.opcode as usize * 8 + kinds as usize];
        // SAFETY: `Handler` — function pointer, на целевых платформах
        // NVM (64-битных) он представим как `u64`.
        code.push(handler as *const () as u64);
        code.push(flatten(instr.operand1));
        code.push(flatten(instr.operand2));
        code.push(flatten(instr.operand3));
    }

    Ok(code)
}

/// "Разворачивает" операнд в число.
fn flatten(operand: Option<Operand>) -> u64 {
    match operand {
        Some(Operand {
            kind: OperandKind::Register(r),
        }) => u64::from(r.0),
        Some(Operand {
            kind: OperandKind::Immediate(v),
        }) => v,
        None => 0,
    }
}

#[inline]
fn ensure_nonzero_divisor(rhs: u64) -> Result<(), VMError> {
    if rhs == 0 {
        Err(VMError::new(VMErrorKind::DivisionByZero))
    } else {
        Ok(())
    }
}
