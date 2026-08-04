// nvm-core/src/vm/default.rs
//
//! # Стандартный исполнитель программы
//!
//! В этом модуле реализован стандартный исполнитель инструкций
//! на основе `match`.
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

type VMResult = Result<(), VMError>;

impl NVM {
    pub fn match_execute(&mut self) -> VMResult {
        let mut ip: usize = 0;

        use OperationCode::*;

        while ip != self.program.len() {
            let current_instruction = self.program[ip];
            let mut jumped = false;

            match current_instruction.opcode {
                NOP => { /* Ничего не делаем так как после этого `match`-а уже повышается `ip` */
                }
                EXIT => return Ok(()),

                MOVE => self.mov(current_instruction)?,

                LOAD8 | LOAD16 | LOAD32 | LOAD64 => self.load_(current_instruction)?,
                STORE8 | STORE16 | STORE32 | STORE64 => self.store_(current_instruction)?,

                IADD | ISUB | IMUL | SDIV | UDIV | SREM | UREM => {
                    self.iarithm(current_instruction)?
                }
                INEG => self.ineg(current_instruction)?,

                FADD | FSUB | FMUL | FDIV | FREM => self.farithm(current_instruction)?,
                FNEG => self.fneg(current_instruction)?,

                AND | OR | XOR | NOT | SHL | SHR | SAR => self.bitwise(current_instruction)?,

                IEQ | INE | SLT | SLE | SGT | SGE | ULT | ULE | UGT | UGE => {
                    self.icmp(current_instruction)?
                }
                FEQ | FNE | FLT | FLE | FGT | FGE => self.fcmp(current_instruction)?,

                JMP | JZ | JNZ | CALL | RET => {
                    jumped = self.jump(current_instruction, &mut ip)?;
                }
            }

            if !jumped {
                ip += 1;
            }
        }
        Ok(())
    }

    // ====== Управление памятью ======
    // * `MOVE`,
    // * `LOAD*`,
    // * `STORE*`.

    fn mov(&mut self, instr: Instruction) -> VMResult {
        let (dst, src) = instr.expect2()?;

        let dst = self.get_register(dst)?;
        let src = self.get_value(src)?;

        self.registers[dst] = src;

        Ok(())
    }

    fn load_(&mut self, instr: Instruction) -> VMResult {
        let (dst, src) = instr.expect2()?;

        let dst = self.get_register(dst)?;
        let address = self.get_value(src)? as usize;

        self.registers[dst] = (match instr.opcode {
            OperationCode::LOAD8 => self.memory.load_u8(address).map(u64::from),
            OperationCode::LOAD16 => self.memory.load_u16(address).map(u64::from),
            OperationCode::LOAD32 => self.memory.load_u32(address).map(u64::from),
            OperationCode::LOAD64 => self.memory.load_u64(address),
            _ => unreachable!(),
        })
        .ok_or_else(|| {
            VMError::new(VMErrorKind::InvalidAddress {
                got: address,
                memory_length: self.memory.len(),
            })
        })?;

        Ok(())
    }

    fn store_(&mut self, instr: Instruction) -> VMResult {
        let (dst, src) = instr.expect2()?;

        let address = self.get_value(dst)? as usize;
        let value = self.get_value(src)?;

        (match instr.opcode {
            OperationCode::STORE8 => self.memory.store_u8(address, value as u8),
            OperationCode::STORE16 => self.memory.store_u16(address, value as u16),
            OperationCode::STORE32 => self.memory.store_u32(address, value as u32),
            OperationCode::STORE64 => self.memory.store_u64(address, value),
            _ => unreachable!(),
        })
        .ok_or_else(|| {
            VMError::new(VMErrorKind::InvalidAddress {
                got: address,
                memory_length: self.memory.len(),
            })
        })?;

        Ok(())
    }

    // ====== Арифметика и операции ======
    //
    // * Целочисленная арифм. и операции:
    //     * `IADD`,
    //     * `ISUB`,
    //     * `IMUL`,
    //     * `INEG`,
    //
    //     * Знаковая:
    //         * `SDIV`,
    //         * `SREM`,
    //
    //     * Беззнаковая:
    //         * `UDIV`,
    //         * `UREM`.
    //
    // * Дробная арифм. и операции:
    //     * `FADD`,
    //     * `FSUB`,
    //     * `FMUL`,
    //     * `FNEG`,
    //     * `FREM`,
    //     * `FDIV`

