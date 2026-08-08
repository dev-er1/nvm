// nvm-core/src/vm/executer.rs
//
//! # NVM bytecode executor
//!
//! This module implements the NVM instruction executor —
//! based on *Direct Threading* (direct threaded dispatch).
//!
//! ## The idea
//!
//! The program is encoded **once** into a flat array of [`u64`]:
//!
//! 4 slots of 8 bytes per instruction.
//!
//! Slot `0` is the header: the **handler address** of this instruction, chosen
//! by the opcode and the operand kinds. "Kinds" are one bit per operand
//! ("the operand is a register"). Handlers are specialized by signature:
//! one for each valid combination of operand kinds
//! (for example, `IADD` — four: `(imm, imm)`, `(reg, imm)`,
//! `(imm, reg)`, `(reg, reg)`). The handler has no branches
//! by operand kind — the branching is eliminated already at encoding time.
//!
//! The jump table is used **only at the encoding stage**:
//! the handler addresses are placed directly into the program stream, so
//! in the hot loop there is no table indexing, no dispatch index
//! computation, and no table bounds check.
//!
//! Each operand is "flattened" into a number:
//! - a register — into the register number (the "register" bit in the header);
//! - an immediate — as is (the bit cleared);
//! - a missing operand — into `0` (the bit cleared).
//!
//! The operand count and the mandatory operand types (destinations must be
//! registers) are checked **once** at encoding, before execution starts.
//!
//! ## Hot loop
//!
//! In the loop, per instruction: reading the handler address from
//! the header without a bounds check (the `ip < len` invariant), an indirect
//! handler call, and one check that the next `ip` does not go past the
//! end of the program. Operands are read by the handler via a raw pointer
//! without copies.
//!
//! The entry point is [`NVM::run`].
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

/// How many slots one instruction occupies in the encoded program.
const SLOTS: usize = 4;

/// The number of NVM opcodes.
const OPCODE_COUNT: usize = OperationCode::RET as usize + 1;

/// The number of entries in the jump table: 8 signatures per opcode.
const TABLE_LEN: usize = OPCODE_COUNT * 8;

/// The `EXIT` value for the next `ip`: ends execution, since
/// it is `>= instruction_count`.
const EXIT_MARKER: usize = usize::MAX;

/// The handler result: the index of the next instruction.
type HandlerResult = Result<usize, VMError>;

/// The handler function of a single instruction.
///
/// Receives the VM, a pointer to the operand slots of the current instruction
/// (slot `0` is operand 1, slot `1` is operand 2, slot `2` is operand 3),
/// and the index of the current instruction. Returns the index of the next instruction.
///
/// # Safety
///
/// The pointer points to the operands of the instruction with index `ip`
/// in the encoded program (guaranteed by the invariant
/// `0 <= ip < instruction_count` in [`NVM::run`]);
/// the handler reads only slots `0..SLOTS - 1` relative to it,
/// i.e. strictly within the instruction.
type Handler = unsafe fn(&mut NVM, *const u64, usize) -> HandlerResult;

/// Reads an operand slot via the pointer.
#[inline(always)]
fn slot(p: *const u64, n: usize) -> u64 {
    // SAFETY: see `Handler` — the slots are within the instruction.
    unsafe { *p.add(n) }
}

/// Reads the value of a register operand (the slot holds the register number).
#[inline(always)]
fn read_reg(vm: &NVM, p: *const u64, n: usize) -> u64 {
    vm.registers[Register(slot(p, n) as u8)]
}

/// Reads the value of an immediate operand (the slot holds the value).
#[inline(always)]
fn read_imm(p: *const u64, n: usize) -> u64 {
    slot(p, n)
}

/// Reads the value of an operand:
/// - `reg` — the register contents;
/// - `imm` — the immediate from the slot.
macro_rules! read_operand {
    ($vm:expr, $p:expr, $n:expr, reg) => {
        read_reg($vm, $p, $n)
    };
    ($vm:expr, $p:expr, $n:expr, imm) => {
        read_imm($p, $n)
    };
}

// ====== Handlers ======
//
// Handlers are grouped by instruction "shapes". For each shape,
// specialized variants are generated per operand signature:
// `r` — register, `i` — immediate (the letter order is the operand order).

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

/// Stub for invalid signatures (never called:
/// encoding cannot produce such a signature).
fn invalid_signature(_vm: &mut NVM, _p: *const u64, _ip: usize) -> HandlerResult {
    unreachable!("jump table: invalid operand signature reached")
}

/// Generator of a `MOVE` variant (2 operands: dst — register, src — any).
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

