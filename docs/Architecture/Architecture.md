# NVM Architecture
This document describes the architecture of NVM.

## Table of Contents
- [Definitions](#definitions)
- [NVM in More Detail](#nvm-in-more-detail)
- [Pipeline](#pipeline)
  - [Bytecode](#bytecode)
  - [Memory Allocation](#memory-allocation)
  - [Bytecode Execution](#bytecode-execution)
- [Where Everything Is Located](#where-everything-is-located)

## Definitions
- **"Opcode"** — the operation code of an instruction that determines the instruction's type. An opcode is represented as a single byte.
- **"Operand"** — an argument of an instruction that can be either a register or an immediate value.
- **"Instruction"** — a command that performs a specific operation. Each instruction has an opcode that determines its type and may have operands that specify arguments for the operation.
- **"Bytecode"** — a sequence of instructions represented as bytes.

---

## NVM in More Detail
NVM has 255 64-bit registers. Each instruction contains:
- 1 opcode;
- 3 optional operands.

There are 2 types of operands:

- Register (1 byte);
- Immediate value (8 bytes).

## Pipeline
```mermaid
flowchart TD
File[File] --> |Load bytecode| Bytecode[Bytecode]
Bytecode --> |VM initialization| Execution(Bytecode execution)
```

Let's go through each stage:

### Bytecode
First, NVM loads bytecode from a file in the NVM Bytecode format (see the [documentation](../File-Format/FILE-FORMAT.md)).

The loader checks:
- the file header;
- whether NVM can execute the bytecode (whether the version is supported).

After the header, the loader starts reading instructions.

Instructions are read as follows:
1. Read the instruction opcode (1 byte).
2. Determine the number of operands following the opcode (1 byte).
3. Determine how many bytes the next operand occupies based on its tag (`0x00` — register, 1 byte; `0x01` — immediate value, 8 bytes).
4. Read the operand.

Steps 3 and 4 are repeated for each operand of the instruction.

The result is a `Vec<Instruction>`.

---

### Memory Allocation
After the bytecode has been loaded, NVM allocates memory (64 KB by default if no memory size was specified) for program execution.
The memory itself is simply a sequence of bytes that can be interpreted in any way.

#### Why Memory Is Needed
Memory is needed when there are not enough registers to store data.

---

### Bytecode Execution
After the bytecode has been loaded and memory has been allocated, the program is executed. NVM executes bytecode in two ways:
1. **Using the standard executor.** The implementation simply uses a `match` on the opcode and executes the corresponding instruction.
2. **Using a Jump Table.** An address of the instruction handler is stored in advance for each opcode. During execution, NVM obtains the opcode and jumps directly to the corresponding handler.

#### What Is a Jump Table?
A Jump Table is an array of addresses of instruction handlers. The opcode is used as the array index:
```text
jump_table[opcode] -> handler
```

When executing an instruction, NVM:
1. Reads the opcode.
2. Uses the opcode as an index into the Jump Table.
3. Jumps to the corresponding handler.
4. The handler executes the instruction.
5. Control is passed to the next instruction.

---

## Where Everything Is Located
- ISA:
  - Module path: `nvm-core/src/isa/`
  - Opcode enumeration: [`isa/opcode.rs`](../../nvm-core/src/isa/opcode.rs)
  - Operand types and structure: [`isa/operand.rs`](../../nvm-core/src/isa/operand.rs)
  - `Register` structure: [`isa/register.rs`](../../nvm-core/src/isa/register.rs)
  - Instruction structure: [`isa/instruction.rs`](../../nvm-core/src/isa/instruction.rs)
  - Error enumeration: [`isa/err.rs`](../../nvm-core/src/isa/err.rs)
- Loader:
  - Module path: `nvm-core/src/loader/`
  - Loader implementation: [`loader/mod.rs`](../../nvm-core/src/loader/mod.rs)
  - Error enumeration: [`loader/err.rs`](../../nvm-core/src/loader/err.rs)
- VM:
  - Module path: `nvm-core/src/vm/`
  - VM structure definition: [`vm/mod.rs`](../../nvm-core/src/vm/mod.rs)
  - Error enumeration: [`vm/err.rs`](../../nvm-core/src/vm/err.rs)
  - Standard executor: [`vm/default.rs`](../../nvm-core/src/vm/default.rs)
  - Jump Table: [`vm/jumptable.rs`](../../nvm-core/src/vm/jumptable.rs)
  - Memory structure: [`vm/memory.rs`](../../nvm-core/src/vm/memory.rs)
  - Register file: [`vm/register_file.rs`](../../nvm-core/src/vm/register_file.rs)