    fn iarithm(&mut self, instr: Instruction) -> VMResult {
        let (dst, src1, src2) = instr.expect3()?;

        let dst = self.get_register(dst)?;
        let lhs = self.get_value(src1)?;
        let rhs = self.get_value(src2)?;

        if matches!(
            instr.opcode,
            OperationCode::SDIV | OperationCode::UDIV | OperationCode::SREM | OperationCode::UREM
        ) {
            ensure_nonzero_divisor(rhs)?
        }

        self.registers[dst] = match instr.opcode {
            OperationCode::IADD => lhs.wrapping_add(rhs),
            OperationCode::ISUB => lhs.wrapping_sub(rhs),
            OperationCode::IMUL => lhs.wrapping_mul(rhs),

            // Знаковое деление.
            OperationCode::SDIV => ((lhs as i64).wrapping_div(rhs as i64)) as u64,

            // Беззнаковое деление.
            OperationCode::UDIV => lhs / rhs,

            // Знаковый остаток.
            OperationCode::SREM => ((lhs as i64).wrapping_rem(rhs as i64)) as u64,

            // Беззнаковый остаток.
            OperationCode::UREM => lhs % rhs,

            _ => unreachable!(),
        };

        Ok(())
    }

    fn ineg(&mut self, instr: Instruction) -> VMResult {
        let (dst, src) = instr.expect2()?;

        let dst = self.get_register(dst)?;
        let value = self.get_value(src)?;

        self.registers[dst] = (value as i64).wrapping_neg() as u64;

        Ok(())
    }

    fn farithm(&mut self, instr: Instruction) -> VMResult {
        let (dst, src1, src2) = instr.expect3()?;

        let dst = self.get_register(dst)?;

        // Так как в регистрах хранятся `u64` — загружаем и выгружаем
        // значения через `from_bits()`/`to_bits()`.

        let lhs = f64::from_bits(self.get_value(src1)?);
        let rhs = f64::from_bits(self.get_value(src2)?);

        self.registers[dst] = match instr.opcode {
            OperationCode::FADD => (lhs + rhs).to_bits(),
            OperationCode::FSUB => (lhs - rhs).to_bits(),
            OperationCode::FMUL => (lhs * rhs).to_bits(),
            OperationCode::FDIV => (lhs / rhs).to_bits(),
            OperationCode::FREM => (lhs % rhs).to_bits(),
            _ => unreachable!(),
        };

        Ok(())
    }

    fn fneg(&mut self, instr: Instruction) -> VMResult {
        let (dst, src) = instr.expect2()?;

        let dst = self.get_register(dst)?;
        let value = f64::from_bits(self.get_value(src)?);

        self.registers[dst] = (-value).to_bits();

        Ok(())
    }

    // ====== Побитовые операции ======
    //
    // * `AND`,
    // * `OR`,
    // * `XOR`,
    // * `NOT`,
    // * `SHL`,
    // * `SHR`,
    // * `SAR`.

    fn bitwise(&mut self, instr: Instruction) -> VMResult {
        use OperationCode::*;

        if matches!(instr.opcode, NOT) {
            let (dst, src) = instr.expect2()?;

            let dst = self.get_register(dst)?;
            let value = self.get_value(src)?;

            self.registers[dst] = !value;

            return Ok(());
        }

        let (dst, src1, src2) = instr.expect3()?;

        let dst = self.get_register(dst)?;
        let lhs = self.get_value(src1)?;
        let rhs = self.get_value(src2)?;

        self.registers[dst] = match instr.opcode {
            AND => lhs & rhs,
            OR => lhs | rhs,
            XOR => lhs ^ rhs,
            SHL => lhs.wrapping_shl(rhs as u32),
            SHR => lhs.wrapping_shr(rhs as u32),
            SAR => ((lhs as i64).wrapping_shr(rhs as u32)) as u64,
            _ => unreachable!(),
        };

        Ok(())
    }

    // ====== Операции сравнения ======
    //
    // * Целочисленные:
    //     * `IEQ`,
    //     * `INE`,
    //     * `SLT`,
    //     * `SLE`,
    //     * `SGT`,
    //     * `SGE`,
    //     * `ULT`,
    //     * `ULE`,
    //     * `UGT`,
    //     * `UGE`.
    //
    // * Дробные:
    //     * `FEQ`,
    //     * `FNE`,
    //     * `FLT`,
    //     * `FLE`,
    //     * `FGT`,
    //     * `FGE`.