/// Generator of a `LOAD*` variant (2 operands: dst — register, address — any).
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

/// Generator of a `STORE*` variant (2 operands: address and value — any).
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

/// Generator of a binary operation variant (3 operands: dst — register,
/// `src1` and `src2` — any). `$op` — a closure `|lhs, rhs| ...` over `u64`
/// (bit conversion to `f64` and back — inside the closure).
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

/// Generator of a division/remainder variant — like `binary_variant`, but
/// with a divisor zero check.
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

/// Generator of a unary operation variant (2 operands: dst — register,
/// src — any).
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

/// Generator of an unconditional jump variant (1 operand — the target).
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

/// Generator of a `CALL` variant (1 operand — the target).
macro_rules! call_variant {
    ($name:ident, $k:tt) => {
        fn $name(vm: &mut NVM, p: *const u64, ip: usize) -> HandlerResult {
            let target = read_operand!(vm, p, 0, $k) as usize;
            // The return address is the instruction following `CALL`.
            vm.call_stack.push(ip + 1);
            Ok(target)
        }
    };
}

call_variant!(call_i, imm);
call_variant!(call_r, reg);

/// Generator of a conditional jump variant (2 operands:
/// the condition and the target — any). `$taken` — a closure `|cond| bool`.
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

// ====== Jump table ======

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

/// Builds the jump table: `index = opcode * 8 + operand kinds`.
///
/// Operand kinds are one bit per operand (`1` — register):
/// bit `0` — operand 1, bit `1` — operand 2, bit `2` — operand 3.
///
/// For each opcode, only valid signatures are registered
/// (destinations are always registers); the remaining slots are the
/// [`invalid_signature`] stub.
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

/// The jump table: `index = opcode * 8 + operand kinds`.
///
/// Used **only at the encoding stage** ([`encode`]) to put the address
/// of the required handler into the program stream. The hot loop does not
/// touch the table.
static JUMP_TABLE: [Handler; TABLE_LEN] = build_jump_table();

// ====== Executor ======

impl NVM {
    /// Runs the VM program (Direct Threading).
    pub fn run(&mut self) -> Result<(), VMError> {
        // Encode the program once.
        let code = encode(&self.program)?;
        let instruction_count = self.program.len();

        if instruction_count == 0 {
            return Ok(());
        }

        let mut ip = 0usize;

        // Loop invariant: `0 <= ip < instruction_count`, so reading
        // the instruction header and the operand slots (through the handler)
        // never goes out of bounds of `code`.
        loop {
            // SAFETY: `ip < instruction_count`, see the invariant above.
            let base = unsafe { code.as_ptr().add(ip * SLOTS) };
            // SAFETY: the header is the instruction's first slot (within bounds);
            // it stores the handler address, see [`encode`].
            let handler: Handler = unsafe { std::mem::transmute(*base) };
            // SAFETY: the handler reads only the operand slots
            // of the current instruction, see [`Handler`].
            let next = unsafe { handler(self, base.add(1), ip) }?;

            // `EXIT` returns [`EXIT_MARKER`]; jumping past the end of the program
            // also ends execution (as in the `default` executor).
            if next >= instruction_count {
                return Ok(());
            }
            ip = next;
        }
    }
}

// ====== Program encoding ======

/// The expected operand count and the slots that must be registers.
///
/// Slots are numbered from `1` (slot `0` is the header). Slots not in
/// `register_slots` can be either a register or an immediate.
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

/// Whether the operand is a register.
fn is_register(operand: &Option<Operand>) -> bool {
    matches!(
        operand,
        Some(Operand {
            kind: OperandKind::Register(_),
            ..
        })
    )
}

/// Encodes a program into a flat array of `u64`.
///
/// Each instruction occupies [`SLOTS`] slots:
/// `[header, operand1, operand2, operand3]`. The header holds the address
/// of the handler for this instruction (chosen by the opcode and operand kinds,
/// see [`build_jump_table`]).
///
/// On encoding, the operand count and the required operand types
/// are checked (see [`operand_pattern`]).
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

        // Dispatch: the handler address itself is placed into the header.
        // The jump table is used only at the encoding stage —
        // in the hot loop it is absent.
        let handler = JUMP_TABLE[instr.opcode as usize * 8 + kinds as usize];
        // SAFETY: `Handler` is a function pointer; on the target platforms
        // of NVM (64-bit) it is representable as `u64`.
        code.push(handler as *const () as u64);
        code.push(flatten(instr.operand1));
        code.push(flatten(instr.operand2));
        code.push(flatten(instr.operand3));
    }

    Ok(code)
}

/// "Flattens" an operand into a number.
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
