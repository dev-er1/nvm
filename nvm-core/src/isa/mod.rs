//! # NVM ISA
//!
//! The ISA (Instruction Set Architecture) of NVM is the specification
//! of the instruction set of the NVM virtual machine. It defines:
//!
//! - the set of supported instructions;
//! - their encoding format;
//! - the allowed operands;
//! - the data representation;
//! - the memory and register model.
//!
//! The ISA describes **what** the virtual machine must do.
//!
//! ## Module contents
//!
//! - [`instruction`] — the representation of an NVM instruction;
//! - [`opcode`] — operation codes (opcodes);
//! - [`operand`] — the representation of instruction operands;
//! - [`register`] — virtual machine register identifiers;
//! - [`err`] — errors related to the ISA.
pub mod err;
pub mod instruction;
pub mod opcode;
pub mod operand;
pub mod register;