    fn icmp(&mut self, instr: Instruction) -> VMResult {
        let (dst, src1, src2) = instr.expect3()?;

        let dst = self.get_register(dst)?;
        let lhs = self.get_value(src1)?;
        let rhs = self.get_value(src2)?;

        self.registers[dst] = match instr.opcode {
            OperationCode::IEQ => (lhs == rhs) as u64,
            OperationCode::INE => (lhs != rhs) as u64,
            OperationCode::SLT => ((lhs as i64) < (rhs as i64)) as u64,
            OperationCode::SLE => ((lhs as i64) <= (rhs as i64)) as u64,
            OperationCode::SGT => ((lhs as i64) > (rhs as i64)) as u64,
            OperationCode::SGE => ((lhs as i64) >= (rhs as i64)) as u64,
            OperationCode::ULT => (lhs < rhs) as u64,
            OperationCode::ULE => (lhs <= rhs) as u64,
            OperationCode::UGT => (lhs > rhs) as u64,
            OperationCode::UGE => (lhs >= rhs) as u64,
            _ => unreachable!(),
        };

        Ok(())
    }

    fn fcmp(&mut self, instr: Instruction) -> VMResult {
        let (dst, src1, src2) = instr.expect3()?;

        let dst = self.get_register(dst)?;
        let lhs = f64::from_bits(self.get_value(src1)?);
        let rhs = f64::from_bits(self.get_value(src2)?);

        self.registers[dst] = match instr.opcode {
            OperationCode::FEQ => (lhs == rhs) as u64,
            OperationCode::FNE => (lhs != rhs) as u64,
            OperationCode::FLT => (lhs < rhs) as u64,
            OperationCode::FLE => (lhs <= rhs) as u64,
            OperationCode::FGT => (lhs > rhs) as u64,
            OperationCode::FGE => (lhs >= rhs) as u64,
            _ => unreachable!(),
        };

        Ok(())
    }

    // ====== Инструкции перехода ======
    //
    // * `JMP`,
    // * `JZ`,
    // * `JNZ`,
    // * `CALL`,
    // * `RET`.

    fn jump(&mut self, instr: Instruction, ip: &mut usize) -> Result<bool, VMError> {
        let mut jumped = false;

        match instr.opcode {
            OperationCode::JMP => {
                let offset = instr.expect1()?;
                *ip = self.get_value(offset)? as usize;
                jumped = true;
            }
            OperationCode::JZ => {
                let (value, offset) = instr.expect2()?;
                if self.get_value(value)? == 0 {
                    *ip = self.get_value(offset)? as usize;
                    jumped = true;
                }
            }
            OperationCode::JNZ => {
                let (value, offset) = instr.expect2()?;
                if self.get_value(value)? != 0 {
                    *ip = self.get_value(offset)? as usize;
                    jumped = true;
                }
            }
            OperationCode::CALL => {
                let offset = instr.expect1()?;
                self.call_stack.push(*ip);
                *ip = self.get_value(offset)? as usize;
                jumped = true;
            }
            OperationCode::RET => {
                if instr.operand_count() != 0 {
                    return Err(VMError::new(VMErrorKind::IncorrectNumberOfOperands {
                        expected: 0,
                        got: instr.operand_count() as u8,
                    }));
                }
                *ip = self
                    .call_stack
                    .pop()
                    .ok_or_else(|| VMError::new(VMErrorKind::EmptyCallStack))?;
                jumped = true;
            }
            _ => unreachable!(),
        }

        Ok(jumped)
    }

    // ====== Вспомогательные функции ======

    /// Получить значение из операнда (вне зависимости от типа)
    fn get_value(&self, operand: Operand) -> Result<u64, VMError> {
        match operand.kind {
            OperandKind::Register(r) => Ok(self.registers[r]),
            OperandKind::Immediate(v) => Ok(v),
        }
    }

    /// Получить значение из регистра.
    fn get_register(&self, operand: Operand) -> Result<Register, VMError> {
        match operand.kind {
            OperandKind::Register(r) => Ok(r),
            got => Err(VMError::new(VMErrorKind::IncorrectTypeOfOperand {
                expected: OperandKind::Register(Register(0)),
                got,
            })),
        }
    }
}

#[inline]
fn ensure_nonzero_divisor(rhs: u64) -> VMResult {
    if rhs == 0 {
        Err(VMError::new(VMErrorKind::DivisionByZero))
    } else {
        Ok(())
    }
}
