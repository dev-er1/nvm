# NVM Bytecode Format (`.nb`)
This document describes the binary format used to store NVM bytecode for execution by the NVM virtual machine.

## Contents
- [Byte Order](#byte-order)
- [NVM Bytecode File Structure](#nvm-bytecode-file-structure)
- [Magic Signature](#magic-signature)
- [Minimum NVM Version](#minimum-nvm-version)
- [Bytecode](#bytecode)

## Byte Order
All multi-byte integer values in the NVM Bytecode format are stored in **Little-Endian** byte order.

---

## NVM Bytecode File Structure
| Offset | Size    | Section     |
|:------:|:-------:|-------------|
| 0      | 5 bytes | Magic       |
| 5      | 6 bytes | NVM Version |
| 11     | —       | Bytecode    |

---

## Magic Signature
The first 5 bytes of the file must be:
```text
4E 56 4D 42 43
```
(`NVMBC`)

This signature identifies the file as an NVM Bytecode file.

---

## Minimum NVM Version
Immediately following the magic signature are **6 bytes** specifying the minimum required version of the NVM virtual machine.

The version is stored as three consecutive `u16` values.
```text
<u16><u16><u16>
```

The virtual machine must compare the minimum required version stored in the file with its own version. If the virtual machine's version is lower than the required version, loading the file must fail with a version compatibility error.

---

## Bytecode
The file header is immediately followed by a stream of instructions.

### Instruction Encoding
```
[opcode: u8]                 — OperationCode (see opcode.rs)
[operand_count: u8]          — number of operands, 0–3
[operand₁]                   — if count ≥ 1
[operand₂]                   — if count ≥ 2
[operand₃]                   — if count ≥ 3
```

### Operand Encoding
Each operand starts with a 1-byte tag followed by its data:

| Tag  | Type      | Data                                  |
|:----:|-----------|---------------------------------------|
| 0x00 | Register  | 1 byte — register number (`u8`)       |
| 0x01 | Immediate | 8 bytes — value (`u64` Little-Endian) |

### Examples

`NOP` (0 operands):
```
[0x00] [0x00]
```

`MOVE r0, 42` (2 operands: Register, Immediate):
```
[0x05] [0x02]
[0x00] [0x00]       ; reg(0)
[0x01] [0x2A 0x00 ... 0x00]  ; imm(42) LE
```

`IADD r0, r1, r2` (3 operands: Register, Register, Register):
```
[0x07] [0x03]
[0x00] [0x00]       ; reg(0)
[0x00] [0x01]       ; reg(1)
[0x00] [0x02]       ; reg(2)
```
